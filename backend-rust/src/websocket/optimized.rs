use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use bytes::{Bytes, BytesMut};
use futures::{sink::SinkExt, stream::StreamExt};
use std::{
    sync::Arc,
    time::Duration,
    collections::VecDeque,
};
use tokio::{
    sync::{mpsc, Mutex, RwLock, Semaphore},
    time::{interval, Instant},
};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{
    tmux,
    types::*,
    AppState,
    websocket::optimized_session_manager::{OptimizedSessionManager, ManagerConfig},
};

// Constants for performance tuning
const OUTPUT_BUFFER_SIZE: usize = 65536; // 64KB buffer
const MAX_BATCH_SIZE: usize = 32; // Max messages to batch
const BATCH_TIMEOUT_MS: u64 = 5; // Max time to wait for batching
const BACKPRESSURE_THRESHOLD: usize = 256; // Queue size before applying backpressure
const MAX_MESSAGE_SIZE: usize = 1048576; // 1MB max message size

/// Binary protocol message types
#[repr(u8)]
pub enum BinaryMessageType {
    Output = 0x01,
    Resize = 0x02,
    Input = 0x03,
    WindowSwitch = 0x04,
    Stats = 0x05,
    Ping = 0x06,
    Pong = 0x07,
}

/// Optimized message for zero-copy broadcasting
#[derive(Clone)]
pub enum OptimizedMessage {
    /// Pre-serialized JSON message
    Json(Arc<str>),
    /// Binary message with type prefix
    Binary(Bytes),
    /// Terminal output with efficient encoding
    TerminalOutput(Bytes),
}

/// Client connection state with backpressure control
pub struct ClientConnection {
    id: String,
    tx: mpsc::Sender<OptimizedMessage>,
    /// Semaphore for backpressure control
    permits: Arc<Semaphore>,
    /// Last activity timestamp
    last_activity: Instant,
    /// Client-specific settings
    binary_mode: bool,
    compression_enabled: bool,
}


/// High-performance client manager with connection pooling
pub struct OptimizedClientManager {
    /// Active client connections
    clients: Arc<RwLock<dashmap::DashMap<String, ClientConnection>>>,
    /// Session manager
    session_manager: Arc<OptimizedSessionManager>,
    /// Message batching queue
    batch_queue: Arc<Mutex<VecDeque<(String, OptimizedMessage)>>>,
    /// Stats for monitoring
    stats: Arc<RwLock<PerformanceStats>>,
}

#[derive(Default)]
pub struct PerformanceStats {
    messages_sent: u64,
    messages_batched: u64,
    bytes_sent: u64,
    backpressure_events: u64,
    active_sessions: usize,
    active_clients: usize,
}

impl OptimizedClientManager {
    pub fn new() -> Self {
        let manager = Self {
            clients: Arc::new(RwLock::new(dashmap::DashMap::new())),
            session_manager: Arc::new(OptimizedSessionManager::new(ManagerConfig::default())),
            batch_queue: Arc::new(Mutex::new(VecDeque::with_capacity(1024))),
            stats: Arc::new(RwLock::new(PerformanceStats::default())),
        };
        
        // Start batch processor
        let queue = manager.batch_queue.clone();
        let clients = manager.clients.clone();
        tokio::spawn(async move {
            process_message_batches(queue, clients).await;
        });
        
        manager
    }
    
    pub async fn add_client(&self, client_id: String, binary_mode: bool) -> mpsc::Receiver<OptimizedMessage> {
        let (tx, rx) = mpsc::channel(BACKPRESSURE_THRESHOLD);
        let permits = Arc::new(Semaphore::new(BACKPRESSURE_THRESHOLD));
        
        let connection = ClientConnection {
            id: client_id.clone(),
            tx,
            permits: permits.clone(),
            last_activity: Instant::now(),
            binary_mode,
            compression_enabled: false,
        };
        
        self.clients.write().await.insert(client_id.clone(), connection);
        
        let mut stats = self.stats.write().await;
        stats.active_clients += 1;
        
        info!("Client {} added (binary_mode: {})", client_id, binary_mode);
        rx
    }
    
    pub async fn remove_client(&self, client_id: &str) {
        self.clients.write().await.remove(client_id);
        
        let mut stats = self.stats.write().await;
        stats.active_clients = stats.active_clients.saturating_sub(1);
        
        info!("Client {} removed", client_id);
    }
    
    /// Broadcast message with automatic batching and backpressure
    pub async fn broadcast(&self, session_name: &str, message: OptimizedMessage) {
        // Get all clients in this session from the session manager
        // For now, broadcast to all connected clients
        let clients = self.clients.read().await;
        for entry in clients.iter() {
            self.send_to_client(entry.key(), message.clone()).await;
        }
    }
    
    /// Send message to specific client with backpressure handling
    pub async fn send_to_client(&self, client_id: &str, message: OptimizedMessage) {
        // First check if client exists and get what we need
        let client_info = {
            let clients = self.clients.read().await;
            clients.get(client_id).map(|client| {
                let available_permits = client.permits.available_permits();
                (available_permits, client.permits.clone(), client.tx.clone())
            })
        };
        
        match client_info {
            Some((available_permits, permits, tx)) => {
                if available_permits == 0 {
                    // Client exists but backpressure triggered
                    let mut stats = self.stats.write().await;
                    stats.backpressure_events += 1;
                    warn!("Backpressure triggered for client {}", client_id);
                } else {
                    // Try to acquire permit without blocking
                    if let Ok(_permit) = permits.try_acquire_owned() {
                        if let Err(e) = tx.try_send(message) {
                            error!("Failed to send to client {}: {}", client_id, e);
                        }
                    }
                }
            }
            None => {
                // Client doesn't exist
            }
        }
    }
    
    /// Attach client to a shared session using capture-pane approach
    pub async fn attach_to_session(&self, client_id: &str, session_name: &str) -> anyhow::Result<()> {
        // Use the session manager to handle the session
        let clients = self.clients.read().await;
        if let Some(client) = clients.get(client_id) {
            // Create a channel for session output
            let (tx, mut rx) = mpsc::channel(256);
            
            // Add client to session in the session manager
            self.session_manager.add_client_to_session(
                session_name,
                client_id.to_string(),
                tx,
            ).await?;
            
            // Forward messages from session to client
            let client_tx = client.tx.clone();
            let client_id = client_id.to_string();
            tokio::spawn(async move {
                while let Some(data) = rx.recv().await {
                    if let Err(e) = client_tx.try_send(OptimizedMessage::TerminalOutput(data)) {
                        error!("Failed to forward to client {}: {}", client_id, e);
                        break;
                    }
                }
            });
        }
        
        Ok(())
    }
}

impl Clone for OptimizedClientManager {
    fn clone(&self) -> Self {
        Self {
            clients: self.clients.clone(),
            session_manager: self.session_manager.clone(),
            batch_queue: self.batch_queue.clone(),
            stats: self.stats.clone(),
        }
    }
}

/// Process message batches efficiently
async fn process_message_batches(
    queue: Arc<Mutex<VecDeque<(String, OptimizedMessage)>>>,
    clients: Arc<RwLock<dashmap::DashMap<String, ClientConnection>>>,
) {
    let mut ticker = interval(Duration::from_millis(BATCH_TIMEOUT_MS));
    let mut batch: Vec<(String, OptimizedMessage)> = Vec::with_capacity(MAX_BATCH_SIZE);
    
    loop {
        ticker.tick().await;
        
        // Collect messages for batching
        {
            let mut queue = queue.lock().await;
            while batch.len() < MAX_BATCH_SIZE && !queue.is_empty() {
                if let Some(msg) = queue.pop_front() {
                    batch.push(msg);
                }
            }
        }
        
        if batch.is_empty() {
            continue;
        }
        
        // Group messages by client
        let mut client_batches: dashmap::DashMap<String, Vec<OptimizedMessage>> = dashmap::DashMap::new();
        for (client_id, message) in batch.drain(..) {
            client_batches.entry(client_id).or_default().push(message);
        }
        
        // Send batched messages
        let clients_ref = clients.read().await;
        for (client_id, messages) in client_batches {
            if let Some(client) = clients_ref.get(&client_id) {
                // Combine terminal outputs if possible
                let combined = combine_terminal_outputs(messages);
                for msg in combined {
                    if let Err(e) = client.tx.try_send(msg) {
                        error!("Batch send failed for client {}: {}", client_id, e);
                    }
                }
            }
        }
    }
}

/// Combine multiple terminal output messages into one
fn combine_terminal_outputs(messages: Vec<OptimizedMessage>) -> Vec<OptimizedMessage> {
    let mut result = Vec::new();
    let mut terminal_buffer = BytesMut::new();
    
    for msg in messages {
        match msg {
            OptimizedMessage::TerminalOutput(data) => {
                if terminal_buffer.len() + data.len() > MAX_MESSAGE_SIZE {
                    if !terminal_buffer.is_empty() {
                        result.push(OptimizedMessage::TerminalOutput(terminal_buffer.freeze()));
                        terminal_buffer = BytesMut::new();
                    }
                }
                terminal_buffer.extend_from_slice(&data);
            }
            other => {
                if !terminal_buffer.is_empty() {
                    result.push(OptimizedMessage::TerminalOutput(terminal_buffer.freeze()));
                    terminal_buffer = BytesMut::new();
                }
                result.push(other);
            }
        }
    }
    
    if !terminal_buffer.is_empty() {
        result.push(OptimizedMessage::TerminalOutput(terminal_buffer.freeze()));
    }
    
    result
}


/// Encode terminal output in efficient binary format
fn encode_terminal_output(text: &str) -> Bytes {
    let mut buffer = BytesMut::with_capacity(text.len() + 5);
    
    // Message type
    buffer.extend_from_slice(&[BinaryMessageType::Output as u8]);
    
    // Length (4 bytes, little endian)
    buffer.extend_from_slice(&(text.len() as u32).to_le_bytes());
    
    // UTF-8 content
    buffer.extend_from_slice(text.as_bytes());
    
    buffer.freeze()
}

/// WebSocket handler with optimized message processing
pub async fn optimized_ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_optimized_socket(socket, state))
}

async fn handle_optimized_socket(socket: WebSocket, state: Arc<AppState>) {
    let client_id = Uuid::new_v4().to_string();
    info!("New optimized WebSocket connection: {}", client_id);
    
    let (mut sender, mut receiver) = socket.split();
    
    // Detect if client supports binary protocol
    let binary_mode = true; // TODO: Negotiate with client
    
    // Register client with optimized manager
    let manager = state.optimized_client_manager.clone();
    let mut rx = manager.add_client(client_id.clone(), binary_mode).await;
    
    let client_id_clone = client_id.clone();
    
    // Spawn task to handle outgoing messages
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let ws_msg = match msg {
                OptimizedMessage::Json(json) => Message::Text(json.to_string()),
                OptimizedMessage::Binary(data) | OptimizedMessage::TerminalOutput(data) => {
                    Message::Binary(data.to_vec())
                }
            };
            
            if let Err(e) = sender.send(ws_msg).await {
                error!("Failed to send to client {}: {}", client_id_clone, e);
                break;
            }
        }
    });
    
    // Handle incoming messages
    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            Message::Text(text) => {
                if let Ok(ws_msg) = serde_json::from_str::<WebSocketMessage>(&text) {
                    if let Err(e) = handle_optimized_message(ws_msg, &client_id, &manager).await {
                        error!("Error handling message: {}", e);
                    }
                }
            }
            Message::Binary(data) => {
                if let Err(e) = handle_binary_message(&data, &client_id, &manager).await {
                    error!("Error handling binary message: {}", e);
                }
            }
            Message::Close(_) => {
                info!("Client {} disconnected", client_id);
                break;
            }
            _ => {}
        }
    }
    
    // Cleanup
    send_task.abort();
    manager.remove_client(&client_id).await;
}

async fn handle_optimized_message(
    msg: WebSocketMessage,
    client_id: &str,
    manager: &OptimizedClientManager,
) -> anyhow::Result<()> {
    match msg {
        WebSocketMessage::AttachSession { session_name, .. } => {
            manager.attach_to_session(client_id, &session_name).await?;
        }
        WebSocketMessage::Input { data } => {
            // TODO: Get session name from client state and use the alternative session manager approach
            // For now, this is a placeholder
            warn!("Input handling not fully implemented in optimized handler");
        }
        // Handle other messages...
        _ => {}
    }
    
    Ok(())
}

async fn handle_binary_message(
    data: &[u8],
    client_id: &str,
    manager: &OptimizedClientManager,
) -> anyhow::Result<()> {
    if data.is_empty() {
        return Ok(());
    }
    
    match data[0] {
        x if x == BinaryMessageType::Input as u8 => {
            if data.len() > 5 {
                let len = u32::from_le_bytes([data[1], data[2], data[3], data[4]]) as usize;
                if data.len() >= 5 + len {
                    let input = std::str::from_utf8(&data[5..5 + len])?;
                    // Handle input using alternative approach
                    // TODO: Get session name from client state
                }
            }
        }
        x if x == BinaryMessageType::Ping as u8 => {
            let pong = Bytes::from_static(&[BinaryMessageType::Pong as u8]);
            manager.send_to_client(client_id, OptimizedMessage::Binary(pong)).await;
        }
        _ => {}
    }
    
    Ok(())
}