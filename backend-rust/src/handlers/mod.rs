use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sysinfo::System;
use tracing::error;

use crate::{
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

pub async fn list_sessions() -> Result<impl IntoResponse, StatusCode> {
    match tmux::list_sessions().await {
        Ok(sessions) => Ok(Json(SessionsResponse { sessions })),
        Err(e) => {
            error!("Failed to list sessions: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn create_session(
    Json(payload): Json<CreateSessionRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let session_name = payload.name.unwrap_or_else(|| format!("session-{}", chrono::Utc::now().timestamp_millis()));
    
    match tmux::create_session(&session_name).await {
        Ok(_) => Ok(Json(CreateSessionResponse {
            success: true,
            session_name,
        })),
        Err(e) => {
            error!("Failed to create session: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

pub async fn kill_session(
    Path(name): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    match tmux::kill_session(&name).await {
        Ok(_) => Ok(Json(SuccessResponse { success: true })),
        Err(e) => {
            error!("Failed to kill session: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

pub async fn rename_session(
    Path(name): Path<String>,
    Json(payload): Json<RenameSessionRequest>,
) -> impl IntoResponse {
    if payload.new_name.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                success: false,
                error: "Session name cannot be empty".to_string(),
            }),
        );
    }

    match tmux::rename_session(&name, &payload.new_name).await {
        Ok(_) => (
            StatusCode::OK,
            Json(ErrorResponse {
                success: true,
                error: String::new(),
            }),
        ),
        Err(e) => {
            error!("Failed to rename session: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    success: false,
                    error: e.to_string(),
                }),
            )
        }
    }
}

pub async fn list_windows(
    Path(session_name): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    match tmux::list_windows(&session_name).await {
        Ok(windows) => Ok(Json(WindowsResponse { windows })),
        Err(e) => {
            error!("Failed to list windows: {}", e);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

pub async fn create_window(
    Path(session_name): Path<String>,
    Json(payload): Json<CreateWindowRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    match tmux::create_window(&session_name, payload.window_name.as_deref()).await {
        Ok(_) => Ok(Json(SuccessResponse { success: true })),
        Err(e) => {
            error!("Failed to create window: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

pub async fn kill_window(
    Path((session_name, window_index)): Path<(String, String)>,
) -> Result<impl IntoResponse, StatusCode> {
    match tmux::kill_window(&session_name, &window_index).await {
        Ok(_) => Ok(Json(SuccessResponse { success: true })),
        Err(e) => {
            error!("Failed to kill window: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

pub async fn rename_window(
    Path((session_name, window_index)): Path<(String, String)>,
    Json(payload): Json<RenameWindowRequest>,
) -> impl IntoResponse {
    if payload.new_name.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                success: false,
                error: "Window name cannot be empty".to_string(),
            }),
        );
    }

    match tmux::rename_window(&session_name, &window_index, &payload.new_name).await {
        Ok(_) => (
            StatusCode::OK,
            Json(ErrorResponse {
                success: true,
                error: String::new(),
            }),
        ),
        Err(e) => {
            error!("Failed to rename window: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    success: false,
                    error: e.to_string(),
                }),
            )
        }
    }
}

pub async fn select_window(
    Path((session_name, window_index)): Path<(String, String)>,
) -> Result<impl IntoResponse, StatusCode> {
    match tmux::select_window(&session_name, &window_index).await {
        Ok(_) => Ok(Json(SuccessResponse { success: true })),
        Err(e) => {
            error!("Failed to select window: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}