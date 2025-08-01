use anyhow::Result;
use std::{
    collections::HashMap,
    sync::Arc,
    time::Duration,
};
use tokio::{
    sync::{Mutex, RwLock},
    time::interval,
    process::Command,
};
use tracing::{debug, error, info};

/// Manages tmux sessions without attaching directly
/// This avoids conflicts with interactive applications
pub struct TmuxSessionManager {
    /// Currently active session for this client
    active_session: Arc<RwLock<Option<String>>>,
    /// Output capture tasks
    capture_tasks: Arc<Mutex<HashMap<String, tokio::task::JoinHandle<()>>>>,
}

impl TmuxSessionManager {
    pub fn new() -> Self {
        Self {
            active_session: Arc::new(RwLock::new(None)),
            capture_tasks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Switch to a different tmux session without attaching
    pub async fn switch_session(&self, session_name: &str) -> Result<()> {
        info!("Switching to session: {}", session_name);
        
        // Verify session exists
        let output = Command::new("tmux")
            .args(&["has-session", "-t", session_name])
            .output()
            .await?;
        
        if !output.status.success() {
            return Err(anyhow::anyhow!("Session {} does not exist", session_name));
        }
        
        // Update active session
        let mut active = self.active_session.write().await;
        *active = Some(session_name.to_string());
        
        Ok(())
    }

    /// Send input to the active session using send-keys
    pub async fn send_input(&self, data: &str) -> Result<()> {
        let active = self.active_session.read().await;
        if let Some(session) = active.as_ref() {
            // Use send-keys to send input to the session
            // The -l flag sends the keys literally (doesn't interpret them)
            let status = Command::new("tmux")
                .args(&["send-keys", "-t", session, "-l", data])
                .status()
                .await?;
            
            if !status.success() {
                return Err(anyhow::anyhow!("Failed to send input to session"));
            }
        }
        Ok(())
    }

    /// Send special keys (like Enter, Escape, etc)
    pub async fn send_special_key(&self, key: &str) -> Result<()> {
        let active = self.active_session.read().await;
        if let Some(session) = active.as_ref() {
            let status = Command::new("tmux")
                .args(&["send-keys", "-t", session, key])
                .status()
                .await?;
            
            if !status.success() {
                return Err(anyhow::anyhow!("Failed to send special key"));
            }
        }
        Ok(())
    }

    /// Capture the current pane content
    pub async fn capture_pane(&self) -> Result<String> {
        let active = self.active_session.read().await;
        if let Some(session) = active.as_ref() {
            let output = Command::new("tmux")
                .args(&[
                    "capture-pane",
                    "-t", session,
                    "-p",  // Print to stdout
                    "-e",  // Include escape sequences
                    "-S", "-",  // Start from beginning of history
                    "-E", "-",  // End at bottom
                ])
                .output()
                .await?;
            
            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                Err(anyhow::anyhow!("Failed to capture pane"))
            }
        } else {
            Ok(String::new())
        }
    }

    /// Start continuous capture for a session
    pub async fn start_capture_stream(
        &self,
        session_name: String,
        tx: tokio::sync::mpsc::UnboundedSender<String>,
    ) -> Result<()> {
        let mut tasks = self.capture_tasks.lock().await;
        
        // Stop any existing capture for this session
        if let Some(task) = tasks.remove(&session_name) {
            task.abort();
        }
        
        let session = session_name.clone();
        let task = tokio::spawn(async move {
            let mut ticker = interval(Duration::from_millis(100)); // Capture every 100ms
            let mut last_content = String::new();
            
            loop {
                ticker.tick().await;
                
                // Capture current content
                let output = Command::new("tmux")
                    .args(&[
                        "capture-pane",
                        "-t", &session,
                        "-p",
                        "-e",
                        "-J",  // Join wrapped lines
                    ])
                    .output()
                    .await;
                
                if let Ok(output) = output {
                    if output.status.success() {
                        let content = String::from_utf8_lossy(&output.stdout).to_string();
                        
                        // Only send if content changed
                        if content != last_content {
                            if tx.send(content.clone()).is_err() {
                                break; // Client disconnected
                            }
                            last_content = content;
                        }
                    }
                }
            }
        });
        
        tasks.insert(session_name, task);
        Ok(())
    }

    /// Select a specific window in the active session
    pub async fn select_window(&self, window_index: u32) -> Result<()> {
        let active = self.active_session.read().await;
        if let Some(session) = active.as_ref() {
            let target = format!("{}:{}", session, window_index);
            let status = Command::new("tmux")
                .args(&["select-window", "-t", &target])
                .status()
                .await?;
            
            if !status.success() {
                return Err(anyhow::anyhow!("Failed to select window"));
            }
        }
        Ok(())
    }

    /// Clean up resources
    pub async fn cleanup(&self) {
        let mut tasks = self.capture_tasks.lock().await;
        for (_, task) in tasks.drain() {
            task.abort();
        }
    }
}

impl Drop for TmuxSessionManager {
    fn drop(&mut self) {
        // Cleanup is handled by the async cleanup method
    }
}