use anyhow::Result;
use bytes::{Bytes, BytesMut};
use dashmap::DashMap;
use std::{
    sync::Arc,
    time::{Duration, Instant},
    collections::VecDeque,
};
use tokio::{
    sync::{Mutex, RwLock, mpsc, Semaphore},
    time::interval,
};
use tracing::{debug, error, info, warn};

use crate::{
    tmux,
    buffer::OptimizedTerminalBuffer,
};

/// Shared session state that multiple clients can connect to
pub struct SharedTmuxSession {
    /// Session name
    name: String,
    /// Terminal dimensions
    dimensions: (u16, u16),
    /// Shared terminal buffer
    buffer: Arc<OptimizedTerminalBuffer>,
    /// Last capture time
    last_capture: Instant,
    /// Active capture task
    capture_task: Option<tokio::task::JoinHandle<()>>,
    /// Connected clients
    clients: Arc<RwLock<Vec<ClientHandle>>>,
    /// Input queue for batching
    input_queue: Arc<Mutex<InputQueue>>,
    /// Stats
    stats: Arc<Mutex<SessionStats>>,
}

#[derive(Clone)]
pub struct ClientHandle {
    pub id: String,
    pub tx: mpsc::Sender<Bytes>,
    pub joined_at: Instant,
    pub last_activity: Instant,
}

pub struct InputQueue {
    queue: VecDeque<InputCommand>,
    last_flush: Instant,
}

pub enum InputCommand {
    Text(String),
    SpecialKey(String),
    Resize(u16, u16),
}

#[derive(Default)]
pub struct SessionStats {
    pub total_captures: u64,
    pub total_inputs: u64,
    pub bytes_captured: u64,
    pub capture_errors: u64,
    pub input_errors: u64,
}

/// Optimized session manager that avoids direct PTY attachment
pub struct OptimizedSessionManager {
    /// All active sessions
    sessions: Arc<DashMap<String, Arc<RwLock<SharedTmuxSession>>>>,
    /// Global semaphore for limiting concurrent captures
    capture_semaphore: Arc<Semaphore>,
    /// Configuration
    config: ManagerConfig,
}

pub struct ManagerConfig {
    /// How often to capture pane content (milliseconds)
    pub capture_interval_ms: u64,
    /// Maximum input batch size
    pub max_input_batch: usize,
    /// Input batch timeout (milliseconds)
    pub input_batch_timeout_ms: u64,
    /// Maximum buffer size per session
    pub max_buffer_size: usize,
    /// Maximum concurrent captures
    pub max_concurrent_captures: usize,
}

impl Default for ManagerConfig {
    fn default() -> Self {
        Self {
            capture_interval_ms: 33, // ~30fps
            max_input_batch: 100,
            input_batch_timeout_ms: 5,
            max_buffer_size: 10 * 1024 * 1024, // 10MB
            max_concurrent_captures: 10,
        }
    }
}

impl OptimizedSessionManager {
    pub fn new(config: ManagerConfig) -> Self {
        Self {
            sessions: Arc::new(DashMap::new()),
            capture_semaphore: Arc::new(Semaphore::new(config.max_concurrent_captures)),
            config,
        }
    }
    
    /// Get or create a shared session
    pub async fn get_or_create_session(&self, session_name: &str) -> Result<Arc<RwLock<SharedTmuxSession>>> {
        // Check if session already exists
        if let Some(session) = self.sessions.get(session_name) {
            return Ok(session.clone());
        }
        
        // Create new session
        info!("Creating new shared session: {}", session_name);
        
        // Ensure TMUX session exists
        match tmux::list_sessions().await {
            Ok(sessions) => {
                if !sessions.iter().any(|s| s.name == session_name) {
                    tmux::create_session(session_name).await?;
                }
            }
            Err(_) => {
                tmux::create_session(session_name).await?;
            }
        }
        
        let buffer = Arc::new(OptimizedTerminalBuffer::new(self.config.max_buffer_size));
        let input_queue = Arc::new(Mutex::new(InputQueue {
            queue: VecDeque::with_capacity(self.config.max_input_batch),
            last_flush: Instant::now(),
        }));
        
        let session = Arc::new(RwLock::new(SharedTmuxSession {
            name: session_name.to_string(),
            dimensions: (80, 24),
            buffer: buffer.clone(),
            last_capture: Instant::now(),
            capture_task: None,
            clients: Arc::new(RwLock::new(Vec::new())),
            input_queue: input_queue.clone(),
            stats: Arc::new(Mutex::new(SessionStats::default())),
        }));
        
        // Start capture task
        let session_name_for_capture = session_name.to_string();
        let buffer_clone = buffer.clone();
        let capture_interval = self.config.capture_interval_ms;
        let capture_semaphore = self.capture_semaphore.clone();
        let session_guard = session.read().await;
        let clients = session_guard.clients.clone();
        let stats = session_guard.stats.clone();
        drop(session_guard);
        
        let capture_task = tokio::spawn(async move {
            capture_loop(
                session_name_for_capture,
                buffer_clone,
                clients,
                stats,
                capture_interval,
                capture_semaphore,
            ).await;
        });
        
        // Start input processor
        let session_name_for_input = session_name.to_string();
        let input_queue_clone = input_queue.clone();
        let session_guard = session.read().await;
        let stats_clone = session_guard.stats.clone();
        drop(session_guard);
        let batch_timeout = self.config.input_batch_timeout_ms;
        let max_batch = self.config.max_input_batch;
        
        tokio::spawn(async move {
            input_processor_loop(
                session_name_for_input,
                input_queue_clone,
                stats_clone,
                batch_timeout,
                max_batch,
            ).await;
        });
        
        // Store capture task handle
        session.write().await.capture_task = Some(capture_task);
        
        // Store session
        self.sessions.insert(session_name.to_string(), session.clone());
        
        Ok(session)
    }
    
    /// Add client to session
    pub async fn add_client_to_session(
        &self,
        session_name: &str,
        client_id: String,
        tx: mpsc::Sender<Bytes>,
    ) -> Result<()> {
        let session = self.get_or_create_session(session_name).await?;
        let session_guard = session.write().await;
        
        // Create client handle
        let client = ClientHandle {
            id: client_id.clone(),
            tx,
            joined_at: Instant::now(),
            last_activity: Instant::now(),
        };
        
        // Add to clients list
        session_guard.clients.write().await.push(client);
        
        info!("Client {} joined session {}", client_id, session_name);
        
        Ok(())
    }
    
    /// Remove client from session
    pub async fn remove_client_from_session(&self, session_name: &str, client_id: &str) -> Result<()> {
        if let Some(session) = self.sessions.get(session_name) {
            let session_guard = session.read().await;
            let mut clients = session_guard.clients.write().await;
            clients.retain(|c| c.id != client_id);
            
            info!("Client {} left session {}", client_id, session_name);
            
            // If no more clients, consider stopping the capture task
            if clients.is_empty() {
                info!("No more clients in session {}, stopping capture", session_name);
                drop(clients);
                drop(session_guard);
                
                if let Some(task) = session.write().await.capture_task.take() {
                    task.abort();
                }
                // Remove session after a delay to allow for quick reconnects
                let sessions = self.sessions.clone();
                let session_name = session_name.to_string();
                tokio::spawn(async move {
                    tokio::time::sleep(Duration::from_secs(30)).await;
                    if let Some(session) = sessions.get(&session_name) {
                        let session_guard = session.read().await;
                        if session_guard.clients.read().await.is_empty() {
                            drop(session_guard);
                            sessions.remove(&session_name);
                            info!("Removed idle session: {}", session_name);
                        }
                    }
                });
            }
        }
        
        Ok(())
    }
    
    /// Send input to session
    pub async fn send_input(&self, session_name: &str, input: &str) -> Result<()> {
        if let Some(session) = self.sessions.get(session_name) {
            let session_guard = session.read().await;
            let mut queue = session_guard.input_queue.lock().await;
            queue.queue.push_back(InputCommand::Text(input.to_string()));
            Ok(())
        } else {
            Err(anyhow::anyhow!("Session not found"))
        }
    }
    
    /// Send special key to session
    pub async fn send_special_key(&self, session_name: &str, key: &str) -> Result<()> {
        if let Some(session) = self.sessions.get(session_name) {
            let session_guard = session.read().await;
            let mut queue = session_guard.input_queue.lock().await;
            queue.queue.push_back(InputCommand::SpecialKey(key.to_string()));
            Ok(())
        } else {
            Err(anyhow::anyhow!("Session not found"))
        }
    }
    
    /// Resize session
    pub async fn resize_session(&self, session_name: &str, cols: u16, rows: u16) -> Result<()> {
        if let Some(session) = self.sessions.get(session_name) {
            session.write().await.dimensions = (cols, rows);
            let session_guard = session.read().await;
            let mut queue = session_guard.input_queue.lock().await;
            queue.queue.push_back(InputCommand::Resize(cols, rows));
            Ok(())
        } else {
            Err(anyhow::anyhow!("Session not found"))
        }
    }
    
    /// Get session statistics
    pub async fn get_session_stats(&self, session_name: &str) -> Option<SessionStats> {
        if let Some(session) = self.sessions.get(session_name) {
            let session_guard = session.read().await;
            let stats = session_guard.stats.lock().await;
            Some(SessionStats {
                total_captures: stats.total_captures,
                total_inputs: stats.total_inputs,
                bytes_captured: stats.bytes_captured,
                capture_errors: stats.capture_errors,
                input_errors: stats.input_errors,
            })
        } else {
            None
        }
    }
}

/// Capture loop that runs continuously for a session
async fn capture_loop(
    session_name: String,
    buffer: Arc<OptimizedTerminalBuffer>,
    clients: Arc<RwLock<Vec<ClientHandle>>>,
    stats: Arc<Mutex<SessionStats>>,
    capture_interval_ms: u64,
    semaphore: Arc<Semaphore>,
) {
    let mut ticker = interval(Duration::from_millis(capture_interval_ms));
    let mut last_content_hash = 0u64;
    let mut consecutive_errors = 0;
    
    loop {
        ticker.tick().await;
        
        // Check if we have any clients
        if clients.read().await.is_empty() {
            tokio::time::sleep(Duration::from_secs(1)).await;
            continue;
        }
        
        // Acquire semaphore permit to limit concurrent captures
        let _permit = match semaphore.try_acquire() {
            Ok(permit) => permit,
            Err(_) => {
                warn!("Too many concurrent captures, skipping");
                continue;
            }
        };
        
        // Capture pane content
        match tmux::capture_pane(&session_name).await {
            Ok(content) => {
                consecutive_errors = 0;
                
                // Calculate hash to detect changes
                let hash = xxhash_rust::xxh3::xxh3_64(content.as_bytes());
                
                if hash != last_content_hash {
                    last_content_hash = hash;
                    
                    // Write to buffer
                    if let Err(e) = buffer.write(content.as_bytes()).await {
                        error!("Failed to write to buffer: {}", e);
                        stats.lock().await.capture_errors += 1;
                        continue;
                    }
                    
                    // Update stats
                    let mut stats_guard = stats.lock().await;
                    stats_guard.total_captures += 1;
                    stats_guard.bytes_captured += content.len() as u64;
                    drop(stats_guard);
                    
                    // Create binary message
                    let mut message = BytesMut::with_capacity(content.len() + 5);
                    message.extend_from_slice(&[0x01]); // Output message type
                    message.extend_from_slice(&(content.len() as u32).to_le_bytes());
                    message.extend_from_slice(content.as_bytes());
                    let message = message.freeze();
                    
                    // Broadcast to all clients
                    let clients_list = clients.read().await;
                    let mut disconnected = Vec::new();
                    
                    for (i, client) in clients_list.iter().enumerate() {
                        if let Err(e) = client.tx.try_send(message.clone()) {
                            warn!("Failed to send to client {}: {}", client.id, e);
                            disconnected.push(i);
                        }
                    }
                    
                    drop(clients_list);
                    
                    // Remove disconnected clients
                    if !disconnected.is_empty() {
                        let mut clients_write = clients.write().await;
                        for i in disconnected.into_iter().rev() {
                            clients_write.remove(i);
                        }
                    }
                }
            }
            Err(e) => {
                consecutive_errors += 1;
                error!("Failed to capture pane (attempt {}): {}", consecutive_errors, e);
                stats.lock().await.capture_errors += 1;
                
                if consecutive_errors > 10 {
                    error!("Too many capture errors, stopping capture loop for session {}", session_name);
                    break;
                }
                
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
    }
}

/// Process batched input commands
async fn input_processor_loop(
    session_name: String,
    input_queue: Arc<Mutex<InputQueue>>,
    stats: Arc<Mutex<SessionStats>>,
    batch_timeout_ms: u64,
    max_batch_size: usize,
) {
    let mut ticker = interval(Duration::from_millis(batch_timeout_ms));
    
    loop {
        ticker.tick().await;
        
        let mut queue = input_queue.lock().await;
        
        if queue.queue.is_empty() {
            continue;
        }
        
        // Process up to max_batch_size commands
        let mut text_buffer = String::new();
        let mut commands_processed = 0;
        
        while commands_processed < max_batch_size && !queue.queue.is_empty() {
            if let Some(cmd) = queue.queue.pop_front() {
                match cmd {
                    InputCommand::Text(text) => {
                        text_buffer.push_str(&text);
                        commands_processed += 1;
                    }
                    InputCommand::SpecialKey(key) => {
                        // Flush text buffer first
                        if !text_buffer.is_empty() {
                            if let Err(e) = tmux::send_keys_to_session(&session_name, &text_buffer).await {
                                error!("Failed to send text: {}", e);
                                stats.lock().await.input_errors += 1;
                            }
                            text_buffer.clear();
                        }
                        
                        // Send special key
                        if let Err(e) = tmux::send_special_key(&session_name, &key).await {
                            error!("Failed to send special key: {}", e);
                            stats.lock().await.input_errors += 1;
                        }
                        commands_processed += 1;
                    }
                    InputCommand::Resize(cols, rows) => {
                        // Flush text buffer first
                        if !text_buffer.is_empty() {
                            if let Err(e) = tmux::send_keys_to_session(&session_name, &text_buffer).await {
                                error!("Failed to send text: {}", e);
                                stats.lock().await.input_errors += 1;
                            }
                            text_buffer.clear();
                        }
                        
                        // Resize window
                        let resize_cmd = format!("tmux resize-window -t {} -x {} -y {}", session_name, cols, rows);
                        if let Err(e) = tokio::process::Command::new("sh")
                            .arg("-c")
                            .arg(&resize_cmd)
                            .status()
                            .await
                        {
                            error!("Failed to resize: {}", e);
                            stats.lock().await.input_errors += 1;
                        }
                        commands_processed += 1;
                    }
                }
            }
        }
        
        // Send any remaining text
        if !text_buffer.is_empty() {
            if let Err(e) = tmux::send_keys_to_session(&session_name, &text_buffer).await {
                error!("Failed to send text: {}", e);
                stats.lock().await.input_errors += 1;
            } else {
                stats.lock().await.total_inputs += 1;
            }
        }
        
        queue.last_flush = Instant::now();
    }
}