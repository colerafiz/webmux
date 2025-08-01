use anyhow::Result;
use std::{
    process::Stdio,
    sync::Arc,
};
use tokio::{
    io::AsyncReadExt,
    process::{Child, Command},
    sync::{mpsc, Mutex},
};
use tracing::{error, info};
use bytes::Bytes;

use crate::{types::ServerMessage, websocket::BroadcastMessage};

type AudioClient = mpsc::UnboundedSender<BroadcastMessage>;

lazy_static::lazy_static! {
    static ref AUDIO_STATE: Arc<Mutex<AudioState>> = Arc::new(Mutex::new(AudioState::default()));
}

#[derive(Default)]
struct AudioState {
    ffmpeg_process: Option<Child>,
    is_streaming: bool,
    clients: Vec<AudioClient>,
}

pub async fn start_streaming(client_tx: mpsc::UnboundedSender<BroadcastMessage>) -> Result<()> {
    let mut state = AUDIO_STATE.lock().await;
    
    // Add client
    state.clients.push(client_tx.clone());
    info!("Audio client added. Total clients: {}", state.clients.len());
    
    // Send current status
    let status = ServerMessage::AudioStatus {
        streaming: state.is_streaming,
        error: None,
    };
    if let Ok(json) = serde_json::to_string(&status) {
        let _ = client_tx.send(BroadcastMessage::Text(Arc::new(json)));
    }
    
    // Start streaming if not already running
    if !state.is_streaming {
        start_ffmpeg(&mut state).await?;
    }
    
    Ok(())
}

pub async fn stop_streaming_for_client(client_tx: &mpsc::UnboundedSender<BroadcastMessage>) -> Result<()> {
    let mut state = AUDIO_STATE.lock().await;
    
    // Remove only this specific client
    state.clients.retain(|c| !c.same_channel(client_tx));
    info!("Audio client removed. Remaining clients: {}", state.clients.len());
    
    // Only stop ffmpeg if no clients remain
    if state.clients.is_empty() && state.is_streaming {
        stop_ffmpeg(&mut state).await;
    }
    
    Ok(())
}



async fn get_default_monitor_source() -> Result<String> {
    // Get the default sink first
    let output = Command::new("pactl")
        .args(&["get-default-sink"])
        .output()
        .await?;
    
    if !output.status.success() {
        return Err(anyhow::anyhow!("Failed to get default sink"));
    }
    
    let sink = String::from_utf8_lossy(&output.stdout).trim().to_string();
    // Append .monitor to get the monitor source
    Ok(format!("{}.monitor", sink))
}

async fn start_ffmpeg(state: &mut AudioState) -> Result<()> {
    info!("Starting audio streaming...");
    state.is_streaming = true;
    
    // Determine platform-specific input args
    let (input_source, input_args) = if cfg!(target_os = "linux") {
        // First try to get the default monitor source
        match get_default_monitor_source().await {
            Ok(source) => {
                info!("Using PulseAudio monitor source: {}", source);
                (source, vec!["-f", "pulse", "-i"])
            }
            Err(_) => {
                info!("Using default PulseAudio source");
                ("default".to_string(), vec!["-f", "pulse", "-i"])
            }
        }
    } else if cfg!(target_os = "macos") {
        (":0".to_string(), vec!["-f", "avfoundation", "-i"])
    } else {
        error!("Unsupported platform for audio capture");
        state.is_streaming = false;
        notify_clients_error(state, "Unsupported platform for audio capture").await;
        return Err(anyhow::anyhow!("Unsupported platform"));
    };
    
    // Spawn ffmpeg process
    let mut child = Command::new("ffmpeg")
        .args(&input_args)
        .arg(&input_source)
        .args(&[
            "-acodec", "libopus",
            "-b:a", "128k",
            "-ar", "48000",
            "-ac", "2",
            "-f", "webm",
            "-"
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    
    // Get stdout for reading audio data
    let mut stdout = child.stdout.take().unwrap();
    
    // Clone clients for the spawned task
    let clients_clone: Vec<AudioClient> = state.clients.clone();
    
    // Spawn task to read and broadcast audio data
    tokio::spawn(async move {
        let mut buffer = vec![0u8; 16384]; // Larger buffer for better throughput
        loop {
            match stdout.read(&mut buffer).await {
                Ok(0) => break, // EOF
                Ok(n) => {
                    // Send as binary frame for efficiency
                    let data = Bytes::copy_from_slice(&buffer[..n]);
                    info!("Sending audio chunk: {} bytes", n);
                    broadcast_binary_to_clients(&clients_clone, data).await;
                }
                Err(e) => {
                    error!("Error reading ffmpeg output: {}", e);
                    break;
                }
            }
        }
    });
    
    // Spawn task to monitor stderr
    if let Some(mut stderr) = child.stderr.take() {
        tokio::spawn(async move {
            let mut buffer = vec![0u8; 1024];
            while let Ok(n) = stderr.read(&mut buffer).await {
                if n == 0 { break; }
                if crate::ENABLE_AUDIO_LOGS.load(std::sync::atomic::Ordering::Relaxed) {
                    let log = String::from_utf8_lossy(&buffer[..n]);
                    info!("FFmpeg: {}", log);
                }
            }
        });
    }
    
    state.ffmpeg_process = Some(child);
    
    // Notify clients that streaming started
    notify_clients_status(state, true).await;
    
    Ok(())
}

async fn stop_ffmpeg(state: &mut AudioState) {
    info!("Stopping audio streaming...");
    
    if let Some(mut child) = state.ffmpeg_process.take() {
        let _ = child.kill().await;
    }
    
    state.is_streaming = false;
    notify_clients_status(state, false).await;
}

async fn notify_clients_status(state: &AudioState, streaming: bool) {
    let msg = ServerMessage::AudioStatus {
        streaming,
        error: None,
    };
    if let Ok(json) = serde_json::to_string(&msg) {
        let broadcast_msg = BroadcastMessage::Text(Arc::new(json));
        for client in &state.clients {
            let _ = client.send(broadcast_msg.clone());
        }
    }
}

async fn notify_clients_error(state: &AudioState, error: &str) {
    let msg = ServerMessage::AudioStatus {
        streaming: false,
        error: Some(error.to_string()),
    };
    if let Ok(json) = serde_json::to_string(&msg) {
        let broadcast_msg = BroadcastMessage::Text(Arc::new(json));
        for client in &state.clients {
            let _ = client.send(broadcast_msg.clone());
        }
    }
}

async fn broadcast_binary_to_clients(
    clients: &[AudioClient],
    data: Bytes,
) {
    info!("Broadcasting binary to {} clients", clients.len());
    let msg = BroadcastMessage::Binary(data);
    for client in clients {
        let _ = client.send(msg.clone());
    }
}