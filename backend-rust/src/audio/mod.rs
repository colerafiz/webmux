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

use crate::types::ServerMessage;

type AudioClient = mpsc::UnboundedSender<ServerMessage>;

lazy_static::lazy_static! {
    static ref AUDIO_STATE: Arc<Mutex<AudioState>> = Arc::new(Mutex::new(AudioState::default()));
}

#[derive(Default)]
struct AudioState {
    ffmpeg_process: Option<Child>,
    is_streaming: bool,
    clients: Vec<AudioClient>,
}

pub async fn start_streaming(client_tx: mpsc::UnboundedSender<ServerMessage>) -> Result<()> {
    let mut state = AUDIO_STATE.lock().await;
    
    // Add client
    state.clients.push(client_tx.clone());
    info!("Audio client added. Total clients: {}", state.clients.len());
    
    // Send current status
    let status = ServerMessage::AudioStatus {
        streaming: state.is_streaming,
        error: None,
    };
    let _ = client_tx.send(status);
    
    // Start streaming if not already running
    if !state.is_streaming {
        start_ffmpeg(&mut state).await?;
    }
    
    Ok(())
}

pub async fn stop_streaming() -> Result<()> {
    let mut state = AUDIO_STATE.lock().await;
    
    state.clients.clear();
    info!("All audio clients removed");
    
    if state.is_streaming {
        stop_ffmpeg(&mut state).await;
    }
    
    Ok(())
}


async fn start_ffmpeg(state: &mut AudioState) -> Result<()> {
    info!("Starting audio streaming...");
    state.is_streaming = true;
    
    // Determine platform-specific input args
    let input_args = if cfg!(target_os = "linux") {
        vec!["-f", "pulse", "-i", "default"]
    } else if cfg!(target_os = "macos") {
        vec!["-f", "avfoundation", "-i", ":0"]
    } else {
        error!("Unsupported platform for audio capture");
        state.is_streaming = false;
        notify_clients_error(state, "Unsupported platform for audio capture").await;
        return Err(anyhow::anyhow!("Unsupported platform"));
    };
    
    // Spawn ffmpeg process
    let mut child = Command::new("ffmpeg")
        .args(&input_args)
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
        let mut buffer = vec![0u8; 4096];
        loop {
            match stdout.read(&mut buffer).await {
                Ok(0) => break, // EOF
                Ok(n) => {
                    let data = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &buffer[..n]);
                    let msg = ServerMessage::AudioStream { data };
                    broadcast_to_clients(&clients_clone, &msg).await;
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
    broadcast_to_clients(&state.clients, &msg).await;
}

async fn notify_clients_error(state: &AudioState, error: &str) {
    let msg = ServerMessage::AudioStatus {
        streaming: false,
        error: Some(error.to_string()),
    };
    broadcast_to_clients(&state.clients, &msg).await;
}

async fn broadcast_to_clients(
    clients: &[AudioClient],
    msg: &ServerMessage,
) {
    for client in clients {
        let _ = client.send(msg.clone());
    }
}