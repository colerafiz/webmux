use axum::{
    extract::Path,
    response::IntoResponse,
    Json,
};
use sysinfo::System;
use tracing::info;

use crate::{
    error::{AppError, Result},
    tmux,
    types::*,
};

pub async fn get_stats() -> impl IntoResponse {
    let mut sys = System::new_all();
    sys.refresh_all();

    let load_avg = System::load_average();
    let stats = SystemStats {
        cpu: CpuInfo {
            cores: sys.cpus().len(),
            model: sys.cpus().first().map(|c| c.brand().to_string()).unwrap_or_default(),
            usage: load_avg.one as f32,
            load_avg: [load_avg.one as f32, load_avg.five as f32, load_avg.fifteen as f32],
        },
        memory: MemoryInfo {
            total: sys.total_memory(),
            used: sys.used_memory(),
            free: sys.available_memory(),
            percent: format!("{:.1}", (sys.used_memory() as f64 / sys.total_memory() as f64) * 100.0),
        },
        uptime: System::uptime(),
        hostname: System::host_name().unwrap_or_default(),
        platform: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
    };

    Json(stats)
}

pub async fn list_sessions() -> Result<impl IntoResponse> {
    let sessions = tmux::list_sessions().await?;
    Ok(Json(SessionsResponse { sessions }))
}

pub async fn create_session(
    Json(payload): Json<CreateSessionRequest>,
) -> Result<impl IntoResponse> {
    let session_name = payload.name.unwrap_or_else(|| format!("session-{}", chrono::Utc::now().timestamp_millis()));
    info!("Creating session: {}", session_name);
    
    tmux::create_session(&session_name).await
        .map_err(|e| AppError::SessionError(format!("Failed to create session: {}", e)))?;
    
    info!("Successfully created session: {}", session_name);
    Ok(Json(CreateSessionResponse {
        success: true,
        session_name,
    }))
}

pub async fn kill_session(
    Path(name): Path<String>,
) -> Result<impl IntoResponse> {
    info!("Kill session request for: {}", name);
    
    tmux::kill_session(&name).await
        .map_err(|e| AppError::SessionError(format!("Failed to kill session: {}", e)))?;
    
    info!("Successfully killed session: {}", name);
    Ok(Json(SuccessResponse { success: true }))
}

pub async fn rename_session(
    Path(name): Path<String>,
    Json(payload): Json<RenameSessionRequest>,
) -> Result<impl IntoResponse> {
    if payload.new_name.trim().is_empty() {
        return Err(AppError::BadRequest("Session name cannot be empty".to_string()));
    }

    tmux::rename_session(&name, &payload.new_name).await
        .map_err(|e| AppError::SessionError(format!("Failed to rename session: {}", e)))?;

    Ok(Json(SuccessResponse { success: true }))
}

pub async fn list_windows(
    Path(session_name): Path<String>,
) -> Result<impl IntoResponse> {
    let windows = tmux::list_windows(&session_name).await
        .map_err(|e| AppError::NotFound(format!("Session not found: {}", e)))?;
    Ok(Json(WindowsResponse { windows }))
}

pub async fn create_window(
    Path(session_name): Path<String>,
    Json(payload): Json<CreateWindowRequest>,
) -> Result<impl IntoResponse> {
    tmux::create_window(&session_name, payload.window_name.as_deref()).await
        .map_err(|e| AppError::SessionError(format!("Failed to create window: {}", e)))?;
    Ok(Json(SuccessResponse { success: true }))
}

pub async fn kill_window(
    Path((session_name, window_index)): Path<(String, String)>,
) -> Result<impl IntoResponse> {
    tmux::kill_window(&session_name, &window_index).await
        .map_err(|e| AppError::SessionError(format!("Failed to kill window: {}", e)))?;
    Ok(Json(SuccessResponse { success: true }))
}

pub async fn rename_window(
    Path((session_name, window_index)): Path<(String, String)>,
    Json(payload): Json<RenameWindowRequest>,
) -> Result<impl IntoResponse> {
    if payload.new_name.trim().is_empty() {
        return Err(AppError::BadRequest("Window name cannot be empty".to_string()));
    }

    tmux::rename_window(&session_name, &window_index, &payload.new_name).await
        .map_err(|e| AppError::SessionError(format!("Failed to rename window: {}", e)))?;

    Ok(Json(SuccessResponse { success: true }))
}

pub async fn select_window(
    Path((session_name, window_index)): Path<(String, String)>,
) -> Result<impl IntoResponse> {
    tmux::select_window(&session_name, &window_index).await
        .map_err(|e| AppError::SessionError(format!("Failed to select window: {}", e)))?;
    Ok(Json(SuccessResponse { success: true }))
}