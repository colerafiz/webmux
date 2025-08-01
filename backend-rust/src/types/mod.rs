use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TmuxSession {
    pub name: String,
    pub attached: bool,
    pub created: DateTime<Utc>,
    pub windows: u32,
    pub dimensions: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TmuxWindow {
    pub index: u32,
    pub name: String,
    pub active: bool,
    pub panes: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSessionRequest {
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameSessionRequest {
    pub new_name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateWindowRequest {
    pub window_name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameWindowRequest {
    pub new_name: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemStats {
    pub cpu: CpuInfo,
    pub memory: MemoryInfo,
    pub uptime: u64,
    pub hostname: String,
    pub platform: String,
    pub arch: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CpuInfo {
    pub cores: usize,
    pub model: String,
    pub usage: f32,
    pub load_avg: [f32; 3],
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryInfo {
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub percent: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum WebSocketMessage {
    ListSessions,
    AttachSession {
        #[serde(rename = "sessionName")]
        session_name: String,
        cols: u16,
        rows: u16,
    },
    Input {
        data: String,
    },
    Resize {
        cols: u16,
        rows: u16,
    },
    ListWindows {
        #[serde(rename = "sessionName")]
        session_name: String,
    },
    SelectWindow {
        #[serde(rename = "sessionName")]
        session_name: String,
        #[serde(rename = "windowIndex")]
        window_index: u32,
    },
    Ping,
    AudioControl {
        action: AudioAction,
    },
    // Session management
    CreateSession {
        name: Option<String>,
    },
    KillSession {
        #[serde(rename = "sessionName")]
        session_name: String,
    },
    RenameSession {
        #[serde(rename = "sessionName")]
        session_name: String,
        #[serde(rename = "newName")]
        new_name: String,
    },
    // Window management
    CreateWindow {
        #[serde(rename = "sessionName")]
        session_name: String,
        #[serde(rename = "windowName")]
        window_name: Option<String>,
    },
    KillWindow {
        #[serde(rename = "sessionName")]
        session_name: String,
        #[serde(rename = "windowIndex")]
        window_index: String,
    },
    RenameWindow {
        #[serde(rename = "sessionName")]
        session_name: String,
        #[serde(rename = "windowIndex")]
        window_index: String,
        #[serde(rename = "newName")]
        new_name: String,
    },
    // System stats
    GetStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AudioAction {
    Start,
    Stop,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum ServerMessage {
    SessionsList {
        sessions: Vec<TmuxSession>,
    },
    Attached {
        #[serde(rename = "sessionName")]
        session_name: String,
    },
    Output {
        data: String,
    },
    Disconnected,
    WindowsList {
        #[serde(rename = "sessionName")]
        session_name: String,
        windows: Vec<TmuxWindow>,
    },
    WindowSelected {
        success: bool,
        #[serde(rename = "windowIndex", skip_serializing_if = "Option::is_none")]
        window_index: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
    },
    Pong,
    AudioStatus {
        streaming: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
    },
    AudioStream {
        data: String, // base64 encoded audio data
    },
    // Session management responses
    SessionCreated {
        success: bool,
        #[serde(rename = "sessionName", skip_serializing_if = "Option::is_none")]
        session_name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
    },
    SessionKilled {
        success: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
    },
    SessionRenamed {
        success: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
    },
    // Window management responses
    WindowCreated {
        success: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
    },
    WindowKilled {
        success: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
    },
    WindowRenamed {
        success: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
    },
    // System stats response
    Stats {
        stats: SystemStats,
    },
    // Generic error response
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct ApiResponse<T> {
    #[serde(flatten)]
    pub data: T,
}

#[derive(Debug, Clone, Serialize)]
pub struct SuccessResponse {
    pub success: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub error: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionsResponse {
    pub sessions: Vec<TmuxSession>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WindowsResponse {
    pub windows: Vec<TmuxWindow>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateSessionResponse {
    pub success: bool,
    #[serde(rename = "sessionName")]
    pub session_name: String,
}