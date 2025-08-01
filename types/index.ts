/**
 * WebMux TypeScript Type Definitions
 * Shared types for both backend and frontend
 */

// ==================== TMUX Types ====================

/**
 * Represents a TMUX session
 */
export interface TmuxSession {
  name: string;
  attached: boolean;
  created: Date | string;
  windows: number;
  dimensions: string; // Format: "80x24"
}

/**
 * Represents a TMUX window
 */
export interface TmuxWindow {
  index: number;
  name: string;
  active: boolean;
  layout: string;
  panes: number;
}

/**
 * Represents a TMUX pane (future use)
 */
export interface TmuxPane {
  index: number;
  active: boolean;
  width: number;
  height: number;
  command?: string;
}

// ==================== API Types ====================

/**
 * API Response wrapper
 */
export interface ApiResponse<T = any> {
  success: boolean;
  data?: T;
  error?: string;
  message?: string;
}

/**
 * Session list response
 */
export interface SessionsResponse {
  sessions: TmuxSession[];
}

/**
 * Window list response
 */
export interface WindowsResponse {
  windows: TmuxWindow[];
}

/**
 * Create session request
 */
export interface CreateSessionRequest {
  name: string;
}

/**
 * Rename session request
 */
export interface RenameSessionRequest {
  newName: string;
}

/**
 * Create window request
 */
export interface CreateWindowRequest {
  windowName?: string;
}

/**
 * Rename window request
 */
export interface RenameWindowRequest {
  newName: string;
}

/**
 * System statistics response
 */
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

// ==================== WebSocket Message Types ====================

/**
 * Base WebSocket message
 */
export interface BaseMessage {
  type: string;
}

// ----- Client to Server Messages -----

/**
 * Attach to a TMUX session
 */
export interface AttachSessionMessage extends BaseMessage {
  type: 'attach-session';
  sessionName: string;
  cols: number;
  rows: number;
}

/**
 * Send terminal input
 */
export interface InputMessage extends BaseMessage {
  type: 'input';
  data: string;
}

/**
 * Resize terminal
 */
export interface ResizeMessage extends BaseMessage {
  type: 'resize';
  cols: number;
  rows: number;
}

/**
 * List windows in a session
 */
export interface ListWindowsMessage extends BaseMessage {
  type: 'list-windows';
  sessionName: string;
}

/**
 * Select a window in a session
 */
export interface SelectWindowMessage extends BaseMessage {
  type: 'select-window';
  sessionName: string;
  windowIndex: number;
}

/**
 * Ping message to keep connection alive
 */
export interface PingMessage extends BaseMessage {
  type: 'ping';
}

/**
 * Union type for all client messages
 */
export type ClientMessage =
  | AttachSessionMessage
  | InputMessage
  | ResizeMessage
  | ListWindowsMessage
  | SelectWindowMessage
  | PingMessage;

// ----- Server to Client Messages -----

/**
 * Terminal output data
 */
export interface OutputMessage extends BaseMessage {
  type: 'output';
  data: string;
}

/**
 * Session attached confirmation
 */
export interface AttachedMessage extends BaseMessage {
  type: 'attached';
  sessionName: string;
}

/**
 * Session disconnected notification
 */
export interface DisconnectedMessage extends BaseMessage {
  type: 'disconnected';
  reason?: string;
}

/**
 * Windows list response
 */
export interface WindowsListMessage extends BaseMessage {
  type: 'windows-list';
  windows: TmuxWindow[];
}

/**
 * Error message
 */
export interface ErrorMessage extends BaseMessage {
  type: 'error';
  error: string;
  code?: string;
}

/**
 * Pong response to ping
 */
export interface PongMessage extends BaseMessage {
  type: 'pong';
}

/**
 * Union type for all server messages
 */
export type ServerMessage =
  | OutputMessage
  | AttachedMessage
  | DisconnectedMessage
  | WindowsListMessage
  | ErrorMessage
  | PongMessage;

// ==================== Terminal Configuration Types ====================

/**
 * Terminal options for xterm.js
 */
export interface TerminalOptions {
  cols?: number;
  rows?: number;
  fontSize?: number;
  fontFamily?: string;
  theme?: TerminalTheme;
  cursorBlink?: boolean;
  cursorStyle?: 'block' | 'underline' | 'bar';
  scrollback?: number;
  tabStopWidth?: number;
  bellStyle?: 'none' | 'sound' | 'visual' | 'both';
}

/**
 * Terminal theme configuration
 */
export interface TerminalTheme {
  foreground?: string;
  background?: string;
  cursor?: string;
  cursorAccent?: string;
  selection?: string;
  black?: string;
  red?: string;
  green?: string;
  yellow?: string;
  blue?: string;
  magenta?: string;
  cyan?: string;
  white?: string;
  brightBlack?: string;
  brightRed?: string;
  brightGreen?: string;
  brightYellow?: string;
  brightBlue?: string;
  brightMagenta?: string;
  brightCyan?: string;
  brightWhite?: string;
}

// ==================== WebSocket Connection Types ====================

/**
 * WebSocket connection state
 */
export enum ConnectionState {
  DISCONNECTED = 'disconnected',
  CONNECTING = 'connecting',
  CONNECTED = 'connected',
  RECONNECTING = 'reconnecting',
  ERROR = 'error'
}

/**
 * WebSocket manager configuration
 */
export interface WebSocketConfig {
  url?: string;
  reconnect?: boolean;
  reconnectInterval?: number;
  maxReconnectAttempts?: number;
  pingInterval?: number;
}

// ==================== Component Props Types ====================

/**
 * Session item component props
 */
export interface SessionItemProps {
  session: TmuxSession;
  isActive?: boolean;
  onSelect?: (session: TmuxSession) => void;
  onKill?: (sessionName: string) => void;
  onRename?: (sessionName: string, newName: string) => void;
}

/**
 * Window item component props
 */
export interface WindowItemProps {
  window: TmuxWindow;
  sessionName: string;
  isActive?: boolean;
  onSelect?: (window: TmuxWindow) => void;
  onKill?: (windowIndex: number) => void;
  onRename?: (windowIndex: number, newName: string) => void;
}

/**
 * Terminal view component props
 */
export interface TerminalViewProps {
  sessionName?: string;
  options?: TerminalOptions;
  onReady?: () => void;
  onDisconnect?: () => void;
}

// ==================== Utility Types ====================

/**
 * Terminal dimensions
 */
export interface TerminalDimensions {
  cols: number;
  rows: number;
}

/**
 * Session creation options
 */
export interface SessionOptions {
  name: string;
  command?: string;
  workingDirectory?: string;
  dimensions?: TerminalDimensions;
}

/**
 * Window creation options
 */
export interface WindowOptions {
  name?: string;
  command?: string;
  workingDirectory?: string;
}

// ==================== Event Types ====================

/**
 * Terminal event types
 */
export interface TerminalEvents {
  data: (data: string) => void;
  resize: (dimensions: TerminalDimensions) => void;
  title: (title: string) => void;
  bell: () => void;
}

/**
 * Session event types
 */
export interface SessionEvents {
  created: (session: TmuxSession) => void;
  destroyed: (sessionName: string) => void;
  renamed: (oldName: string, newName: string) => void;
  attached: (sessionName: string) => void;
  detached: (sessionName: string) => void;
}

/**
 * Window event types
 */
export interface WindowEvents {
  created: (window: TmuxWindow) => void;
  destroyed: (windowIndex: number) => void;
  renamed: (windowIndex: number, newName: string) => void;
  selected: (windowIndex: number) => void;
}

// ==================== Type Guards ====================

/**
 * Type guard for client messages
 */
export function isClientMessage(message: any): message is ClientMessage {
  return message && typeof message.type === 'string' && [
    'attach-session',
    'input',
    'resize',
    'list-windows',
    'select-window',
    'ping'
  ].includes(message.type);
}

/**
 * Type guard for server messages
 */
export function isServerMessage(message: any): message is ServerMessage {
  return message && typeof message.type === 'string' && [
    'output',
    'attached',
    'disconnected',
    'windows-list',
    'error',
    'pong'
  ].includes(message.type);
}

/**
 * Type guard for specific message types
 */
export function isMessageType<T extends BaseMessage>(
  message: any,
  type: T['type']
): message is T {
  return message && message.type === type;
}