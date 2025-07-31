let ws = null;
let terminal = null;
let currentSession = null;
let currentWindowIndex = null;

function initWebSocket() {
    ws = new WebSocket(`ws://${window.location.host}`);
    
    ws.onopen = () => {
        console.log('WebSocket connected');
        refreshSessions();
    };
    
    ws.onmessage = (event) => {
        const data = JSON.parse(event.data);
        
        switch (data.type) {
            case 'sessions-list':
                updateSessionsList(data.sessions);
                break;
            
            case 'output':
                if (terminal) {
                    terminal.write(data.data);
                }
                break;
            
            case 'disconnected':
                if (terminal) {
                    terminal.write('\r\n\r\n[Session disconnected]\r\n');
                }
                currentSession = null;
                currentWindowIndex = null;
                updateUI();
                break;
            
            case 'windows-list':
                updateWindowsList(data.windows);
                break;
            
            case 'window-selected':
                if (data.success) {
                    currentWindowIndex = data.windowIndex;
                    refreshWindows();
                }
                break;
        }
    };
    
    ws.onerror = (error) => {
        console.error('WebSocket error:', error);
    };
    
    ws.onclose = () => {
        console.log('WebSocket disconnected');
        setTimeout(initWebSocket, 3000);
    };
}

function initTerminal() {
    terminal = new Terminal({
        cursorBlink: true,
        fontSize: 14,
        fontFamily: 'Menlo, Monaco, "Courier New", monospace',
        theme: {
            background: '#000000',
            foreground: '#ffffff',
            cursor: '#ffffff',
            selection: 'rgba(255, 255, 255, 0.3)'
        }
    });
    
    terminal.open(document.getElementById('terminal'));
    
    terminal.onData((data) => {
        if (ws && ws.readyState === WebSocket.OPEN && currentSession) {
            ws.send(JSON.stringify({
                type: 'input',
                data: data
            }));
        }
    });
    
    window.addEventListener('resize', () => {
        if (terminal) {
            terminal.fit();
        }
    });
}

function refreshSessions() {
    if (ws && ws.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify({ type: 'list-sessions' }));
    }
}

function updateSessionsList(sessions) {
    const listEl = document.getElementById('sessions-list');
    listEl.innerHTML = '';
    
    if (sessions.length === 0) {
        listEl.innerHTML = '<p style="color: #666; text-align: center;">No TMUX sessions found</p>';
        return;
    }
    
    sessions.forEach(session => {
        const sessionEl = document.createElement('div');
        sessionEl.className = 'session-item';
        if (session.attached) {
            sessionEl.className += ' attached';
        }
        if (session.name === currentSession) {
            sessionEl.className += ' active';
        }
        
        const nameEl = document.createElement('div');
        nameEl.className = 'session-name';
        nameEl.textContent = session.name;
        
        const infoEl = document.createElement('div');
        infoEl.className = 'session-info';
        infoEl.textContent = session.attached ? 'Attached' : 'Detached';
        
        sessionEl.appendChild(nameEl);
        sessionEl.appendChild(infoEl);
        
        sessionEl.addEventListener('click', () => attachToSession(session.name));
        
        listEl.appendChild(sessionEl);
    });
}

function attachToSession(sessionName) {
    if (ws && ws.readyState === WebSocket.OPEN) {
        currentSession = sessionName;
        currentWindowIndex = null;
        terminal.clear();
        ws.send(JSON.stringify({
            type: 'attach-session',
            sessionName: sessionName
        }));
        updateUI();
        // Request windows list after attaching
        setTimeout(() => refreshWindows(), 500);
    }
}

function updateUI() {
    const noSessionEl = document.getElementById('no-session');
    const terminalEl = document.getElementById('terminal');
    const windowsSection = document.getElementById('windows-section');
    
    if (currentSession) {
        noSessionEl.style.display = 'none';
        terminalEl.style.display = 'block';
        windowsSection.style.display = 'block';
        terminal.focus();
    } else {
        noSessionEl.style.display = 'block';
        terminalEl.style.display = 'none';
        windowsSection.style.display = 'none';
    }
    
    document.querySelectorAll('.session-item').forEach(el => {
        const name = el.querySelector('.session-name').textContent;
        if (name === currentSession) {
            el.classList.add('active');
        } else {
            el.classList.remove('active');
        }
    });
}

document.addEventListener('DOMContentLoaded', () => {
    initTerminal();
    initWebSocket();
    
    document.getElementById('refresh-btn').addEventListener('click', refreshSessions);
    document.getElementById('refresh-windows-btn').addEventListener('click', refreshWindows);
});

document.addEventListener('keydown', (e) => {
    if (e.key === 'r' && e.metaKey) {
        e.preventDefault();
        refreshSessions();
    }
});

function refreshWindows() {
    if (ws && ws.readyState === WebSocket.OPEN && currentSession) {
        ws.send(JSON.stringify({ 
            type: 'list-windows',
            sessionName: currentSession
        }));
    }
}

function updateWindowsList(windows) {
    const listEl = document.getElementById('windows-list');
    listEl.innerHTML = '';
    
    if (windows.length === 0) {
        listEl.innerHTML = '<p style="color: #666; text-align: center;">No windows</p>';
        return;
    }
    
    windows.forEach(window => {
        const windowEl = document.createElement('div');
        windowEl.className = 'window-item';
        if (window.active) {
            windowEl.className += ' active';
            currentWindowIndex = window.index;
        }
        
        const indexEl = document.createElement('span');
        indexEl.className = 'window-index';
        indexEl.textContent = window.index + ':';
        
        const nameEl = document.createElement('span');
        nameEl.className = 'window-name';
        nameEl.textContent = window.name;
        
        windowEl.appendChild(indexEl);
        windowEl.appendChild(nameEl);
        
        windowEl.addEventListener('click', () => selectWindow(window.index));
        
        listEl.appendChild(windowEl);
    });
}

function selectWindow(windowIndex) {
    console.log('Selecting window:', windowIndex, 'in session:', currentSession);
    if (ws && ws.readyState === WebSocket.OPEN && currentSession) {
        ws.send(JSON.stringify({
            type: 'select-window',
            sessionName: currentSession,
            windowIndex: windowIndex
        }));
    } else {
        console.error('Cannot select window - WebSocket not ready or no session');
    }
}