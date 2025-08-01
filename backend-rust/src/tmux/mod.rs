use anyhow::Result;
use chrono::{DateTime, Utc};
use std::process::Stdio;
use tokio::process::Command;
use tracing::{debug, error, info};

use crate::types::{TmuxSession, TmuxWindow};

fn escape_single_quotes(s: &str) -> String {
    s.replace('\'', "'\\''")
}

pub async fn ensure_tmux_server() -> Result<()> {
    // Check if tmux server is running
    let output = Command::new("tmux")
        .args(&["list-sessions"])
        .stderr(Stdio::null())
        .output()
        .await?;

    if !output.status.success() {
        // Start tmux server with a dummy session
        debug!("Starting TMUX server...");
        Command::new("tmux")
            .args(&["new-session", "-d", "-s", "__dummy__", "-c", "~", "exit"])
            .output()
            .await?;
        
        // Small delay to ensure server is fully started
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    Ok(())
}

pub async fn list_sessions() -> Result<Vec<TmuxSession>> {
    // Always use fallback for now - control mode needs more testing
    list_sessions_fallback().await
}

async fn list_sessions_fallback() -> Result<Vec<TmuxSession>> {
    // First ensure tmux server is running
    let check = Command::new("tmux")
        .args(&["list-sessions"])
        .stderr(Stdio::null())
        .output()
        .await?;

    if !check.status.success() {
        // TMUX not running, return empty list
        return Ok(vec![]);
    }

    let output = Command::new("tmux")
        .args(&[
            "list-sessions",
            "-F",
            "#{session_name}:#{session_attached}:#{session_created}:#{session_windows}:#{session_width}x#{session_height}",
        ])
        .output()
        .await?;

    if !output.status.success() {
        return Ok(vec![]);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let sessions: Vec<TmuxSession> = stdout
        .lines()
        .filter(|line| !line.is_empty())
        .filter_map(|line| {
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

    Ok(sessions)
}

pub async fn create_session(name: &str) -> Result<()> {
    ensure_tmux_server().await?;
    
    info!("Executing tmux new-session for: {}", name);
    let status = Command::new("tmux")
        .args(&["new-session", "-d", "-s", name])
        .env("HOME", std::env::var("HOME").unwrap_or_else(|_| "/".to_string()))
        .status()
        .await?;

    if !status.success() {
        error!("tmux new-session failed for: {}", name);
        anyhow::bail!("Failed to create session");
    }

    info!("tmux new-session succeeded for: {}", name);
    Ok(())
}

pub async fn kill_session(name: &str) -> Result<()> {
    info!("Executing tmux kill-session for: {}", name);
    
    // First try regular kill-session
    let status = Command::new("tmux")
        .args(&["kill-session", "-t", name])
        .status()
        .await?;

    if !status.success() {
        // If that fails, try with -C flag to kill all clients
        error!("tmux kill-session failed, trying with -C flag for: {}", name);
        let status2 = Command::new("tmux")
            .args(&["kill-session", "-C", "-t", name])
            .status()
            .await?;
            
        if !status2.success() {
            error!("tmux kill-session -C also failed for: {}", name);
            anyhow::bail!("Failed to kill session");
        }
    }

    info!("tmux kill-session succeeded for: {}", name);
    Ok(())
}

pub async fn rename_session(old_name: &str, new_name: &str) -> Result<()> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(format!(
            "tmux rename-session -t '{}' '{}'",
            escape_single_quotes(old_name),
            escape_single_quotes(new_name)
        ))
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to rename session: {}", stderr);
    }

    Ok(())
}

pub async fn list_windows(session_name: &str) -> Result<Vec<TmuxWindow>> {
    let output = Command::new("tmux")
        .args(&[
            "list-windows",
            "-t",
            session_name,
            "-F",
            "#{window_index}:#{window_name}:#{window_active}:#{window_panes}",
        ])
        .output()
        .await?;

    if !output.status.success() {
        anyhow::bail!("Session not found");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let windows: Vec<TmuxWindow> = stdout
        .lines()
        .filter(|line| !line.is_empty())
        .filter_map(|line| {
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

    Ok(windows)
}

pub async fn create_window(session_name: &str, window_name: Option<&str>) -> Result<()> {
    let mut args = vec!["new-window", "-a", "-t", session_name];
    if let Some(name) = window_name {
        args.push("-n");
        args.push(name);
    }

    let status = Command::new("tmux")
        .args(&args)
        .status()
        .await?;

    if !status.success() {
        anyhow::bail!("Failed to create window");
    }

    Ok(())
}

pub async fn kill_window(session_name: &str, window_index: &str) -> Result<()> {
    let target = format!("{}:{}", session_name, window_index);
    let status = Command::new("tmux")
        .args(&["kill-window", "-t", &target])
        .status()
        .await?;

    if !status.success() {
        anyhow::bail!("Failed to kill window");
    }

    Ok(())
}

pub async fn rename_window(session_name: &str, window_index: &str, new_name: &str) -> Result<()> {
    let target = format!("{}:{}", session_name, window_index);
    let output = Command::new("sh")
        .arg("-c")
        .arg(format!(
            "tmux rename-window -t '{}' '{}'",
            target,
            escape_single_quotes(new_name)
        ))
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to rename window: {}", stderr);
    }

    Ok(())
}

pub async fn select_window(session_name: &str, window_index: &str) -> Result<()> {
    let target = format!("{}:{}", session_name, window_index);
    let status = Command::new("tmux")
        .args(&["select-window", "-t", &target])
        .status()
        .await?;

    if !status.success() {
        anyhow::bail!("Failed to select window");
    }

    Ok(())
}