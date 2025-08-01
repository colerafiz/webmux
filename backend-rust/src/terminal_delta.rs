use bytes::Bytes;
use dashmap::DashMap;
use serde::Serialize;
use std::sync::Arc;
use xxhash_rust::xxh3::xxh3_64;

use crate::types::ServerMessage;

const MAX_LINES: usize = 10000; // Maximum terminal history

#[derive(Clone)]
pub struct TerminalLine {
    pub content: Bytes,
    pub hash: u64,
}

#[derive(Clone)]
pub struct TerminalSnapshot {
    pub lines: Vec<TerminalLine>,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub viewport_top: usize,
    pub viewport_height: usize,
}

pub struct TerminalDeltaTracker {
    // Client ID -> Last snapshot
    client_snapshots: Arc<DashMap<String, TerminalSnapshot>>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LineDelta {
    pub line_number: u32,
    pub content: String,
    pub hash: u64,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalDelta {
    pub changes: Vec<LineDelta>,
    pub cursor_row: Option<usize>,
    pub cursor_col: Option<usize>,
    pub viewport_top: Option<usize>,
    pub clear_screen: bool,
}

impl TerminalDeltaTracker {
    pub fn new() -> Self {
        Self {
            client_snapshots: Arc::new(DashMap::new()),
        }
    }
    
    pub fn parse_terminal_output(&self, data: &str) -> TerminalSnapshot {
        let mut lines = Vec::new();
        let mut current_line = String::new();
        let mut cursor_row = 0;
        let mut cursor_col = 0;
        
        let mut chars = data.chars().peekable();
        
        while let Some(ch) = chars.next() {
            match ch {
                '\x1b' => {
                    // ANSI escape sequence
                    if chars.peek() == Some(&'[') {
                        chars.next(); // consume '['
                        let mut seq = String::new();
                        
                        while let Some(&next_ch) = chars.peek() {
                            if next_ch.is_ascii_alphabetic() {
                                let cmd = chars.next().unwrap();
                                match cmd {
                                    'H' | 'f' => {
                                        // Cursor position
                                        let parts: Vec<&str> = seq.split(';').collect();
                                        if parts.len() >= 2 {
                                            cursor_row = parts[0].parse::<usize>().unwrap_or(1).saturating_sub(1);
                                            cursor_col = parts[1].parse::<usize>().unwrap_or(1).saturating_sub(1);
                                        }
                                    }
                                    'J' => {
                                        // Clear screen
                                        if seq == "2" {
                                            lines.clear();
                                            current_line.clear();
                                            cursor_row = 0;
                                            cursor_col = 0;
                                        }
                                    }
                                    'K' => {
                                        // Clear line
                                        if seq.is_empty() || seq == "0" {
                                            current_line.truncate(cursor_col);
                                        }
                                    }
                                    _ => {}
                                }
                                break;
                            } else {
                                seq.push(chars.next().unwrap());
                            }
                        }
                    }
                }
                '\n' => {
                    // New line
                    let content = Bytes::from(current_line.clone());
                    let hash = xxh3_64(content.as_ref());
                    lines.push(TerminalLine { content, hash });
                    current_line.clear();
                    cursor_row += 1;
                    cursor_col = 0;
                }
                '\r' => {
                    // Carriage return
                    cursor_col = 0;
                }
                _ => {
                    // Regular character
                    if cursor_col >= current_line.len() {
                        current_line.push(ch);
                    } else {
                        current_line.replace_range(cursor_col..cursor_col + 1, &ch.to_string());
                    }
                    cursor_col += 1;
                }
            }
        }
        
        // Add remaining line if any
        if !current_line.is_empty() || cursor_row >= lines.len() {
            let content = Bytes::from(current_line);
            let hash = xxh3_64(content.as_ref());
            lines.push(TerminalLine { content, hash });
        }
        
        // Limit history
        if lines.len() > MAX_LINES {
            lines.drain(0..lines.len() - MAX_LINES);
        }
        
        TerminalSnapshot {
            lines,
            cursor_row,
            cursor_col,
            viewport_top: cursor_row.saturating_sub(24),
            viewport_height: 24,
        }
    }
    
    pub fn compute_delta(&self, client_id: &str, new_snapshot: &TerminalSnapshot) -> Option<TerminalDelta> {
        let mut delta = TerminalDelta {
            changes: Vec::new(),
            cursor_row: None,
            cursor_col: None,
            viewport_top: None,
            clear_screen: false,
        };
        
        // Get previous snapshot
        if let Some(old_snapshot) = self.client_snapshots.get(client_id) {
            // Check if screen was cleared
            if new_snapshot.lines.len() < old_snapshot.lines.len() / 2 {
                delta.clear_screen = true;
                
                // Send all new lines
                for (idx, line) in new_snapshot.lines.iter().enumerate() {
                    delta.changes.push(LineDelta {
                        line_number: idx as u32,
                        content: String::from_utf8_lossy(&line.content).to_string(),
                        hash: line.hash,
                    });
                }
            } else {
                // Compute line-by-line differences
                let old_lines = &old_snapshot.lines;
                let new_lines = &new_snapshot.lines;
                
                // Find changed lines by hash comparison
                for (idx, new_line) in new_lines.iter().enumerate() {
                    let changed = if idx < old_lines.len() {
                        old_lines[idx].hash != new_line.hash
                    } else {
                        true
                    };
                    
                    if changed {
                        delta.changes.push(LineDelta {
                            line_number: idx as u32,
                            content: String::from_utf8_lossy(&new_line.content).to_string(),
                            hash: new_line.hash,
                        });
                    }
                }
                
                // Remove extra lines if terminal shrank
                if old_lines.len() > new_lines.len() {
                    for idx in new_lines.len()..old_lines.len() {
                        delta.changes.push(LineDelta {
                            line_number: idx as u32,
                            content: String::new(),
                            hash: 0,
                        });
                    }
                }
            }
            
            // Update cursor if changed
            if old_snapshot.cursor_row != new_snapshot.cursor_row {
                delta.cursor_row = Some(new_snapshot.cursor_row);
            }
            if old_snapshot.cursor_col != new_snapshot.cursor_col {
                delta.cursor_col = Some(new_snapshot.cursor_col);
            }
            
            // Update viewport if changed
            if old_snapshot.viewport_top != new_snapshot.viewport_top {
                delta.viewport_top = Some(new_snapshot.viewport_top);
            }
        } else {
            // First snapshot - send everything
            delta.clear_screen = true;
            for (idx, line) in new_snapshot.lines.iter().enumerate() {
                delta.changes.push(LineDelta {
                    line_number: idx as u32,
                    content: String::from_utf8_lossy(&line.content).to_string(),
                    hash: line.hash,
                });
            }
            delta.cursor_row = Some(new_snapshot.cursor_row);
            delta.cursor_col = Some(new_snapshot.cursor_col);
            delta.viewport_top = Some(new_snapshot.viewport_top);
        }
        
        // Store new snapshot
        self.client_snapshots.insert(client_id.to_string(), new_snapshot.clone());
        
        // Only send delta if there are changes
        if delta.changes.is_empty() 
            && delta.cursor_row.is_none() 
            && delta.cursor_col.is_none() 
            && delta.viewport_top.is_none() 
            && !delta.clear_screen {
            None
        } else {
            Some(delta)
        }
    }
    
    pub fn remove_client(&self, client_id: &str) {
        self.client_snapshots.remove(client_id);
    }
}

// Extension to ServerMessage for delta updates
impl ServerMessage {
    pub fn delta(delta: TerminalDelta) -> Self {
        // We'll need to add a new variant to ServerMessage enum
        // For now, convert to regular output
        let mut output = String::new();
        
        if delta.clear_screen {
            output.push_str("\x1b[2J\x1b[H"); // Clear screen and home cursor
        }
        
        for change in &delta.changes {
            // Move cursor to line
            output.push_str(&format!("\x1b[{};1H", change.line_number + 1));
            // Clear line and write content
            output.push_str("\x1b[2K");
            output.push_str(&change.content);
        }
        
        if let (Some(row), Some(col)) = (delta.cursor_row, delta.cursor_col) {
            // Position cursor
            output.push_str(&format!("\x1b[{};{}H", row + 1, col + 1));
        }
        
        ServerMessage::Output { data: output }
    }
}