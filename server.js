const express = require('express');
const https = require('https');
const fs = require('fs');
const WebSocket = require('ws');
const pty = require('node-pty');
const { spawn, exec } = require('child_process');
const path = require('path');
const cors = require('cors');
const os = require('os');
const { promisify } = require('util');

const execAsync = promisify(exec);

const app = express();
const port = 3000;
const httpsPort = 3443;

// HTTPS configuration
const httpsOptions = {
  key: fs.readFileSync(path.join(__dirname, 'certs', 'key.pem')),
  cert: fs.readFileSync(path.join(__dirname, 'certs', 'cert.pem'))
};

app.use(cors());
app.use(express.json());
app.use(express.static('public'));

// System stats endpoint
app.get('/api/stats', (req, res) => {
  const cpus = os.cpus();
  const totalMem = os.totalmem();
  const freeMem = os.freemem();
  const usedMem = totalMem - freeMem;
  const loadAvg = os.loadavg();
  
  res.json({
    cpu: {
      cores: cpus.length,
      model: cpus[0].model,
      usage: loadAvg[0],
      loadAvg: loadAvg
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
  });
});

// REST API endpoints
app.get('/api/sessions', async (req, res) => {
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
    
    const sessions = output.trim().split('\n').filter(line => line)
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

app.post('/api/sessions/:name/kill', (req, res) => {
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

app.post('/api/sessions/:name/rename', (req, res) => {
  const { name } = req.params;
  const { newName } = req.body;
  
  const renameCmd = spawn('tmux', ['rename-session', '-t', name, newName]);

  renameCmd.on('close', (code) => {
    if (code === 0) {
      res.json({ success: true });
    } else {
      res.status(400).json({ success: false, error: 'Failed to rename session' });
    }
  });
});

app.post('/api/sessions', async (req, res) => {
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
app.get('/api/sessions/:name/windows', (req, res) => {
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
    
    const windows = output.trim().split('\n').filter(line => line)
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

app.post('/api/sessions/:name/windows', (req, res) => {
  const { name } = req.params;
  const { windowName } = req.body;
  
  const args = ['new-window', '-t', name];
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

app.delete('/api/sessions/:sessionName/windows/:windowIndex', (req, res) => {
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

app.post('/api/sessions/:sessionName/windows/:windowIndex/rename', (req, res) => {
  const { sessionName, windowIndex } = req.params;
  const { newName } = req.body;
  
  const renameCmd = spawn('tmux', ['rename-window', '-t', `${sessionName}:${windowIndex}`, newName]);

  renameCmd.on('close', (code) => {
    if (code === 0) {
      res.json({ success: true });
    } else {
      res.status(400).json({ success: false, error: 'Failed to rename window' });
    }
  });
});

app.post('/api/sessions/:sessionName/windows/:windowIndex/select', (req, res) => {
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
const server = app.listen(port, '0.0.0.0', () => {
  console.log(`WebMux HTTP server running on port ${port}`);
  console.log(`  Local:    http://localhost:${port}`);
  console.log(`  Network:  http://0.0.0.0:${port}`);
});

// Start HTTPS server
const httpsServer = https.createServer(httpsOptions, app);
httpsServer.listen(httpsPort, '0.0.0.0', () => {
  console.log(`WebMux HTTPS server running on port ${httpsPort}`);
  console.log(`  Local:    https://localhost:${httpsPort}`);
  console.log(`  Network:  https://0.0.0.0:${httpsPort}`);
  console.log(`  Tailscale: Use your Tailscale IP with port ${httpsPort}`);
  console.log(`  Note: You may need to accept the self-signed certificate`);
});

// WebSocket servers for both HTTP and HTTPS
const wss = new WebSocket.Server({ server, path: '/ws' });
const wssHttps = new WebSocket.Server({ server: httpsServer, path: '/ws' });

const sessions = new Map();

// WebSocket connection handler (shared between HTTP and HTTPS)
function handleWebSocketConnection(ws) {
  console.log('New WebSocket connection established');
  
  ws.on('message', (message) => {
    try {
      const data = JSON.parse(message);
      
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
            const ptyProcess = sessions.get(ws);
            ptyProcess.write(data.data);
          }
          break;
          
        case 'resize':
          if (sessions.has(ws)) {
            const ptyProcess = sessions.get(ws);
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
      const ptyProcess = sessions.get(ws);
      console.log('Killing PTY process for closed connection');
      ptyProcess.kill();
      sessions.delete(ws);
      console.log('Remaining sessions:', sessions.size);
    }
  });
}

// Attach WebSocket handlers to both servers
wss.on('connection', handleWebSocketConnection);
wssHttps.on('connection', handleWebSocketConnection);

async function listTmuxSessions(ws) {
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

  listCmd.on('close', (code) => {
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

function attachToSession(ws, sessionName, cols = 120, rows = 40) {
  console.log(`Attaching to session '${sessionName}'`);
  
  // Check if we already have a PTY for this connection
  if (sessions.has(ws)) {
    const ptyProcess = sessions.get(ws);
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

function createNewPtySession(ws, sessionName, cols, rows) {
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
  ptyProcess.onData((data) => {
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

function listSessionWindows(ws, sessionName) {
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

function selectWindow(ws, sessionName, windowIndex) {
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

  const ptyProcess = sessions.get(ws);
  
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