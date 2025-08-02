// WebSocket message types
export interface WsMessage {
  type: string;
  [key: string]: string | number | boolean | object | undefined;
}

export interface AttachSessionMessage extends WsMessage {
  type: 'attach-session';
  sessionName: string;
  cols: number;
  rows: number;
}

export interface InputMessage extends WsMessage {
  type: 'input';
  data: string;
}

export interface ResizeMessage extends WsMessage {
  type: 'resize';
  cols: number;
  rows: number;
}

export interface ListWindowsMessage extends WsMessage {
  type: 'list-windows';
  sessionName: string;
}

export interface SelectWindowMessage extends WsMessage {
  type: 'select-window';
  sessionName: string;
  windowIndex: number;
}

// Server response types
export interface OutputMessage extends WsMessage {
  type: 'output';
  data: string;
}

export interface AttachedMessage extends WsMessage {
  type: 'attached';
  sessionName: string;
}

export interface DisconnectedMessage extends WsMessage {
  type: 'disconnected';
}

export interface WindowsListMessage extends WsMessage {
  type: 'windows-list';
  sessionName: string;
  windows: TmuxWindow[];
}

// TMUX types
export interface TmuxSession {
  name: string;
  windows: number;
  created: string;
  attached: boolean;
  dimensions?: string;
}

export interface TmuxWindow {
  index: number;
  name: string;
  active: boolean;
  panes: number;
}

// API response types
export interface ApiResponse<T = unknown> {
  success: boolean;
  data?: T;
  error?: string;
}

// Terminal types
export interface TerminalSize {
  cols: number;
  rows: number;
}

// System stats
export interface SystemStats {
  hostname: string;
  platform: string;
  arch: string;
  uptime: number;
  cpu: {
    model: string;
    cores: number;
    usage: number;
    loadAvg: number[];
  };
  memory: {
    total: number;
    used: number;
    free: number;
    percent: string;
  };
}

// Additional WebSocket messages
export interface SessionsListMessage extends WsMessage {
  type: 'sessions-list';
  sessions: TmuxSession[];
}

export interface WindowSelectedMessage extends WsMessage {
  type: 'window-selected';
  sessionName: string;
  windowIndex: number;
}

// API-specific response types
export interface SessionCreateResponse {
  success: boolean;
  sessionName: string;
  message?: string;
}
export interface SessionActionResponse {
  success: boolean;
  message?: string;
}
export interface WindowsListResponse {
  windows: TmuxWindow[];
}
export interface WindowCreateResponse {
  success: boolean;
  window?: TmuxWindow;
  message?: string;
}