/**
 * WebMux Frontend-Specific TypeScript Type Definitions
 * Types specific to the Vue.js frontend implementation
 */

import { Ref, ComputedRef } from 'vue';
import { TmuxSession, TmuxWindow, ConnectionState, TerminalOptions } from './index';
import type { Terminal } from '@xterm/xterm';

// ==================== Vue Composable Return Types ====================

/**
 * useWebSocket composable return type
 */
export interface UseWebSocketReturn {
  isConnected: Ref<boolean>;
  connectionState: Ref<ConnectionState>;
  connect: () => Promise<void>;
  disconnect: () => void;
  send: (data: any) => void;
  onMessage: (type: string, handler: (data: any) => void) => void;
  offMessage: (type: string, handler: (data: any) => void) => void;
}

/**
 * useTerminal composable return type
 */
export interface UseTerminalReturn {
  terminal: Ref<Terminal | null>;
  isReady: Ref<boolean>;
  attachToSession: (sessionName: string) => Promise<void>;
  detachFromSession: () => void;
  write: (data: string) => void;
  clear: () => void;
  focus: () => void;
  blur: () => void;
  resize: (cols: number, rows: number) => void;
  getSize: () => { cols: number; rows: number } | null;
}

/**
 * useSessionManager composable return type
 */
export interface UseSessionManagerReturn {
  sessions: Ref<TmuxSession[]>;
  currentSession: Ref<TmuxSession | null>;
  isLoading: Ref<boolean>;
  error: Ref<Error | null>;
  fetchSessions: () => Promise<void>;
  createSession: (name: string) => Promise<void>;
  killSession: (name: string) => Promise<void>;
  renameSession: (oldName: string, newName: string) => Promise<void>;
  selectSession: (session: TmuxSession) => void;
}

/**
 * useWindowManager composable return type
 */
export interface UseWindowManagerReturn {
  windows: Ref<TmuxWindow[]>;
  currentWindow: Ref<TmuxWindow | null>;
  isLoading: Ref<boolean>;
  error: Ref<Error | null>;
  fetchWindows: (sessionName: string) => Promise<void>;
  createWindow: (sessionName: string, windowName?: string) => Promise<void>;
  killWindow: (sessionName: string, windowIndex: number) => Promise<void>;
  renameWindow: (sessionName: string, windowIndex: number, newName: string) => Promise<void>;
  selectWindow: (sessionName: string, windowIndex: number) => Promise<void>;
}

// ==================== Store Types ====================

/**
 * Terminal store state
 */
export interface TerminalStoreState {
  sessions: TmuxSession[];
  currentSession: TmuxSession | null;
  windows: TmuxWindow[];
  currentWindow: TmuxWindow | null;
  connectionState: ConnectionState;
  terminalOptions: TerminalOptions;
}

/**
 * Terminal store getters
 */
export interface TerminalStoreGetters {
  isConnected: ComputedRef<boolean>;
  hasActiveSessions: ComputedRef<boolean>;
  activeSessionCount: ComputedRef<number>;
  currentSessionName: ComputedRef<string | null>;
  currentWindowIndex: ComputedRef<number | null>;
}

/**
 * Terminal store actions
 */
export interface TerminalStoreActions {
  initialize: () => Promise<void>;
  connectWebSocket: () => Promise<void>;
  disconnectWebSocket: () => void;
  fetchSessions: () => Promise<void>;
  selectSession: (session: TmuxSession) => void;
  createSession: (name: string) => Promise<void>;
  killSession: (name: string) => Promise<void>;
  renameSession: (oldName: string, newName: string) => Promise<void>;
  fetchWindows: (sessionName: string) => Promise<void>;
  selectWindow: (windowIndex: number) => Promise<void>;
  updateTerminalOptions: (options: Partial<TerminalOptions>) => void;
}

// ==================== Component Emits Types ====================

/**
 * SessionList component emits
 */
export interface SessionListEmits {
  (e: 'select', session: TmuxSession): void;
  (e: 'create', name: string): void;
  (e: 'kill', sessionName: string): void;
  (e: 'rename', sessionName: string, newName: string): void;
  (e: 'refresh'): void;
}

/**
 * WindowList component emits
 */
export interface WindowListEmits {
  (e: 'select', window: TmuxWindow): void;
  (e: 'create', windowName?: string): void;
  (e: 'kill', windowIndex: number): void;
  (e: 'rename', windowIndex: number, newName: string): void;
  (e: 'refresh'): void;
}

/**
 * TerminalView component emits
 */
export interface TerminalViewEmits {
  (e: 'ready'): void;
  (e: 'disconnect'): void;
  (e: 'resize', dimensions: { cols: number; rows: number }): void;
  (e: 'data', data: string): void;
  (e: 'title', title: string): void;
  (e: 'bell'): void;
}

// ==================== UI State Types ====================

/**
 * Modal state
 */
export interface ModalState {
  isOpen: boolean;
  title: string;
  type: 'create' | 'rename' | 'confirm' | 'error';
  data?: any;
}

/**
 * Notification types
 */
export enum NotificationType {
  SUCCESS = 'success',
  ERROR = 'error',
  WARNING = 'warning',
  INFO = 'info'
}

/**
 * Notification
 */
export interface Notification {
  id: string;
  type: NotificationType;
  title: string;
  message?: string;
  duration?: number;
  timestamp: Date;
}

/**
 * Context menu item
 */
export interface ContextMenuItem {
  id: string;
  label: string;
  icon?: string;
  shortcut?: string;
  disabled?: boolean;
  separator?: boolean;
  action?: () => void;
  submenu?: ContextMenuItem[];
}

// ==================== Form Types ====================

/**
 * Session create form data
 */
export interface SessionCreateFormData {
  name: string;
  command?: string;
  workingDirectory?: string;
}

/**
 * Session rename form data
 */
export interface SessionRenameFormData {
  currentName: string;
  newName: string;
}

/**
 * Terminal settings form data
 */
export interface TerminalSettingsFormData {
  fontSize: number;
  fontFamily: string;
  cursorStyle: 'block' | 'underline' | 'bar';
  cursorBlink: boolean;
  scrollback: number;
  bellStyle: 'none' | 'sound' | 'visual' | 'both';
  theme: 'dark' | 'light' | 'custom';
  customTheme?: Record<string, string>;
}

// ==================== Router Types ====================

/**
 * Route params for session view
 */
export interface SessionRouteParams {
  sessionName: string;
  windowIndex?: string;
}

/**
 * Router meta for terminal routes
 */
export interface TerminalRouteMeta {
  requiresConnection: boolean;
  requiresSession: boolean;
  title?: string;
}

// ==================== Event Handler Types ====================

/**
 * Keyboard shortcut handler
 */
export interface KeyboardShortcut {
  key: string;
  ctrl?: boolean;
  alt?: boolean;
  shift?: boolean;
  meta?: boolean;
  action: () => void;
  description?: string;
}

/**
 * Drag and drop handler
 */
export interface DragDropHandler {
  onDragStart?: (event: DragEvent) => void;
  onDragEnd?: (event: DragEvent) => void;
  onDragOver?: (event: DragEvent) => void;
  onDrop?: (event: DragEvent) => void;
}

// ==================== Utility Types ====================

/**
 * Terminal fit addon options
 */
export interface FitAddonOptions {
  autoResize?: boolean;
  debounceDelay?: number;
}

/**
 * Search addon options
 */
export interface SearchAddonOptions {
  caseSensitive?: boolean;
  wholeWord?: boolean;
  regex?: boolean;
  incremental?: boolean;
}

/**
 * Terminal performance metrics
 */
export interface TerminalMetrics {
  fps: number;
  renderTime: number;
  bufferSize: number;
  scrollbackSize: number;
}

/**
 * Session statistics
 */
export interface SessionStatistics {
  totalSessions: number;
  activeSessions: number;
  totalWindows: number;
  totalPanes: number;
  uptime: number;
  lastActivity: Date;
}

// ==================== API Query Types ====================

/**
 * Query options for @tanstack/vue-query
 */
export interface QueryOptions<T = any> {
  enabled?: boolean;
  refetchInterval?: number;
  refetchOnWindowFocus?: boolean;
  retry?: number | boolean;
  staleTime?: number;
  cacheTime?: number;
  onSuccess?: (data: T) => void;
  onError?: (error: Error) => void;
}

/**
 * Mutation options for @tanstack/vue-query
 */
export interface MutationOptions<T = any, V = any> {
  onSuccess?: (data: T, variables: V) => void;
  onError?: (error: Error, variables: V) => void;
  onSettled?: (data: T | undefined, error: Error | null, variables: V) => void;
}