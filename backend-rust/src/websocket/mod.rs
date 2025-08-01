use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures::{sink::SinkExt, stream::StreamExt};
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::{
    sync::Arc,
    io::{Read, Write},
};
use tokio::{
    sync::{mpsc, Mutex},
    task::JoinHandle,
};
use tracing::{debug, error, info};
use uuid::Uuid;

use crate::{
    audio,
    tmux,
    types::*,
    AppState,
};

type ClientId = String;

struct PtySession {
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    master: Arc<Mutex<Box<dyn portable_pty::MasterPty + Send>>>,
    reader_task: JoinHandle<()>,
    child: Arc<Mutex<Box<dyn portable_pty::Child + Send>>>,
    tmux_session: String,
}

struct WsState {
    client_id: ClientId,
    current_pty: Arc<Mutex<Option<PtySession>>>,
    audio_tx: Option<mpsc::UnboundedSender<ServerMessage>>,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, _state: Arc<AppState>) {
    let client_id = Uuid::new_v4().to_string();
    info!("New WebSocket connection established: {}", client_id);

    let (mut sender, mut receiver) = socket.split();
    
    // Create channel for server messages
    let (tx, mut rx) = mpsc::unbounded_channel::<ServerMessage>();
    
    let mut ws_state = WsState {
        client_id: client_id.clone(),
        current_pty: Arc::new(Mutex::new(None)),
        audio_tx: None,
    };
    
    // Spawn task to forward server messages to WebSocket
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = sender.send(Message::Text(json)).await;
            }
        }
    });

    // Handle incoming messages
    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            Message::Text(text) => {
                if let Ok(ws_msg) = serde_json::from_str::<WebSocketMessage>(&text) {
                    if let Err(e) = handle_message(ws_msg, &mut ws_state, &tx).await {
                        error!("Error handling message: {}", e);
                    }
                }
            }
            Message::Close(_) => {
                info!("WebSocket connection closed: {}", client_id);
                break;
            }
            _ => {
                debug!("Ignoring WebSocket message type: {:?}", msg);
            }
        }
    }

    // Cleanup
    cleanup_session(&ws_state).await;
}

async fn handle_message(
    msg: WebSocketMessage,
    state: &mut WsState,
    tx: &mpsc::UnboundedSender<ServerMessage>,
) -> anyhow::Result<()> {
    match msg {
        WebSocketMessage::ListSessions => {
            let sessions = tmux::list_sessions().await.unwrap_or_default();
            let response = ServerMessage::SessionsList { sessions };
            tx.send(response)?;
        }
        
        WebSocketMessage::AttachSession { session_name, cols, rows } => {
            info!("Attaching to session: {}", session_name);
            attach_to_session(tx, state, &session_name, cols, rows).await?;
        }
        
        WebSocketMessage::Input { data } => {
            let pty_opt = state.current_pty.lock().await;
            if let Some(ref pty) = *pty_opt {
                let mut writer = pty.writer.lock().await;
                if let Err(e) = writer.write_all(data.as_bytes()) {
                    error!("Failed to write to PTY: {}", e);
                    return Err(e.into());
                }
                writer.flush()?;
            } else {
                debug!("No PTY session active, ignoring input");
            }
        }
        
        WebSocketMessage::Resize { cols, rows } => {
            let pty_opt = state.current_pty.lock().await;
            if let Some(ref pty) = *pty_opt {
                let master = pty.master.lock().await;
                master.resize(PtySize {
                    rows,
                    cols,
                    pixel_width: 0,
                    pixel_height: 0,
                })?;
                debug!("Resized PTY to {}x{}", cols, rows);
            } else {
                debug!("No PTY session active, ignoring resize");
            }
        }
        
        WebSocketMessage::ListWindows { session_name } => {
            let windows = tmux::list_windows(&session_name).await.unwrap_or_default();
            let response = ServerMessage::WindowsList { windows };
            tx.send(response)?;
        }
        
        WebSocketMessage::SelectWindow { session_name, window_index } => {
            debug!("Selecting window {} in session {}", window_index, session_name);
            match tmux::select_window(&session_name, &window_index.to_string()).await {
                Ok(_) => {
                    // Send refresh command to PTY
                    let pty_opt = state.current_pty.lock().await;
                    if let Some(ref pty) = *pty_opt {
                        let mut writer = pty.writer.lock().await;
                        writer.write_all(b"\x0c")?; // Ctrl-L
                        writer.flush()?;
                    }
                    
                    let response = ServerMessage::WindowSelected {
                        success: true,
                        window_index: Some(window_index),
                        error: None,
                    };
                    tx.send(response)?;
                    
                    // Refresh windows list
                    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                    let windows = tmux::list_windows(&session_name).await.unwrap_or_default();
                    let windows_response = ServerMessage::WindowsList { windows };
                    tx.send(windows_response)?;
                }
                Err(e) => {
                    let response = ServerMessage::WindowSelected {
                        success: false,
                        window_index: None,
                        error: Some(e.to_string()),
                    };
                    tx.send(response)?;
                }
            }
        }
        
        WebSocketMessage::Ping => {
            tx.send(ServerMessage::Pong)?;
        }
        
        WebSocketMessage::AudioControl { action } => {
            info!("Received audio control: {:?}", action);
            match action {
                AudioAction::Start => {
                    info!("Starting audio streaming for client");
                    state.audio_tx = Some(tx.clone());
                    audio::start_streaming(tx.clone()).await?;
                }
                AudioAction::Stop => {
                    info!("Stopping audio streaming for client");
                    state.audio_tx = None;
                    audio::stop_streaming_for_client(&tx).await?;
                }
            }
        }
    }
    
    Ok(())
}

async fn attach_to_session(
    tx: &mpsc::UnboundedSender<ServerMessage>,
    state: &WsState,
    session_name: &str,
    cols: u16,
    rows: u16,
) -> anyhow::Result<()> {
    // Clean up any existing PTY session first
    let mut pty_guard = state.current_pty.lock().await;
    if let Some(old_pty) = pty_guard.take() {
        debug!("Cleaning up previous PTY session for tmux: {}", old_pty.tmux_session);
        // Kill the child process
        {
            let mut child = old_pty.child.lock().await;
            let _ = child.kill();
            let _ = child.wait();
        }
        // Abort the reader task
        old_pty.reader_task.abort();
        let _ = old_pty.reader_task.await;
    }
    
    // Small delay to ensure cleanup is complete
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    
    // Create new PTY session
    debug!("Creating new PTY session for: {}", session_name);
    
    let pty_system = native_pty_system();
    let pair = pty_system.openpty(PtySize {
        rows,
        cols,
        pixel_width: 0,
        pixel_height: 0,
    })?;
    
    let mut cmd = CommandBuilder::new("bash");
    cmd.args(&["-l"]); // Login shell to source profile
    cmd.env("TERM", "xterm-256color");
    cmd.env("COLORTERM", "truecolor");
    
    // Clear SSH-related environment variables that might confuse starship
    cmd.env_remove("SSH_CLIENT");
    cmd.env_remove("SSH_CONNECTION");
    cmd.env_remove("SSH_TTY");
    cmd.env_remove("SSH_AUTH_SOCK");
    
    // Set up proper environment for local terminal
    cmd.env("WEBMUX", "1");
    
    if let Ok(home) = std::env::var("HOME") {
        cmd.cwd(home);
    }
    
    // Get reader before we move master
    let reader = pair.master.try_clone_reader()?;
    
    // Get writer and spawn command
    let writer = pair.master.take_writer()?;
    let writer = Arc::new(Mutex::new(writer));
    
    let child = pair.slave.spawn_command(cmd)?;
    let child: Arc<Mutex<Box<dyn portable_pty::Child + Send>>> = Arc::new(Mutex::new(child));
    
    // Attach to tmux session
    {
        let mut w = writer.lock().await;
        let attach_cmd = format!("tmux attach-session -t '{}' || tmux new-session -s '{}'\r", session_name, session_name);
        w.write_all(attach_cmd.as_bytes())?;
        w.flush()?;
    }
    
    // Set up reader task
    let tx_clone = tx.clone();
    let client_id = state.client_id.clone();
    let reader_task = tokio::task::spawn_blocking(move || {
        let mut reader = reader;
        let mut buffer = vec![0u8; 8192]; // Larger buffer for better performance
        let mut consecutive_errors = 0;
        
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => {
                    info!("PTY EOF for client {}", client_id);
                    break;
                }
                Ok(n) => {
                    consecutive_errors = 0; // Reset error counter
                    let data = String::from_utf8_lossy(&buffer[..n]).to_string();
                    
                    // Send in chunks if needed
                    const MAX_CHUNK_SIZE: usize = 32 * 1024;
                    if data.len() > MAX_CHUNK_SIZE {
                        for chunk in data.as_bytes().chunks(MAX_CHUNK_SIZE) {
                            let chunk_str = String::from_utf8_lossy(chunk).to_string();
                            let output = ServerMessage::Output { data: chunk_str };
                            if tx_clone.send(output).is_err() {
                                error!("Client {} disconnected, stopping PTY reader", client_id);
                                break;
                            }
                        }
                    } else {
                        let output = ServerMessage::Output { data };
                        if tx_clone.send(output).is_err() {
                            error!("Client {} disconnected, stopping PTY reader", client_id);
                            break;
                        }
                    }
                }
                Err(e) => {
                    consecutive_errors += 1;
                    if consecutive_errors > 5 {
                        error!("Too many consecutive PTY read errors for client {}: {}", client_id, e);
                        break;
                    }
                    error!("PTY read error for client {} (attempt {}): {}", client_id, consecutive_errors, e);
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            }
        }
        
        let _ = tx_clone.send(ServerMessage::Disconnected);
    });
    
    let pty_session = PtySession {
        writer: writer.clone(),
        master: Arc::new(Mutex::new(pair.master)),
        reader_task,
        child,
        tmux_session: session_name.to_string(),
    };
    
    *pty_guard = Some(pty_session);
    drop(pty_guard);
    
    // Send attached confirmation
    let response = ServerMessage::Attached {
        session_name: session_name.to_string(),
    };
    tx.send(response)?;
    
    Ok(())
}

async fn cleanup_session(state: &WsState) {
    info!("Cleaning up session for client: {}", state.client_id);
    
    // Clean up PTY session
    let mut pty_guard = state.current_pty.lock().await;
    if let Some(pty) = pty_guard.take() {
        info!("Cleaning up PTY for tmux session: {}", pty.tmux_session);
        
        // Kill the child process first
        {
            let mut child = pty.child.lock().await;
            let _ = child.kill();
            let _ = child.wait();
        }
        
        // Abort the reader task
        pty.reader_task.abort();
        
        // Writer and master will be dropped automatically
    }
    drop(pty_guard);
    
    // Clean up audio streaming
    if let Some(ref audio_tx) = state.audio_tx {
        if let Err(e) = audio::stop_streaming_for_client(audio_tx).await {
            error!("Failed to stop audio streaming: {}", e);
        }
    }
}