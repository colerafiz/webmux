import express, { Request, Response } from 'express';
import https from 'https';
import fs from 'fs';
import WebSocket from 'ws';
import * as pty from 'node-pty';
import { spawn, exec } from 'child_process';
import path from 'path';
import cors from 'cors';
import os from 'os';
import { promisify } from 'util';
import type { IPty } from 'node-pty';
import type { Server } from 'http';
import type { 
  WebSocketMessage,
  TmuxSession,
  TmuxWindow,
  CreateSessionRequest,
  RenameSessionRequest,
  CreateWindowRequest,
  RenameWindowRequest,
  SystemStats
} from './backend-types';

const execAsync = promisify(exec);

const app = express();
// Dev branch uses different ports to allow running alongside main
const port = 4000; // Main uses 3000
const httpsPort = 4443; // Main uses 3443

// HTTPS configuration
// Fix path resolution for ts-node vs compiled
const rootDir = path.resolve(__dirname.includes('node_modules') ? process.cwd() : __dirname);
const certsDir = path.join(rootDir, 'certs');

let httpsOptions: https.ServerOptions | undefined;
try {
  httpsOptions = {
    key: fs.readFileSync(path.join(certsDir, 'key.pem')),
    cert: fs.readFileSync(path.join(certsDir, 'cert.pem'))
  };
} catch (error) {
  console.error('Warning: Could not load SSL certificates from', certsDir);
  console.error('HTTPS server will not be available');
}

// Configure CORS to accept requests from any origin
app.use(cors({
  origin: true, // Accept requests from any origin
  credentials: true
}));
app.use(express.json());
app.use(express.static('public'));

// System stats endpoint
app.get('/api/stats', (_req: Request, res: Response) => {
  const cpus = os.cpus();
  const totalMem = os.totalmem();
  const freeMem = os.freemem();
  const usedMem = totalMem - freeMem;
  const loadAvg = os.loadavg();
  
  const stats: SystemStats = {
    cpu: {
      cores: cpus.length,
      model: cpus[0].model,
      usage: loadAvg[0],
      loadAvg: loadAvg as [number, number, number]
    },
    memory: {
      total: totalMem,
      used: usedMem,
      free: freeMem,
      percent: ((usedMem / totalMem) * 100).toFixed(1)
    },
    uptime: os.uptime(),
    hostname: os.hostname(),
    platform: os.platform(),
    arch: os.arch()
  };

  res.json(stats);
});

// REST API endpoints
app.get('/api/sessions', async (_req: Request, res: Response) => {
  // First check if tmux server is running
  try {
    await execAsync('tmux list-sessions 2>/dev/null');
  } catch (error) {
    // TMUX not running, return empty list
    res.json({ sessions: [] });
    return;
  }
  
  const listCmd = spawn('tmux', ['list-sessions', '-F', '#{session_name}:#{session_attached}:#{session_created}:#{session_windows}:#{session_width}x#{session_height}']);

  let output = '';
  
  listCmd.stdout.on('data', (data) => {
    output += data.toString();
  });

  listCmd.on('close', (code) => {
    if (code !== 0) {
      res.json({ sessions: [] });
      return;
    }
    
    const sessions: TmuxSession[] = output.trim().split('\n').filter(line => line)
      .map(line => {
        const [name, attached, created, windows, dimensions] = line.split(':');
        return { 
          name, 
          attached: attached === '1', 
          created: new Date(parseInt(created) * 1000),
          windows: parseInt(windows),
          dimensions
        };
      });
    
    res.json({ sessions });
  });
});

app.post('/api/sessions/:name/kill', (req: Request, res: Response) => {
  const { name } = req.params;
  const killCmd = spawn('tmux', ['kill-session', '-t', name]);

  killCmd.on('close', (code) => {
    if (code === 0) {
      res.json({ success: true });
    } else {
      res.status(400).json({ success: false, error: 'Failed to kill session' });
    }
  });
});

app.post('/api/sessions/:name/rename', async (req: Request<{ name: string }, any, RenameSessionRequest>, res: Response) => {
  const { name } = req.params;
  const { newName } = req.body;
  
  // Validate input
  if (!newName || newName.trim() === '') {
    return res.status(400).json({ 
      success: false, 
      error: 'Session name cannot be empty' 
    });
  }
  
  try {
    // Use execAsync with proper escaping
    await execAsync(`tmux rename-session -t '${name.replace(/'/g, "'\\''")}' '${newName.replace(/'/g, "'\\''")}'`);
    res.json({ success: true });
  } catch (error: any) {
    console.error('Failed to rename session:', error);
    res.status(400).json({ 
      success: false, 
      error: error.message || 'Failed to rename session' 
    });
  }
});

app.post('/api/sessions', async (req: Request<any, any, CreateSessionRequest>, res: Response) => {
  const { name } = req.body;
  const sessionName = name || `session-${Date.now()}`;
  
  // First check if tmux server is running
  try {
    await execAsync('tmux list-sessions 2>/dev/null');
  } catch (error) {
    // TMUX server not running, start it with a dummy session that will be removed
    try {
      await execAsync('tmux new-session -d -s __dummy__ -c ~ "exit"');
      // Small delay to ensure server is fully started
      await new Promise(resolve => setTimeout(resolve, 100));
    } catch (startError) {
      console.error('Failed to start TMUX server:', startError);
    }
  }
  
  const createCmd = spawn('tmux', ['new-session', '-d', '-s', sessionName], {
    cwd: process.env.HOME
  });

  createCmd.on('close', (code) => {
    if (code === 0) {
      res.json({ success: true, sessionName });
    } else {
      res.status(400).json({ success: false, error: 'Failed to create session' });
    }
  });
});

// Window management endpoints
app.get('/api/sessions/:name/windows', (req: Request, res: Response) => {
  const { name } = req.params;
  const listCmd = spawn('tmux', ['list-windows', '-t', name, '-F', '#{window_index}:#{window_name}:#{window_active}:#{window_panes}']);

  let output = '';
  
  listCmd.stdout.on('data', (data) => {
    output += data.toString();
  });

  listCmd.on('close', (code) => {
    if (code !== 0) {
      res.status(404).json({ error: 'Session not found' });
      return;
    }
    
    const windows: TmuxWindow[] = output.trim().split('\n').filter(line => line)
      .map(line => {
        const [index, name, active, panes] = line.split(':');
        return { 
          index: parseInt(index),
          name, 
          active: active === '1',
          panes: parseInt(panes)
        };
      });
    
    res.json({ windows });
  });
});

app.post('/api/sessions/:name/windows', (req: Request<{ name: string }, any, CreateWindowRequest>, res: Response) => {
  const { name } = req.params;
  const { windowName } = req.body;
  
  // Use -a flag to append window at the end (next available index)
  const args = ['new-window', '-a', '-t', name];
  if (windowName) {
    args.push('-n', windowName);
  }
  
  const createCmd = spawn('tmux', args);

  createCmd.on('close', (code) => {
    if (code === 0) {
      res.json({ success: true });
    } else {
      res.status(400).json({ success: false, error: 'Failed to create window' });
    }
  });
});

app.delete('/api/sessions/:sessionName/windows/:windowIndex', (req: Request, res: Response) => {
  const { sessionName, windowIndex } = req.params;
  const killCmd = spawn('tmux', ['kill-window', '-t', `${sessionName}:${windowIndex}`]);

  killCmd.on('close', (code) => {
    if (code === 0) {
      res.json({ success: true });
    } else {
      res.status(400).json({ success: false, error: 'Failed to kill window' });
    }
  });
});

app.post('/api/sessions/:sessionName/windows/:windowIndex/rename', async (req: Request<{ sessionName: string, windowIndex: string }, any, RenameWindowRequest>, res: Response) => {
  const { sessionName, windowIndex } = req.params;
  const { newName } = req.body;
  
  // Validate input
  if (!newName || newName.trim() === '') {
    return res.status(400).json({ 
      success: false, 
      error: 'Window name cannot be empty' 
    });
  }
  
  try {
    // Use execAsync with proper escaping
    const target = `${sessionName}:${windowIndex}`;
    await execAsync(`tmux rename-window -t '${target}' '${newName.replace(/'/g, "'\\''")}'`);
    res.json({ success: true });
  } catch (error: any) {
    console.error('Failed to rename window:', error);
    res.status(400).json({ 
      success: false, 
      error: error.message || 'Failed to rename window' 
    });
  }
});

app.post('/api/sessions/:sessionName/windows/:windowIndex/select', (req: Request, res: Response) => {
  const { sessionName, windowIndex } = req.params;
  
  const selectCmd = spawn('tmux', ['select-window', '-t', `${sessionName}:${windowIndex}`]);

  selectCmd.on('close', (code) => {
    if (code === 0) {
      res.json({ success: true });
    } else {
      res.status(400).json({ success: false, error: 'Failed to select window' });
    }
  });
});

// Start HTTP server (for development/redirect)
const server: Server = app.listen(port, '0.0.0.0', () => {
  console.log(`WebMux HTTP server running on port ${port}`);
  console.log(`  Local:    http://localhost:${port}`);
  console.log(`  Network:  http://0.0.0.0:${port}`);
});

// Start HTTPS server (only if certificates are available)
let httpsServer: https.Server | undefined;
if (httpsOptions) {
  httpsServer = https.createServer(httpsOptions, app);
  httpsServer.listen(httpsPort, '0.0.0.0', () => {
    console.log(`WebMux HTTPS server running on port ${httpsPort}`);
    console.log(`  Local:    https://localhost:${httpsPort}`);
    console.log(`  Network:  https://0.0.0.0:${httpsPort}`);
    console.log(`  Tailscale: Use your Tailscale IP with port ${httpsPort}`);
    console.log(`  Note: You may need to accept the self-signed certificate`);
  });
}

// WebSocket servers for both HTTP and HTTPS
const wss = new WebSocket.Server({ server, path: '/ws' });
const wssHttps = httpsServer ? new WebSocket.Server({ server: httpsServer, path: '/ws' }) : null;

// Session management
const sessions = new Map<WebSocket, IPty>();

// WebSocket connection handler (shared between HTTP and HTTPS)
function handleWebSocketConnection(ws: WebSocket): void {
  console.log('New WebSocket connection established');
  
  ws.on('message', (message: WebSocket.RawData) => {
    try {
      const data = JSON.parse(message.toString()) as WebSocketMessage;
      
      switch (data.type) {
        case 'list-sessions':
          listTmuxSessions(ws);
          break;
        
        case 'attach-session':
          console.log('Attaching to session:', data.sessionName);
          attachToSession(ws, data.sessionName, data.cols, data.rows);
          break;
        
        case 'input':
          if (sessions.has(ws)) {
            const ptyProcess = sessions.get(ws)!;
            ptyProcess.write(data.data);
          }
          break;
          
        case 'resize':
          if (sessions.has(ws)) {
            const ptyProcess = sessions.get(ws)!;
            ptyProcess.resize(data.cols, data.rows);
          }
          break;
          
        case 'list-windows':
          listSessionWindows(ws, data.sessionName);
          break;
          
        case 'select-window':
          console.log('Selecting window:', data.windowIndex, 'in session:', data.sessionName);
          selectWindow(ws, data.sessionName, data.windowIndex);
          break;
          
        case 'ping':
          // Respond to ping with pong
          if (ws.readyState === WebSocket.OPEN) {
            ws.send(JSON.stringify({ type: 'pong' }));
          }
          break;
      }
    } catch (err) {
      console.error('Error handling message:', err);
    }
  });

  ws.on('close', () => {
    console.log('WebSocket connection closed');
    if (sessions.has(ws)) {
      const ptyProcess = sessions.get(ws)!;
      console.log('Killing PTY process for closed connection');
      ptyProcess.kill();
      sessions.delete(ws);
      console.log('Remaining sessions:', sessions.size);
    }
  });
}

// Attach WebSocket handlers to both servers
wss.on('connection', handleWebSocketConnection);
if (wssHttps) {
  wssHttps.on('connection', handleWebSocketConnection);
}

async function listTmuxSessions(ws: WebSocket): Promise<void> {
  // First check if tmux server is running
  try {
    await execAsync('tmux list-sessions 2>/dev/null');
  } catch (error) {
    // TMUX not running, return empty list
    ws.send(JSON.stringify({
      type: 'sessions-list',
      sessions: []
    }));
    return;
  }
  
  const listCmd = spawn('tmux', ['list-sessions', '-F', '#{session_name}:#{session_attached}:#{session_created}']);

  let output = '';
  
  listCmd.stdout.on('data', (data) => {
    output += data.toString();
  });

  listCmd.on('close', (_code) => {
    const sessions = output.trim().split('\n').filter(line => line)
      .map(line => {
        const [name, attached, created] = line.split(':');
        return { name, attached: attached === '1', created: new Date(parseInt(created) * 1000) };
      });
    
    ws.send(JSON.stringify({
      type: 'sessions-list',
      sessions: sessions
    }));
  });
}

function attachToSession(ws: WebSocket, sessionName: string, cols: number = 120, rows: number = 40): void {
  console.log(`Attaching to session '${sessionName}'`);
  
  // Check if we already have a PTY for this connection
  if (sessions.has(ws)) {
    const ptyProcess = sessions.get(ws)!;
    console.log('Reusing existing PTY connection');
    
    // Just send the tmux switch command
    ptyProcess.write(`\x03`); // Ctrl-C to clear any current input
    setTimeout(() => {
      ptyProcess.write(`tmux switch-client -t '${sessionName}' 2>/dev/null || tmux attach-session -t '${sessionName}'\r`);
      
      // Send attached confirmation
      setTimeout(() => {
        if (ws.readyState === WebSocket.OPEN) {
          ws.send(JSON.stringify({
            type: 'attached',
            sessionName: sessionName
          }));
        }
      }, 200);
    }, 50);
    
    return;
  }
  
  // Only create new PTY if we don't have one
  createNewPtySession(ws, sessionName, cols, rows);
}

function createNewPtySession(ws: WebSocket, sessionName: string, cols: number, rows: number): void {
  console.log('Creating initial PTY session for:', sessionName);

  // Create a new shell that will attach to the tmux session
  const shell = process.platform === 'win32' ? 'powershell.exe' : 'bash';
  const ptyProcess = pty.spawn(shell, [], {
    name: 'xterm-256color',
    cols: cols,
    rows: rows,
    cwd: process.env.HOME,
    env: {
      ...process.env,
      TERM: 'xterm-256color',
      COLORTERM: 'truecolor'
    }
  });

  sessions.set(ws, ptyProcess);
  console.log('PTY session stored for WebSocket, total sessions:', sessions.size);

  // Attach to tmux session immediately
  ptyProcess.write(`tmux attach-session -t '${sessionName}' || tmux new-session -s '${sessionName}'\r`);

  // Simple direct output - no buffering to avoid state issues
  ptyProcess.onData((data: string) => {
    if (ws.readyState === WebSocket.OPEN) {
      try {
        // Send data directly, but limit size to prevent issues
        const maxChunkSize = 32 * 1024; // 32KB max per message
        if (data.length > maxChunkSize) {
          // Split large data into chunks
          for (let i = 0; i < data.length; i += maxChunkSize) {
            const chunk = data.slice(i, i + maxChunkSize);
            ws.send(JSON.stringify({
              type: 'output',
              data: chunk
            }));
          }
        } else {
          ws.send(JSON.stringify({
            type: 'output',
            data: data
          }));
        }
      } catch (err) {
        console.error('WebSocket send error:', err);
        // If WebSocket fails, try to reconnect client
        if (ws.readyState !== WebSocket.OPEN) {
          console.log('WebSocket connection lost, cleaning up PTY');
          sessions.delete(ws);
          ptyProcess.kill();
        }
      }
    }
  });

  // Handle PTY exit
  ptyProcess.onExit(() => {
    console.log('PTY process exited for session:', sessionName);
    if (ws.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify({
        type: 'disconnected'
      }));
    }
    sessions.delete(ws);
  });

  // Send attached confirmation
  if (ws.readyState === WebSocket.OPEN) {
    ws.send(JSON.stringify({
      type: 'attached',
      sessionName: sessionName
    }));
  }
}

function listSessionWindows(ws: WebSocket, sessionName: string): void {
  const listCmd = spawn('tmux', ['list-windows', '-t', sessionName, '-F', '#{window_index}:#{window_name}:#{window_active}']);

  let output = '';
  
  listCmd.stdout.on('data', (data) => {
    output += data.toString();
  });

  listCmd.on('close', (code) => {
    if (code !== 0) {
      ws.send(JSON.stringify({
        type: 'windows-list',
        windows: []
      }));
      return;
    }
    
    const windows = output.trim().split('\n').filter(line => line)
      .map(line => {
        const [index, name, active] = line.split(':');
        return { 
          index: parseInt(index),
          name, 
          active: active === '1'
        };
      });
    
    ws.send(JSON.stringify({
      type: 'windows-list',
      windows: windows
    }));
  });
}

function selectWindow(ws: WebSocket, sessionName: string, windowIndex: number): void {
  console.log(`Switching to window ${windowIndex} in session ${sessionName}`);
  console.log('WebSocket readyState:', ws.readyState);
  console.log('Sessions map size:', sessions.size);
  
  if (!sessions.has(ws)) {
    console.error('No PTY session found for WebSocket');
    console.error('Available sessions:', Array.from(sessions.keys()).map(k => k.readyState));
    ws.send(JSON.stringify({
      type: 'window-selected',
      success: false,
      error: 'No active terminal session'
    }));
    return;
  }

  const ptyProcess = sessions.get(ws)!;
  
  // Use tmux command directly instead of keyboard shortcuts
  // This is more reliable and doesn't depend on the prefix key
  const selectCmd = spawn('tmux', ['select-window', '-t', `${sessionName}:${windowIndex}`]);
  
  selectCmd.on('close', (code) => {
    if (code === 0) {
      console.log('Window selected successfully');
      
      // Send a refresh command to the PTY to update the display
      // Send Ctrl-L to refresh the terminal
      ptyProcess.write('\x0c');
      
      ws.send(JSON.stringify({
        type: 'window-selected',
        success: true,
        windowIndex: windowIndex
      }));
      
      // Refresh windows list
      setTimeout(() => listSessionWindows(ws, sessionName), 200);
    } else {
      console.error('Failed to select window, exit code:', code);
      ws.send(JSON.stringify({
        type: 'window-selected',
        success: false,
        error: 'Failed to select window'
      }));
    }
  });
}