# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

WebMux is a web-based TMUX session viewer that allows users to interact with TMUX sessions through a browser interface. It consists of a Node.js backend server and a Vue 3 frontend application.

## Common Commands

### Development
- **Run the development environment**: `npm run dev` (starts both backend server and frontend client concurrently)
- **Backend server only**: `npm run server` (runs with nodemon for auto-restart)
- **Frontend client only**: `npm run client` (runs Vite dev server)
- **Build for production**: `npm run build`
- **Preview production build**: `npm run preview`

### Installation
- **Install dependencies**: `npm install`

## Architecture

### Backend (Node.js + Express)
- **Main server**: `server.js` - Express server with WebSocket support for terminal sessions
- **Alternative implementation**: `server-improved.js` - Contains improved session handling
- **TMUX handler**: `tmux-handler.js` - Dedicated TMUX command handling logic
- **WebSocket protocol**: Uses `ws` library for real-time communication
- **Terminal emulation**: Uses `node-pty` for pseudo-terminal creation and TMUX attachment

### Frontend (Vue 3 + Vite)
- **Entry point**: `src/main.js` - Vue app initialization with Vue Query
- **Main component**: `src/App.vue` - Root application component
- **Components**:
  - `SessionList.vue` - Displays available TMUX sessions
  - `SessionItem.vue` - Individual session item in the list
  - `TerminalView.vue` - Terminal emulator view using xterm.js
  - `WindowList.vue` - TMUX window management
- **Composables**: `useWebSocket.js` - WebSocket connection management
- **API**: `src/api/tmux.js` - REST API client for TMUX operations

### Key Technologies
- **Frontend framework**: Vue 3 with Composition API
- **Build tool**: Vite
- **State management**: @tanstack/vue-query for server state
- **Terminal emulator**: @xterm/xterm with fit addon
- **Styling**: Tailwind CSS
- **Backend runtime**: Node.js with Express
- **Real-time communication**: WebSocket (ws library)
- **Terminal interface**: node-pty for pseudo-terminal support

## API Endpoints

### REST API
- `GET /api/sessions` - List all TMUX sessions
- `POST /api/sessions` - Create new TMUX session
- `POST /api/sessions/:name/kill` - Kill a session
- `POST /api/sessions/:name/rename` - Rename a session
- `GET /api/sessions/:name/windows` - List windows in a session
- `POST /api/sessions/:name/windows` - Create new window
- `DELETE /api/sessions/:sessionName/windows/:windowIndex` - Kill a window
- `POST /api/sessions/:sessionName/windows/:windowIndex/rename` - Rename a window
- `POST /api/sessions/:sessionName/windows/:windowIndex/select` - Select a window
- `GET /api/stats` - System statistics

### WebSocket Messages
- Client → Server:
  - `{ type: 'attach-session', sessionName, cols, rows }`
  - `{ type: 'input', data }`
  - `{ type: 'resize', cols, rows }`
  - `{ type: 'list-windows', sessionName }`
  - `{ type: 'select-window', sessionName, windowIndex }`
- Server → Client:
  - `{ type: 'output', data }`
  - `{ type: 'attached', sessionName }`
  - `{ type: 'disconnected' }`
  - `{ type: 'windows-list', windows }`

## Testing & Debugging

When debugging terminal input issues:
1. Open browser console (F12) to see debug logs
2. Check that WebSocket is connected (look for "WebSocket connected" message)
3. Verify session is attached (look for "Session attached: [name]" message)
4. Ensure terminal has focus (clicking anywhere in terminal area should focus it)

Common issues and solutions:
- **Keyboard input not working**: Click in the terminal area to focus it
- **Session not responding**: Refresh the page and re-select the session
- **Window switching fails**: Ensure you're attached to the session first

## Development Notes

The project contains a detailed best practices document (`tmux-web-terminal-best-practices.md`) that outlines:
- Current implementation issues with direct TMUX attachment
- Alternative approaches using `capture-pane` and `pipe-pane`
- Recommended hybrid approach for better session management
- Security and performance considerations

Key implementation considerations:
- The current implementation uses `tmux attach-session` directly which can cause conflicts with multiple clients
- Consider implementing the improved patterns outlined in the best practices document
- WebSocket connections are managed per client with individual PTY processes
- TMUX prefix key is set to Ctrl-A (0x01) for window switching