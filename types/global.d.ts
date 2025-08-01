// Global type declarations for WebMux

// Session types
export interface TmuxSession {
  name: string;
  windows: number;
  created: string;
  attached: boolean;
}

export interface TmuxWindow {
  index: number;
  name: string;
  active: boolean;
  panes: number;
}

// WebSocket message types
export interface WSMessage {
  type: string;
  [key: string]: any;
}

export interface AttachSessionMessage extends WSMessage {
  type: 'attach-session';
  sessionName: string;
  cols: number;
  rows: number;
}

export interface InputMessage extends WSMessage {
  type: 'input';
  data: string;
}

export interface ResizeMessage extends WSMessage {
  type: 'resize';
  cols: number;
  rows: number;
}

export interface OutputMessage extends WSMessage {
  type: 'output';
  data: string;
}

// Express extensions
declare global {
  namespace Express {
    interface Request {
      sessionName?: string;
    }
  }
}