import { spawn } from 'child_process';
import { EventEmitter } from 'events';

interface TmuxHandlerEvents {
  output: (data: string) => void;
  error: (error: Error) => void;
}

declare interface TmuxHandler {
  on<U extends keyof TmuxHandlerEvents>(
    event: U, listener: TmuxHandlerEvents[U]
  ): this;
  
  emit<U extends keyof TmuxHandlerEvents>(
    event: U, ...args: Parameters<TmuxHandlerEvents[U]>
  ): boolean;
}

class TmuxHandler extends EventEmitter {
  private sessionName: string;
  private cols: number;
  private rows: number;
  private pollInterval: NodeJS.Timeout | null;
  private lastContent: string;
  private isActive: boolean;
  private inputQueue: string[];
  private isProcessingInput: boolean;

  constructor(sessionName: string, cols: number = 120, rows: number = 40) {
    super();
    this.sessionName = sessionName;
    this.cols = cols;
    this.rows = rows;
    this.pollInterval = null;
    this.lastContent = '';
    this.isActive = false;
    this.inputQueue = [];
    this.isProcessingInput = false;
  }

  start(): void {
    if (this.isActive) return;
    this.isActive = true;
    
    // Set window size for the tmux session
    this.setWindowSize(this.cols, this.rows);
    
    // Start polling for content
    this.pollInterval = setInterval(() => this.capturePane(), 100);
    
    // Initial capture
    this.capturePane();
  }

  stop(): void {
    this.isActive = false;
    if (this.pollInterval) {
      clearInterval(this.pollInterval);
      this.pollInterval = null;
    }
  }

  private capturePane(): void {
    if (!this.isActive) return;
    
    const capture = spawn('tmux', [
      'capture-pane',
      '-t', this.sessionName,
      '-p',
      '-e',  // Include escape sequences for colors
      '-J',  // Join wrapped lines
      '-S', `-${this.rows}`,  // Only capture visible content plus a buffer
      '-E', '-'   // End at the bottom
    ]);

    let output = '';
    
    capture.stdout.on('data', (data) => {
      output += data.toString();
    });

    capture.on('close', (code) => {
      if (code === 0) {
        // Only emit if content has changed
        if (output !== this.lastContent) {
          this.lastContent = output;
          this.emit('output', output);
        }
      }
    });

    capture.on('error', (err) => {
      // Silently handle errors to avoid spamming
      if ((err as NodeJS.ErrnoException).code !== 'ENOENT') {
        this.emit('error', err);
      }
    });
  }

  sendInput(data: string): void {
    if (!this.isActive) return;
    
    // Add to queue
    this.inputQueue.push(data);
    this.processInputQueue();
  }

  private async processInputQueue(): Promise<void> {
    if (this.isProcessingInput || this.inputQueue.length === 0) return;
    
    this.isProcessingInput = true;
    
    while (this.inputQueue.length > 0) {
      const data = this.inputQueue.shift()!;
      await this.sendSingleInput(data);
      // Small delay between inputs to prevent overwhelming
      await new Promise(resolve => setTimeout(resolve, 10));
    }
    
    this.isProcessingInput = false;
  }

  private sendSingleInput(data: string): Promise<void> {
    return new Promise((resolve) => {
      // Handle each character individually for better compatibility
      const chars = data.split('');
      const args: string[] = ['send-keys', '-t', this.sessionName];
      
      for (const char of chars) {
        const code = char.charCodeAt(0);
        
        // Handle special keys
        if (code === 13) {
          args.push('Enter');
        } else if (code === 9) {
          args.push('Tab');
        } else if (code === 127) {
          args.push('BSpace');
        } else if (code === 27) {
          args.push('Escape');
        } else if (code < 32) {
          args.push(`C-${String.fromCharCode(64 + code)}`);
        } else {
          // Send literal characters
          args.push('-l', char);
        }
      }

      const sendKeys = spawn('tmux', args);

      sendKeys.on('close', () => {
        resolve();
      });

      sendKeys.on('error', (err) => {
        this.emit('error', err);
        resolve();
      });
    });
  }

  resize(cols: number, rows: number): void {
    this.cols = cols;
    this.rows = rows;
    this.setWindowSize(cols, rows);
  }

  private setWindowSize(cols: number, rows: number): void {
    const resize = spawn('tmux', [
      'resize-window',
      '-t', this.sessionName,
      '-x', cols.toString(),
      '-y', rows.toString()
    ]);

    resize.on('error', (_err) => {
      // Try alternative resize method
      spawn('tmux', [
        'resize-pane',
        '-t', this.sessionName,
        '-x', cols.toString(),
        '-y', rows.toString()
      ]);
    });
  }

  // Get cursor position for better terminal emulation
  getCursorPosition(): Promise<{ x: number; y: number }> {
    const cursor = spawn('tmux', [
      'display-message',
      '-t', this.sessionName,
      '-p', '#{cursor_x},#{cursor_y}'
    ]);

    return new Promise((resolve) => {
      let output = '';
      cursor.stdout.on('data', (data) => {
        output += data.toString();
      });
      cursor.on('close', () => {
        const [x, y] = output.trim().split(',').map(Number);
        resolve({ x, y });
      });
    });
  }
}

export default TmuxHandler;