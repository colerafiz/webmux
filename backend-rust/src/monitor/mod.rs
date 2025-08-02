use std::{
    collections::HashMap,
    sync::Arc,
    time::Duration,
};
use tokio::{
    sync::{mpsc, RwLock},
    time::interval,
};
use tracing::{debug, error, info};

use crate::{
    tmux,
    types::{ServerMessage, TmuxSession},
};

#[derive(Debug, Clone, PartialEq)]
struct SessionState {
    sessions: Vec<TmuxSession>,
    // Map of session_name -> (window_count, pane_count)
    window_pane_counts: HashMap<String, (usize, usize)>,
}

pub struct TmuxMonitor {
    state: Arc<RwLock<SessionState>>,
    broadcast_tx: mpsc::UnboundedSender<ServerMessage>,
}

impl TmuxMonitor {
    pub fn new(broadcast_tx: mpsc::UnboundedSender<ServerMessage>) -> Self {
        Self {
            state: Arc::new(RwLock::new(SessionState {
                sessions: Vec::new(),
                window_pane_counts: HashMap::new(),
            })),
            broadcast_tx,
        }
    }

    pub async fn start(&self) {
        info!("Starting tmux monitor");
        
        // Initial state fetch
        self.check_for_changes().await;
        
        // Start monitoring loop
        let mut interval = interval(Duration::from_millis(500)); // Check every 500ms for better responsiveness
        
        loop {
            interval.tick().await;
            self.check_for_changes().await;
        }
    }

    async fn check_for_changes(&self) {
        // Get current tmux state
        let current_sessions = match tmux::list_sessions().await {
            Ok(sessions) => sessions,
            Err(e) => {
                error!("Failed to list tmux sessions: {}", e);
                return;
            }
        };

        // Get detailed window/pane counts for each session
        let mut current_window_pane_counts = HashMap::new();
        for session in &current_sessions {
            match tmux::list_windows(&session.name).await {
                Ok(windows) => {
                    let window_count = windows.len();
                    let pane_count: usize = windows.iter().map(|w| w.panes as usize).sum();
                    current_window_pane_counts.insert(session.name.clone(), (window_count, pane_count));
                }
                Err(e) => {
                    error!("Failed to list windows for session {}: {}", session.name, e);
                }
            }
        }

        // Check if state has changed
        let mut state = self.state.write().await;
        let sessions_changed = state.sessions != current_sessions;
        let window_pane_changed = state.window_pane_counts != current_window_pane_counts;

        if sessions_changed || window_pane_changed {
            debug!("Tmux state changed - sessions: {}, windows/panes: {}", 
                   sessions_changed, window_pane_changed);

            // Update state
            state.sessions = current_sessions.clone();
            state.window_pane_counts = current_window_pane_counts.clone();

            // Broadcast sessions list update
            let message = ServerMessage::SessionsList {
                sessions: current_sessions,
            };
            
            if let Err(e) = self.broadcast_tx.send(message) {
                error!("Failed to broadcast session update: {}", e);
            }

            // Don't broadcast window updates - let clients request them per session
            // This prevents sessions from getting mixed up when multiple clients
            // are viewing different sessions
        }
    }
}