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

mod session_manager;
use self::session_manager::TmuxSessionManager;

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

async fn switch_to_session(
    tx: &mpsc::UnboundedSender<ServerMessage>,
    state: &WsState,
    session_name: &str,
) -> anyhow::Result<()> {
    // Stop any existing output capture
    let mut output_task = state.output_task.lock().await;
    if let Some(task) = output_task.take() {
        task.abort();
    }
    
    // Switch to the new session
    state.session_manager.switch_session(session_name).await?;
    
    // Get initial content
    let initial_content = state.session_manager.capture_pane().await?;
    if !initial_content.is_empty() {
        tx.send(ServerMessage::Output { data: initial_content })?;
    }
    
    // Start continuous capture
    let tx_clone = tx.clone();
    let session_manager = state.session_manager.clone();
    let session = session_name.to_string();
    
    let capture_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(100));
        let mut last_content = String::new();
        let mut last_line_count = 0;
        
        loop {
            interval.tick().await;
            
            match session_manager.capture_pane().await {
                Ok(content) => {
                    let current_lines: Vec<&str> = content.lines().collect();
                    let current_line_count = current_lines.len();
                    
                    // Detect what changed
                    if content != last_content {
                        // If content scrolled (new lines added at bottom)
                        if current_line_count > last_line_count {
                            // Send only new lines
                            let new_lines = current_lines[last_line_count..]
                                .join("\n");
                            if !new_lines.is_empty() {
                                let msg = ServerMessage::Output { data: new_lines + "\n" };
                                if tx_clone.send(msg).is_err() {
                                    break;
                                }
                            }
                        } else {
                            // Full refresh (cleared screen, etc)
                            let msg = ServerMessage::Output { data: content.clone() };
                            if tx_clone.send(msg).is_err() {
                                break;
                            }
                        }
                        
                        last_content = content;
                        last_line_count = current_line_count;
                    }
                }
                Err(e) => {
                    error!("Failed to capture pane: {}", e);
                }
            }
        }
    });
    
    *output_task = Some(capture_task);
    
    // Send attached confirmation
    tx.send(ServerMessage::Attached {
        session_name: session_name.to_string(),
    })?;
    
    Ok(())
}

async fn cleanup_session(state: &WsState) {
    info!("Cleaning up session for client: {}", state.client_id);
    
    // Stop output capture task
    let mut output_task = state.output_task.lock().await;
    if let Some(task) = output_task.take() {
        task.abort();
    }
    
    // Clean up session manager
    state.session_manager.cleanup().await;
    
    // Clean up audio streaming
    if let Some(ref audio_tx) = state.audio_tx {
        if let Err(e) = audio::stop_streaming_for_client(audio_tx).await {
            error!("Failed to stop audio streaming: {}", e);
        }
    }
}