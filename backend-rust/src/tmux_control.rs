use anyhow::{Result, bail};
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use std::{
    process::Stdio,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::{Child, ChildStdin, Command},
    sync::{oneshot, Mutex, RwLock},
    time::timeout,
};
use tracing::{error, info};

use crate::types::{TmuxSession, TmuxWindow};

// Cache TTL for session/window lists
const CACHE_TTL: Duration = Duration::from_millis(100);

#[derive(Clone)]
struct CachedSessions {
    sessions: Vec<TmuxSession>,
    timestamp: Instant,
}

#[derive(Clone)]
struct CachedWindows {
    windows: Vec<TmuxWindow>,
    timestamp: Instant,
}

pub struct TmuxControlMode {
    // Control mode process
    process: Arc<Mutex<Child>>,
    stdin: Arc<Mutex<ChildStdin>>,
    // Command ID counter
    cmd_id: Arc<Mutex<u64>>,
    // Pending command responses
    pending: Arc<DashMap<u64, oneshot::Sender<String>>>,
    // Reader task handle
    reader_task: Arc<tokio::task::JoinHandle<()>>,
    // Caches
    session_cache: Arc<RwLock<Option<CachedSessions>>>,
    window_cache: Arc<DashMap<String, CachedWindows>>,
}

impl TmuxControlMode {
    pub async fn new() -> Result<Arc<Self>> {
        // Start tmux in control mode
        let mut child = Command::new("tmux")
            .args(&["-C", "attach"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
        
        let stdin = child.stdin.take().ok_or_else(|| anyhow::anyhow!("Failed to get stdin"))?;
        let stdout = child.stdout.take().ok_or_else(|| anyhow::anyhow!("Failed to get stdout"))?;
        
        let pending: Arc<DashMap<u64, oneshot::Sender<String>>> = Arc::new(DashMap::new());
        let pending_clone = pending.clone();
        
        // Start reader task
        let reader_task = tokio::spawn(async move {
            let mut reader = BufReader::new(stdout);
            let mut line = String::new();
            
            loop {
                line.clear();
                match reader.read_line(&mut line).await {
                    Ok(0) => {
                        info!("Tmux control mode EOF");
                        break;
                    }
                    Ok(_) => {
                        let line = line.trim();
                        
                        // Parse control mode output
                        if line.starts_with("%output") {
                            // Command output: %output %<cmd_id> <data>
                            if let Some(cmd_id_str) = line.split_whitespace().nth(1) {
                                if let Some(cmd_id_str) = cmd_id_str.strip_prefix('%') {
                                    if let Ok(cmd_id) = cmd_id_str.parse::<u64>() {
                                        if let Some(data_start) = line.find(' ').and_then(|i| line[i+1..].find(' ')) {
                                            let data = &line[line.find(' ').unwrap() + 1 + data_start + 1..];
                                            
                                            if let Some((_, tx)) = pending_clone.remove(&cmd_id) {
                                                let _ = tx.send(data.to_string());
                                            }
                                        }
                                    }
                                }
                            }
                        } else if line.starts_with("%done") || line.starts_with("%error") {
                            // Command completion: %done %<cmd_id> or %error %<cmd_id>
                            if let Some(cmd_id_str) = line.split_whitespace().nth(1) {
                                if let Some(cmd_id_str) = cmd_id_str.strip_prefix('%') {
                                    if let Ok(cmd_id) = cmd_id_str.parse::<u64>() {
                                        if let Some((_, tx)) = pending_clone.remove(&cmd_id) {
                                            let _ = tx.send(String::new());
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error reading from tmux control mode: {}", e);
                        break;
                    }
                }
            }
        });
        
        let control = Arc::new(Self {
            process: Arc::new(Mutex::new(child)),
            stdin: Arc::new(Mutex::new(stdin)),
            cmd_id: Arc::new(Mutex::new(0)),
            pending,
            reader_task: Arc::new(reader_task),
            session_cache: Arc::new(RwLock::new(None)),
            window_cache: Arc::new(DashMap::new()),
        });
        
        Ok(control)
    }
    
    async fn send_command(&self, cmd: &str) -> Result<String> {
        let cmd_id = {
            let mut id = self.cmd_id.lock().await;
            *id += 1;
            *id
        };
        
        let (tx, rx) = oneshot::channel();
        self.pending.insert(cmd_id, tx);
        
        // Send command with ID
        let mut stdin = self.stdin.lock().await;
        stdin.write_all(format!("{} %{}\n", cmd, cmd_id).as_bytes()).await?;
        stdin.flush().await?;
        
        // Wait for response with timeout
        match timeout(Duration::from_secs(5), rx).await {
            Ok(Ok(response)) => Ok(response),
            Ok(Err(_)) => bail!("Command response channel closed"),
            Err(_) => {
                self.pending.remove(&cmd_id);
                bail!("Command timeout")
            }
        }
    }
    
    pub async fn list_sessions(&self) -> Result<Vec<TmuxSession>> {
        // Check cache first
        {
            let cache = self.session_cache.read().await;
            if let Some(cached) = cache.as_ref() {
                if cached.timestamp.elapsed() < CACHE_TTL {
                    return Ok(cached.sessions.clone());
                }
            }
        }
        
        // Send list-sessions command
        let response = self.send_command(
            "list-sessions -F '#{session_name}:#{session_attached}:#{session_created}:#{session_windows}:#{session_width}x#{session_height}'"
        ).await?;
        
        let sessions: Vec<TmuxSession> = response
            .lines()
            .filter(|line| !line.is_empty())
            .filter_map(|line| {
                let line = line.trim_matches('\'');
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 5 {
                    let created_timestamp = parts[2].parse::<i64>().ok()?;
                    Some(TmuxSession {
                        name: parts[0].to_string(),
                        attached: parts[1] == "1",
                        created: DateTime::from_timestamp(created_timestamp, 0)
                            .unwrap_or_else(|| Utc::now()),
                        windows: parts[3].parse().unwrap_or(0),
                        dimensions: parts[4].to_string(),
                    })
                } else {
                    None
                }
            })
            .collect();
        
        // Update cache
        {
            let mut cache = self.session_cache.write().await;
            *cache = Some(CachedSessions {
                sessions: sessions.clone(),
                timestamp: Instant::now(),
            });
        }
        
        Ok(sessions)
    }
    
    pub async fn list_windows(&self, session_name: &str) -> Result<Vec<TmuxWindow>> {
        // Check cache first
        if let Some(cached) = self.window_cache.get(session_name) {
            if cached.timestamp.elapsed() < CACHE_TTL {
                return Ok(cached.windows.clone());
            }
        }
        
        // Send list-windows command
        let response = self.send_command(&format!(
            "list-windows -t {} -F '#{{window_index}}:#{{window_name}}:#{{window_active}}:#{{window_panes}}'",
            session_name
        )).await?;
        
        let windows: Vec<TmuxWindow> = response
            .lines()
            .filter(|line| !line.is_empty())
            .filter_map(|line| {
                let line = line.trim_matches('\'');
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 4 {
                    Some(TmuxWindow {
                        index: parts[0].parse().ok()?,
                        name: parts[1].to_string(),
                        active: parts[2] == "1",
                        panes: parts[3].parse().unwrap_or(1),
                    })
                } else {
                    None
                }
            })
            .collect();
        
        // Update cache
        self.window_cache.insert(
            session_name.to_string(),
            CachedWindows {
                windows: windows.clone(),
                timestamp: Instant::now(),
            },
        );
        
        Ok(windows)
    }
    
    pub async fn create_session(&self, name: &str) -> Result<()> {
        self.send_command(&format!("new-session -d -s {}", name)).await?;
        self.invalidate_session_cache().await;
        Ok(())
    }
    
    pub async fn kill_session(&self, name: &str) -> Result<()> {
        self.send_command(&format!("kill-session -t {}", name)).await?;
        self.invalidate_session_cache().await;
        self.window_cache.remove(name);
        Ok(())
    }
    
    pub async fn rename_session(&self, old_name: &str, new_name: &str) -> Result<()> {
        self.send_command(&format!("rename-session -t {} {}", old_name, new_name)).await?;
        self.invalidate_session_cache().await;
        
        // Move window cache entry
        if let Some((_, cached)) = self.window_cache.remove(old_name) {
            self.window_cache.insert(new_name.to_string(), cached);
        }
        
        Ok(())
    }
    
    pub async fn create_window(&self, session_name: &str, window_name: Option<&str>) -> Result<()> {
        let cmd = if let Some(name) = window_name {
            format!("new-window -a -t {} -n {}", session_name, name)
        } else {
            format!("new-window -a -t {}", session_name)
        };
        
        self.send_command(&cmd).await?;
        self.window_cache.remove(session_name);
        Ok(())
    }
    
    pub async fn kill_window(&self, session_name: &str, window_index: &str) -> Result<()> {
        self.send_command(&format!("kill-window -t {}:{}", session_name, window_index)).await?;
        self.window_cache.remove(session_name);
        Ok(())
    }
    
    pub async fn rename_window(&self, session_name: &str, window_index: &str, new_name: &str) -> Result<()> {
        self.send_command(&format!(
            "rename-window -t {}:{} {}",
            session_name, window_index, new_name
        )).await?;
        self.window_cache.remove(session_name);
        Ok(())
    }
    
    pub async fn select_window(&self, session_name: &str, window_index: &str) -> Result<()> {
        self.send_command(&format!("select-window -t {}:{}", session_name, window_index)).await?;
        Ok(())
    }
    
    async fn invalidate_session_cache(&self) {
        let mut cache = self.session_cache.write().await;
        *cache = None;
    }
}

impl Drop for TmuxControlMode {
    fn drop(&mut self) {
        // The child process will be killed when dropped
        self.reader_task.abort();
    }
}

// Global instance
lazy_static::lazy_static! {
    static ref TMUX_CONTROL: Arc<RwLock<Option<Arc<TmuxControlMode>>>> = Arc::new(RwLock::new(None));
}

pub async fn get_control_mode() -> Result<Arc<TmuxControlMode>> {
    let mut control_lock = TMUX_CONTROL.write().await;
    
    if let Some(control) = control_lock.as_ref() {
        // Check if process is still alive
        if let Ok(mut proc) = control.process.try_lock() {
            if proc.try_wait()?.is_none() {
                return Ok(control.clone());
            }
        }
    }
    
    // Create new control mode instance
    info!("Creating new tmux control mode connection");
    let control = TmuxControlMode::new().await?;
    *control_lock = Some(control.clone());
    Ok(control)
}