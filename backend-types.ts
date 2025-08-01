// Backend type definitions

// WebSocket message types
export interface BaseWebSocketMessage {
  type: string;
}

export interface ListSessionsMessage extends BaseWebSocketMessage {
  type: 'list-sessions';
}

export interface AttachSessionMessage extends BaseWebSocketMessage {
  type: 'attach-session';
  sessionName: string;
  cols: number;
  rows: number;
}

export interface InputMessage extends BaseWebSocketMessage {
  type: 'input';
  data: string;
}

export interface ResizeMessage extends BaseWebSocketMessage {
  type: 'resize';
  cols: number;
  rows: number;
}

export interface ListWindowsMessage extends BaseWebSocketMessage {
  type: 'list-windows';
  sessionName: string;
}

export interface SelectWindowMessage extends BaseWebSocketMessage {
  type: 'select-window';
  sessionName: string;
  windowIndex: number;
}

export interface PingMessage extends BaseWebSocketMessage {
  type: 'ping';
}

export interface AudioControlMessage extends BaseWebSocketMessage {
  type: 'audio-control';
  action: 'start' | 'stop';
}

export interface AudioStreamMessage extends BaseWebSocketMessage {
  type: 'audio-stream';
  data: ArrayBuffer;
}

export interface AudioStatusMessage extends BaseWebSocketMessage {
  type: 'audio-status';
  streaming: boolean;
  error?: string;
}

export type WebSocketMessage = 
  | ListSessionsMessage
  | AttachSessionMessage
  | InputMessage
  | ResizeMessage
  | ListWindowsMessage
  | SelectWindowMessage
  | PingMessage
  | AudioControlMessage
  | AudioStreamMessage
  | AudioStatusMessage;

// TMUX types
export interface TmuxSession {
  name: string;
  attached: boolean;
  created: Date;
  windows: number;
  dimensions: string;
}

export interface TmuxWindow {
  index: number;
  name: string;
  active: boolean;
  panes: number;
}

// API request/response types
export interface CreateSessionRequest {
  name?: string;
}

export interface RenameSessionRequest {
  newName: string;
}

export interface CreateWindowRequest {
  windowName?: string;
}

export interface RenameWindowRequest {
  newName: string;
}

export interface SystemStats {
  cpu: {
    cores: number;
    model: string;
    usage: number;
    loadAvg: [number, number, number];
  };
  memory: {
    total: number;
    used: number;
    free: number;
    percent: string;
  };
  uptime: number;
  hostname: string;
  platform: string;
  arch: string;
}