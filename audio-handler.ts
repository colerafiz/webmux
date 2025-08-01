import { spawn, ChildProcess } from 'child_process';
import WebSocket from 'ws';
import ffmpeg from 'fluent-ffmpeg';
import { EventEmitter } from 'events';

export class AudioHandler extends EventEmitter {
  private ffmpegProcess: ChildProcess | null = null;
  private isStreaming: boolean = false;
  private clients: Set<WebSocket> = new Set();

  constructor() {
    super();
  }

  addClient(ws: WebSocket): void {
    this.clients.add(ws);
    console.log(`Audio client added. Total clients: ${this.clients.size}`);
    
    // Send current status
    ws.send(JSON.stringify({
      type: 'audio-status',
      streaming: this.isStreaming
    }));
  }

  removeClient(ws: WebSocket): void {
    this.clients.delete(ws);
    console.log(`Audio client removed. Total clients: ${this.clients.size}`);
    
    // Stop streaming if no clients
    if (this.clients.size === 0 && this.isStreaming) {
      this.stopStreaming();
    }
  }

  startStreaming(): void {
    if (this.isStreaming) {
      console.log('Audio streaming already active');
      return;
    }

    console.log('Starting audio streaming...');
    this.isStreaming = true;

    try {
      // Use ffmpeg to capture system audio
      // For Linux with PulseAudio:
      // -f pulse -i default captures from the default PulseAudio output
      // For macOS: -f avfoundation -i ":0"
      const isLinux = process.platform === 'linux';
      const isMac = process.platform === 'darwin';
      
      let inputArgs: string[] = [];
      if (isLinux) {
        // Capture from PulseAudio monitor
        inputArgs = ['-f', 'pulse', '-i', 'default'];
      } else if (isMac) {
        // Capture from macOS audio
        inputArgs = ['-f', 'avfoundation', '-i', ':0'];
      } else {
        throw new Error('Unsupported platform for audio capture');
      }

      // Spawn ffmpeg process
      this.ffmpegProcess = spawn('ffmpeg', [
        ...inputArgs,
        '-acodec', 'libopus',      // Use Opus codec for better web compatibility
        '-b:a', '128k',             // Audio bitrate
        '-ar', '48000',             // Sample rate
        '-ac', '2',                 // Stereo audio
        '-f', 'webm',               // Output format
        '-'                         // Output to stdout
      ]);

      // Handle ffmpeg stdout (audio data)
      this.ffmpegProcess.stdout?.on('data', (chunk: Buffer) => {
        this.broadcastAudioChunk(chunk);
      });

      // Handle ffmpeg stderr (logs)
      this.ffmpegProcess.stderr?.on('data', (data: Buffer) => {
        console.log('FFmpeg:', data.toString());
      });

      // Handle process exit
      this.ffmpegProcess.on('close', (code) => {
        console.log(`FFmpeg process exited with code ${code}`);
        this.isStreaming = false;
        this.ffmpegProcess = null;
        this.notifyClientsStatus(false);
      });

      // Handle process error
      this.ffmpegProcess.on('error', (error) => {
        console.error('FFmpeg error:', error);
        this.isStreaming = false;
        this.ffmpegProcess = null;
        this.notifyClientsStatus(false, error.message);
      });

      // Notify clients that streaming started
      this.notifyClientsStatus(true);

    } catch (error: any) {
      console.error('Failed to start audio streaming:', error);
      this.isStreaming = false;
      this.notifyClientsStatus(false, error.message);
    }
  }

  stopStreaming(): void {
    if (!this.isStreaming || !this.ffmpegProcess) {
      console.log('Audio streaming not active');
      return;
    }

    console.log('Stopping audio streaming...');
    
    // Kill ffmpeg process
    if (this.ffmpegProcess) {
      this.ffmpegProcess.kill('SIGTERM');
      this.ffmpegProcess = null;
    }
    
    this.isStreaming = false;
    this.notifyClientsStatus(false);
  }

  private broadcastAudioChunk(chunk: Buffer): void {
    const message = {
      type: 'audio-stream',
      data: chunk.toString('base64') // Convert to base64 for JSON transport
    };
    
    const messageStr = JSON.stringify(message);
    
    // Send to all connected clients
    this.clients.forEach(client => {
      if (client.readyState === WebSocket.OPEN) {
        try {
          client.send(messageStr);
        } catch (error) {
          console.error('Error sending audio chunk to client:', error);
        }
      }
    });
  }

  private notifyClientsStatus(streaming: boolean, error?: string): void {
    const message = JSON.stringify({
      type: 'audio-status',
      streaming,
      error
    });
    
    this.clients.forEach(client => {
      if (client.readyState === WebSocket.OPEN) {
        try {
          client.send(message);
        } catch (err) {
          console.error('Error sending status to client:', err);
        }
      }
    });
  }

  isClientStreaming(ws: WebSocket): boolean {
    return this.clients.has(ws);
  }
}

// Export singleton instance
export const audioHandler = new AudioHandler();