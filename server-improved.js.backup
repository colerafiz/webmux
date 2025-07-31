const express = require('express');
const WebSocket = require('ws');
const { spawn } = require('node-pty');
const path = require('path');
const cors = require('cors');
const { v4: uuidv4 } = require('uuid');

const app = express();
const port = 3000;

app.use(cors());
app.use(express.json());
app.use(express.static('public'));

// Session manager for better tmux interaction
class TmuxSessionManager {
  constructor() {
    this.sessions = new Map();
    this.clients = new Map();
  }

  async executeCommand(args) {
    return new Promise((resolve, reject) => {
      const cmd = spawn('tmux', args, {
        name: 'xterm-color',
        cols: 80,
        rows: 30
      });

      let output = '';
      let error = '';

      cmd.on('data', (data) => {
        output += data;
      });

      cmd.on('exit', (code) => {
        if (code === 0) {
          resolve({ success: true, output });
        } else {
          reject({ success: false, error: error || 'Command failed' });
        }
      });
    });
  }

  async listSessions() {
    try {
      const result = await this.executeCommand([
        'list-sessions', 
        '-F', 
        '#{session_name}:#{session_attached}:#{session_created}:#{session_windows}:#{session_width}x#{session_height}'
      ]);

      const sessions = result.output.trim().split('\n').filter(line => line)
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

      return sessions;
    } catch (err) {
      return [];
    }
  }

  async createSession(name) {
    const sessionName = name || `session-${Date.now()}`;
    await this.executeCommand(['new-session', '-d', '-s', sessionName]);
    return sessionName;
  }

  async killSession(name) {
    await this.executeCommand(['kill-session', '-t', name]);
  }

  async capturePane(sessionName, pane = '0') {
    try {
      const result = await this.executeCommand([
        'capture-pane',
        '-t', `${sessionName}:${pane}`,
        '-p',  // Print to stdout
        '-e',  // Include escape sequences
        '-S', '-'  // Start from beginning
      ]);
      return result.output;
    } catch (err) {
      return '';
    }
  }

  async sendKeys(sessionName, keys) {
    await this.executeCommand(['send-keys', '-t', sessionName, keys]);
  }

  async sendCommand(sessionName, command) {
    await this.executeCommand(['send-keys', '-t', sessionName, command, 'Enter']);
  }

  // Create a monitoring session that watches a target session
  async createMonitoringSession(clientId, targetSession) {
    const monitoringPty = spawn('bash', [], {
      name: 'xterm-256color',
      cols: 120,
      rows: 40,
      cwd: process.env.HOME,
      env: {
        ...process.env,
        TERM: 'xterm-256color'
      }
    });

    // Set up periodic capture of the target session
    const captureInterval = setInterval(async () => {
      try {
        const content = await this.capturePane(targetSession);
        const client = this.clients.get(clientId);
        if (client && client.ws.readyState === WebSocket.OPEN) {
          // Clear screen and write captured content
          monitoringPty.write('\x1b[2J\x1b[H'); // Clear screen and move to top
          monitoringPty.write(content);
        } else {
          clearInterval(captureInterval);
          monitoringPty.kill();
        }
      } catch (err) {
        console.error('Error capturing pane:', err);
      }
    }, 1000); // Update every second

    return { pty: monitoringPty, interval: captureInterval };
  }
}

const sessionManager = new TmuxSessionManager();

// REST API endpoints
app.get('/api/sessions', async (req, res) => {
  const sessions = await sessionManager.listSessions();
  res.json({ sessions });
});

app.post('/api/sessions/:name/kill', async (req, res) => {
  const { name } = req.params;
  try {
    await sessionManager.killSession(name);
    res.json({ success: true });
  } catch (err) {
    res.status(400).json({ success: false, error: err.message });
  }
});

app.post('/api/sessions/:name/rename', async (req, res) => {
  const { name } = req.params;
  const { newName } = req.body;
  
  try {
    await sessionManager.executeCommand(['rename-session', '-t', name, newName]);
    res.json({ success: true });
  } catch (err) {
    res.status(400).json({ success: false, error: err.message });
  }
});

app.post('/api/sessions', async (req, res) => {
  const { name } = req.body;
  try {
    const sessionName = await sessionManager.createSession(name);
    res.json({ success: true, sessionName });
  } catch (err) {
    res.status(400).json({ success: false, error: err.message });
  }
});

const server = app.listen(port, () => {
  console.log(`WebMux server (improved) running at http://localhost:${port}`);
});

const wss = new WebSocket.Server({ server, path: '/ws' });

wss.on('connection', (ws) => {
  const clientId = uuidv4();
  const client = {
    id: clientId,
    ws: ws,
    targetSession: null,
    pty: null,
    captureInterval: null,
    inputBuffer: [],
    inputProcessing: false,
    mode: 'view' // 'view' or 'interact'
  };

  sessionManager.clients.set(clientId, client);
  console.log(`New WebSocket connection: ${clientId}`);

  ws.on('message', async (message) => {
    try {
      const data = JSON.parse(message);
      
      switch (data.type) {
        case 'list-sessions':
          const sessions = await sessionManager.listSessions();
          ws.send(JSON.stringify({
            type: 'sessions-list',
            sessions
          }));
          break;
        
        case 'view-session':
          await handleViewSession(client, data.sessionName, data.cols, data.rows);
          break;

        case 'interact-session':
          await handleInteractSession(client, data.sessionName, data.cols, data.rows);
          break;
        
        case 'input':
          if (client.mode === 'interact' && client.targetSession) {
            client.inputBuffer.push(data.data);
            processInputBuffer(client);
          }
          break;
          
        case 'resize':
          if (client.pty) {
            client.pty.resize(data.cols, data.rows);
          }
          break;

        case 'detach':
          detachClient(client);
          break;
      }
    } catch (err) {
      console.error('Error handling message:', err);
      ws.send(JSON.stringify({
        type: 'error',
        message: err.message
      }));
    }
  });

  ws.on('close', () => {
    detachClient(client);
    sessionManager.clients.delete(clientId);
  });
});

async function handleViewSession(client, sessionName, cols, rows) {
  // Clean up any existing session
  detachClient(client);

  client.targetSession = sessionName;
  client.mode = 'view';

  // Send initial content
  const content = await sessionManager.capturePane(sessionName);
  client.ws.send(JSON.stringify({
    type: 'session-content',
    content,
    mode: 'view'
  }));

  // Set up live monitoring
  const { pty, interval } = await sessionManager.createMonitoringSession(client.id, sessionName);
  client.pty = pty;
  client.captureInterval = interval;

  pty.on('data', (data) => {
    if (client.ws.readyState === WebSocket.OPEN) {
      client.ws.send(JSON.stringify({
        type: 'output',
        data: data
      }));
    }
  });

  client.ws.send(JSON.stringify({
    type: 'attached',
    sessionName,
    mode: 'view'
  }));
}

async function handleInteractSession(client, sessionName, cols, rows) {
  // For interaction mode, we create a dedicated tmux client session
  detachClient(client);

  client.targetSession = sessionName;
  client.mode = 'interact';

  // Create a new tmux client that attaches to the target session
  const clientSessionName = `client-${client.id}`;
  
  try {
    // Create a new session that shares windows with target
    await sessionManager.executeCommand([
      'new-session', '-d', '-s', clientSessionName,
      '-t', sessionName
    ]);

    // Now attach to this client session
    const pty = spawn('tmux', ['attach-session', '-t', clientSessionName], {
      name: 'xterm-256color',
      cols: cols,
      rows: rows,
      cwd: process.env.HOME,
      env: {
        ...process.env,
        TERM: 'xterm-256color'
      }
    });

    client.pty = pty;
    client.clientSession = clientSessionName;

    pty.on('data', (data) => {
      if (client.ws.readyState === WebSocket.OPEN) {
        client.ws.send(JSON.stringify({
          type: 'output',
          data: data
        }));
      }
    });

    pty.on('exit', async () => {
      // Clean up client session
      try {
        await sessionManager.killSession(clientSessionName);
      } catch (err) {
        // Session might already be gone
      }

      if (client.ws.readyState === WebSocket.OPEN) {
        client.ws.send(JSON.stringify({
          type: 'disconnected'
        }));
      }
    });

    client.ws.send(JSON.stringify({
      type: 'attached',
      sessionName,
      mode: 'interact'
    }));

  } catch (err) {
    client.ws.send(JSON.stringify({
      type: 'error',
      message: `Failed to create interactive session: ${err.message}`
    }));
  }
}

async function processInputBuffer(client) {
  if (client.inputProcessing || client.inputBuffer.length === 0) return;

  client.inputProcessing = true;
  const input = client.inputBuffer.shift();

  try {
    if (client.mode === 'interact' && client.pty) {
      // Direct PTY write for interactive mode
      client.pty.write(input);
    } else if (client.targetSession) {
      // Use send-keys for view mode (if we want to allow limited input)
      await sessionManager.sendKeys(client.targetSession, input);
    }
  } catch (err) {
    console.error('Error processing input:', err);
  }

  // Process next input after a small delay
  setTimeout(() => {
    client.inputProcessing = false;
    processInputBuffer(client);
  }, 10);
}

function detachClient(client) {
  if (client.captureInterval) {
    clearInterval(client.captureInterval);
    client.captureInterval = null;
  }

  if (client.pty) {
    client.pty.kill();
    client.pty = null;
  }

  if (client.clientSession) {
    sessionManager.killSession(client.clientSession).catch(() => {});
    client.clientSession = null;
  }

  client.targetSession = null;
  client.inputBuffer = [];
}