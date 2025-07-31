/**
 * WebMux Backend-Specific TypeScript Type Definitions
 * Types specific to the Node.js backend implementation
 */

import { TmuxSession, TmuxWindow, BaseMessage } from './index';
import type { IPty } from 'node-pty';
import type { WebSocket } from 'ws';

// ==================== PTY Types ====================

/**
 * PTY process configuration
 */
export interface PtyConfig {
  name?: string;
  cols?: number;
  rows?: number;
  cwd?: string;
  env?: Record<string, string>;
  encoding?: string;
}

/**
 * PTY process wrapper
 */
export interface PtyProcess {
  pty: IPty;
  sessionName: string;
  lastActivity: Date;
  buffer?: string[];
  isPaused?: boolean;
}

// ==================== WebSocket Client Types ====================

/**
 * Extended WebSocket with client metadata
 */
export interface ExtendedWebSocket extends WebSocket {
  id?: string;
  ptyProcess?: PtyProcess;
  sessionName?: string;
  isAlive?: boolean;
  lastPing?: Date;
}

/**
 * WebSocket client connection info
 */
export interface ClientConnection {
  id: string;
  ws: ExtendedWebSocket;
  sessionName?: string;
  connectedAt: Date;
  lastActivity: Date;
  remoteAddress?: string;
}

// ==================== TMUX Command Types ====================

/**
 * TMUX command execution options
 */
export interface TmuxCommandOptions {
  args: string[];
  timeout?: number;
  encoding?: string;
}

/**
 * TMUX command result
 */
export interface TmuxCommandResult {
  success: boolean;
  output?: string;
  error?: string;
  code?: number;
}

/**
 * TMUX session format fields
 */
export interface TmuxSessionFormat {
  session_name: string;
  session_attached: '0' | '1';
  session_created: string;
  session_windows: string;
  session_width: string;
  session_height: string;
}

/**
 * TMUX window format fields
 */
export interface TmuxWindowFormat {
  window_index: string;
  window_name: string;
  window_active: '0' | '1';
  window_layout: string;
  window_panes: string;
}

// ==================== Server Configuration Types ====================

/**
 * Server configuration
 */
export interface ServerConfig {
  port: number;
  httpsPort?: number;
  httpsOptions?: {
    key: string | Buffer;
    cert: string | Buffer;
  };
  cors?: {
    origin?: string | string[] | boolean;
    credentials?: boolean;
  };
  wsOptions?: {
    maxPayload?: number;
    perMessageDeflate?: boolean;
    clientTracking?: boolean;
  };
}

/**
 * Session manager configuration
 */
export interface SessionManagerConfig {
  maxSessionsPerClient?: number;
  sessionTimeout?: number;
  cleanupInterval?: number;
  bufferSize?: number;
  flowControlThreshold?: number;
}

// ==================== Handler Types ====================

/**
 * WebSocket message handler
 */
export type MessageHandler<T extends BaseMessage = BaseMessage> = (
  ws: ExtendedWebSocket,
  message: T
) => void | Promise<void>;

/**
 * Message handler map
 */
export type MessageHandlerMap = {
  [K in BaseMessage['type']]?: MessageHandler;
};

/**
 * Request handler context
 */
export interface HandlerContext {
  ws: ExtendedWebSocket;
  clients: Map<string, ClientConnection>;
  sessionManager?: SessionManager;
}

// ==================== Session Management Types ====================

/**
 * Session manager interface
 */
export interface SessionManager {
  createSession(name: string, options?: PtyConfig): Promise<TmuxSession>;
  killSession(name: string): Promise<void>;
  renameSession(oldName: string, newName: string): Promise<void>;
  listSessions(): Promise<TmuxSession[]>;
  attachToSession(
    ws: ExtendedWebSocket,
    sessionName: string,
    dimensions: { cols: number; rows: number }
  ): Promise<void>;
  detachFromSession(ws: ExtendedWebSocket): void;
  handleInput(ws: ExtendedWebSocket, data: string): void;
  handleResize(
    ws: ExtendedWebSocket,
    dimensions: { cols: number; rows: number }
  ): void;
}

/**
 * Window manager interface
 */
export interface WindowManager {
  listWindows(sessionName: string): Promise<TmuxWindow[]>;
  createWindow(sessionName: string, windowName?: string): Promise<TmuxWindow>;
  killWindow(sessionName: string, windowIndex: number): Promise<void>;
  renameWindow(
    sessionName: string,
    windowIndex: number,
    newName: string
  ): Promise<void>;
  selectWindow(sessionName: string, windowIndex: number): Promise<void>;
}

// ==================== Logging Types ====================

/**
 * Log levels
 */
export enum LogLevel {
  ERROR = 'error',
  WARN = 'warn',
  INFO = 'info',
  DEBUG = 'debug',
  TRACE = 'trace'
}

/**
 * Logger interface
 */
export interface Logger {
  error(message: string, ...args: any[]): void;
  warn(message: string, ...args: any[]): void;
  info(message: string, ...args: any[]): void;
  debug(message: string, ...args: any[]): void;
  trace(message: string, ...args: any[]): void;
}

// ==================== Error Types ====================

/**
 * WebMux error codes
 */
export enum ErrorCode {
  SESSION_NOT_FOUND = 'SESSION_NOT_FOUND',
  WINDOW_NOT_FOUND = 'WINDOW_NOT_FOUND',
  TMUX_NOT_RUNNING = 'TMUX_NOT_RUNNING',
  INVALID_SESSION_NAME = 'INVALID_SESSION_NAME',
  SESSION_ALREADY_EXISTS = 'SESSION_ALREADY_EXISTS',
  PTY_CREATION_FAILED = 'PTY_CREATION_FAILED',
  WEBSOCKET_ERROR = 'WEBSOCKET_ERROR',
  COMMAND_EXECUTION_ERROR = 'COMMAND_EXECUTION_ERROR',
  PERMISSION_DENIED = 'PERMISSION_DENIED',
  TIMEOUT = 'TIMEOUT'
}

/**
 * WebMux error class
 */
export class WebMuxError extends Error {
  constructor(
    public code: ErrorCode,
    message: string,
    public details?: any
  ) {
    super(message);
    this.name = 'WebMuxError';
  }
}

// ==================== Utility Types ====================

/**
 * Async command executor
 */
export type CommandExecutor = (
  command: string,
  args: string[]
) => Promise<TmuxCommandResult>;

/**
 * Session event emitter events
 */
export interface SessionEventMap {
  'session:created': (session: TmuxSession) => void;
  'session:destroyed': (sessionName: string) => void;
  'session:attached': (sessionName: string, clientId: string) => void;
  'session:detached': (sessionName: string, clientId: string) => void;
  'client:connected': (clientId: string) => void;
  'client:disconnected': (clientId: string) => void;
  'error': (error: WebMuxError) => void;
}

/**
 * Performance metrics
 */
export interface PerformanceMetrics {
  activeConnections: number;
  activeSessions: number;
  messagesSent: number;
  messagesReceived: number;
  bytesTransferred: number;
  avgLatency: number;
  uptime: number;
}