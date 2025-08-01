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
    collections::HashMap,
    sync::Arc,
    io::{Read, Write},
};
use tokio::{
    sync::{mpsc, Mutex, RwLock},
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

type SessionId = String;

struct PtySession {
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    master: Arc<Mutex<Box<dyn portable_pty::MasterPty + Send>>>,
    reader_task: JoinHandle<()>,
}

struct WsState {
    pty_sessions: Arc<RwLock<HashMap<SessionId, PtySession>>>,
    session_id: SessionId,
    audio_tx: Option<mpsc::UnboundedSender<ServerMessage>>,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, _state: Arc<AppState>) {
    let session_id = Uuid::new_v4().to_string();
    info!("New WebSocket connection established: {}", session_id);

    let (mut sender, mut receiver) = socket.split();
    
    // Create channel for server messages
    let (tx, mut rx) = mpsc::unbounded_channel::<ServerMessage>();
    
    let mut ws_state = WsState {
        pty_sessions: Arc::new(RwLock::new(HashMap::new())),
        session_id: session_id.clone(),
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
                info!("WebSocket connection closed: {}", session_id);
                break;
            }
            _ => {}
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
            let sessions = state.pty_sessions.read().await;
            if let Some(session) = sessions.get(&state.session_id) {
                let mut writer = session.writer.lock().await;
                writer.write_all(data.as_bytes())?;
                writer.flush()?;
            }
        }
        
        WebSocketMessage::Resize { cols, rows } => {
            let sessions = state.pty_sessions.read().await;
            if let Some(session) = sessions.get(&state.session_id) {
                let master = session.master.lock().await;
                master.resize(PtySize {
                    rows,
                    cols,
                    pixel_width: 0,
                    pixel_height: 0,
                })?;
                debug!("Resized PTY to {}x{}", cols, rows);
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
                    let sessions = state.pty_sessions.read().await;
                    if let Some(session) = sessions.get(&state.session_id) {
                        let mut writer = session.writer.lock().await;
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
            match action {
                AudioAction::Start => {
                    state.audio_tx = Some(tx.clone());
                    audio::start_streaming(tx.clone()).await?;
                }
                AudioAction::Stop => {
                    state.audio_tx = None;
                    audio::stop_streaming().await?;
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
    // Check if we already have a PTY for this session
    let sessions = state.pty_sessions.read().await;
    if let Some(session) = sessions.get(&state.session_id) {
        // Reuse existing PTY - just switch tmux session
        let mut writer = session.writer.lock().await;
        writer.write_all(b"\x03")?; // Ctrl-C
        writer.flush()?;
        drop(writer);
        drop(sessions);
        
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        
        let sessions = state.pty_sessions.read().await;
        if let Some(session) = sessions.get(&state.session_id) {
            let mut writer = session.writer.lock().await;
            let cmd = format!("tmux switch-client -t '{}' 2>/dev/null || tmux attach-session -t '{}'\r", session_name, session_name);
            writer.write_all(cmd.as_bytes())?;
            writer.flush()?;
        }
        drop(sessions);
        
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        let response = ServerMessage::Attached {
            session_name: session_name.to_string(),
        };
        tx.send(response)?;
        return Ok(());
    }
    drop(sessions);
    
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
    cmd.env("TERM", "xterm-256color");
    cmd.env("COLORTERM", "truecolor");
    if let Ok(home) = std::env::var("HOME") {
        cmd.cwd(home);
    }
    
    // Get reader before we move master
    let reader = pair.master.try_clone_reader()?;
    
    // Get writer and spawn command
    let writer = pair.master.take_writer()?;
    let writer = Arc::new(Mutex::new(writer));
    
    let mut _child = pair.slave.spawn_command(cmd)?;
    
    // Attach to tmux session
    {
        let mut w = writer.lock().await;
        let attach_cmd = format!("tmux attach-session -t '{}' || tmux new-session -s '{}'\r", session_name, session_name);
        w.write_all(attach_cmd.as_bytes())?;
        w.flush()?;
    }
    
    // Set up reader task using blocking I/O in a dedicated thread
    let tx_clone = tx.clone();
    let reader_task = tokio::spawn(async move {
        // Run blocking I/O in a dedicated thread
        let _ = tokio::task::spawn_blocking(move || {
            let mut reader = reader;
            let mut buffer = vec![0u8; 4096];
            
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => break, // EOF
                    Ok(n) => {
                        let data = String::from_utf8_lossy(&buffer[..n]).to_string();
                        
                        // Send in chunks if needed
                        const MAX_CHUNK_SIZE: usize = 32 * 1024;
                        if data.len() > MAX_CHUNK_SIZE {
                            for chunk in data.as_bytes().chunks(MAX_CHUNK_SIZE) {
                                let chunk_str = String::from_utf8_lossy(chunk).to_string();
                                let output = ServerMessage::Output { data: chunk_str };
                                if tx_clone.send(output).is_err() {
                                    error!("Failed to send output");
                                    break;
                                }
                            }
                        } else {
                            let output = ServerMessage::Output { data };
                            if tx_clone.send(output).is_err() {
                                error!("Failed to send output");
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        error!("Reader error: {}", e);
                        break;
                    }
                }
            }
            
            let _ = tx_clone.send(ServerMessage::Disconnected);
        }).await;
    });
    
    let pty_session = PtySession {
        writer: writer.clone(),
        master: Arc::new(Mutex::new(pair.master)),
        reader_task,
    };
    
    let mut sessions = state.pty_sessions.write().await;
    sessions.insert(state.session_id.clone(), pty_session);
    drop(sessions);
    
    // Send attached confirmation
    let response = ServerMessage::Attached {
        session_name: session_name.to_string(),
    };
    tx.send(response)?;
    
    Ok(())
}

async fn cleanup_session(state: &WsState) {
    let mut sessions = state.pty_sessions.write().await;
    if let Some(session) = sessions.remove(&state.session_id) {
        session.reader_task.abort();
        // Writer and pty_pair will be dropped automatically
    }
    
    if state.audio_tx.is_some() {
        let _ = audio::stop_streaming().await;
    }
}