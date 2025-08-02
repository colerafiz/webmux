# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

WebMux is a Progressive Web App (PWA) that provides a web-based TMUX session viewer, allowing users to interact with TMUX sessions through a browser interface. It consists of a Rust backend server and a Vue 3 frontend application with full mobile support and installability.

## Recent Changes

- **Complete WebSocket Migration**: Removed all REST endpoints - everything now uses WebSocket for real-time communication
- **Unified Communication**: All TMUX operations (sessions, windows, terminal I/O) go through a single WebSocket connection
- **Real-time Updates**: Added TMUX monitor for automatic state synchronization across all clients
- **Search Feature**: Added centered search bar for quick window navigation
- **Session Isolation**: Fixed issues with alternative session manager
- **Performance**: Improved output buffering and flow control for large outputs
- **Audio Streaming**: Added experimental system audio capture via ffmpeg

## Common Commands

### Development
- **Run the development environment**: `npm run dev` (starts both Rust backend and frontend client concurrently)
- **Run with HTTPS**: `npm run dev:https` (starts both servers with HTTPS enabled)
- **Backend server only**: `npm run rust:dev` (runs with cargo-watch for auto-restart)
- **Frontend client only**: `npm run client` (runs Vite dev server)
- **Build for production**: `npm run build` (builds both Rust backend and frontend)
- **Preview production build**: `npm run preview`
- **Build Rust backend**: `npm run rust:build` (creates optimized release binary)
- **Check Rust code**: `npm run rust:check` (runs cargo check)
- **Test Rust backend**: `npm run rust:test` (runs cargo test)

### Requirements
- **Rust**: Install from https://rustup.rs/
- **cargo-watch**: Install with `cargo install cargo-watch` for auto-restart during development

### HTTPS Setup
WebMux supports HTTPS with self-signed certificates:
- **Generate certificates**: `npm run setup-certs` (creates self-signed SSL certificates)
- **HTTPS ports**: Backend runs on port 3443 (HTTPS) and 3000 (HTTP), Frontend on 5173 (HTTPS)
- **Accept certificate**: You'll need to accept the self-signed certificate in your browser
- **Mobile compatibility**: HTTPS is required for many mobile features and secure connections

### Network Access
The application is configured to accept connections from any network interface:
- **Local HTTP**: `http://localhost:5174` (dev frontend) / `http://localhost:4000` (dev backend)
- **Local HTTPS**: `https://localhost:5174` (dev frontend) / `https://localhost:4443` (dev backend)
- **Production ports**: 5173 (frontend) / 3000 (backend HTTP) / 3443 (backend HTTPS)
- **Network access**: Use your machine's IP address (e.g., `https://192.168.1.100:5174`)
- **Tailscale access**: Use your machine's Tailscale IP (e.g., `https://100.x.x.x:5174`)

Both servers bind to `0.0.0.0`, which means they accept connections from all network interfaces.

### Installation
- **Install dependencies**: `npm install`

### Linting & Type Checking
- **Lint frontend code**: `npm run lint` (runs ESLint on .js,.jsx,.ts,.tsx,.vue files)
- **Type check frontend**: `npm run type-check` (runs vue-tsc --noEmit)

## Architecture

### Backend (Rust + Axum)
- **Main server**: `backend-rust/src/main.rs` - Axum server with WebSocket support for terminal sessions
- **TMUX handler**: `backend-rust/src/tmux/mod.rs` - Dedicated TMUX command handling logic
- **Type definitions**: `backend-rust/src/types/mod.rs` - Rust types for backend
- **WebSocket handler**: `backend-rust/src/websocket/mod.rs` - WebSocket connection management and message handling
- **Session manager**: `backend-rust/src/websocket/session_manager.rs` - Alternative TMUX session management using `send-keys` and `capture-pane` to avoid direct attachment conflicts
- **WebSocket protocol**: Uses `axum::ws` and `tokio-tungstenite` for real-time communication
- **Terminal emulation**: Uses `portable-pty` for cross-platform pseudo-terminal creation and TMUX attachment
- **Audio streaming**: `backend-rust/src/audio/mod.rs` - System audio capture and streaming via ffmpeg

### Frontend (Vue 3 + Vite + TypeScript)
- **Entry point**: `src/main.ts` - Vue app initialization with Vue Query
- **Main component**: `src/App.vue` - Root application component with TypeScript
- **Components** (all using TypeScript):
  - `SessionList.vue` - Displays available TMUX sessions
  - `SessionItem.vue` - Individual session item in the list
  - `TerminalView.vue` - Terminal emulator view using xterm.js
  - `WindowList.vue` - TMUX window management
  - `SearchBar.vue` - Quick window search and navigation
- **Composables**: 
  - `useWebSocket.ts` - WebSocket connection management with types
  - `useWindowSearch.ts` - Window search functionality
- **Type definitions**: `src/types/index.ts` - Shared TypeScript types

### Key Technologies
- **Frontend framework**: Vue 3 with Composition API
- **Build tool**: Vite
- **State management**: @tanstack/vue-query for server state
- **Terminal emulator**: @xterm/xterm with fit addon
- **Styling**: Tailwind CSS
- **Backend runtime**: Rust with Axum web framework
- **Real-time communication**: WebSocket (axum::ws)
- **Terminal interface**: portable-pty for cross-platform pseudo-terminal support
- **Audio streaming**: ffmpeg for system audio capture

## WebSocket API

All communication happens through WebSocket connections at `/ws`. There are no REST endpoints - everything is handled via real-time WebSocket messages.

### WebSocket Messages
**Client → Server:**
- Session Management:
  - `{ type: 'list-sessions' }`
  - `{ type: 'create-session', name }`
  - `{ type: 'attach-session', sessionName, cols, rows }`
  - `{ type: 'kill-session', sessionName }`
  - `{ type: 'rename-session', sessionName, newName }`
- Terminal I/O:
  - `{ type: 'input', data }`
  - `{ type: 'resize', cols, rows }`
- Window Management:
  - `{ type: 'list-windows', sessionName }`
  - `{ type: 'create-window', sessionName, windowName? }`
  - `{ type: 'select-window', sessionName, windowIndex }`
  - `{ type: 'kill-window', sessionName, windowIndex }`
  - `{ type: 'rename-window', sessionName, windowIndex, newName }`
- Audio Streaming:
  - `{ type: 'start-audio' }`
  - `{ type: 'stop-audio' }`

**Server → Client:**
- Session Updates:
  - `{ type: 'sessions-list', sessions }`
  - `{ type: 'session-created', session }`
  - `{ type: 'session-killed', sessionName }`
  - `{ type: 'session-renamed', oldName, newName }`
  - `{ type: 'attached', sessionName }`
  - `{ type: 'disconnected' }`
- Terminal Output:
  - `{ type: 'output', data }`
- Window Updates:
  - `{ type: 'windows-list', windows }`
  - `{ type: 'window-created', window }`
  - `{ type: 'window-selected', windowIndex }`
  - `{ type: 'window-killed', windowIndex }`
  - `{ type: 'window-renamed', windowIndex, newName }`
- Audio Streaming:
  - `{ type: 'audio-data', data }` - Base64 encoded audio chunks
  - `{ type: 'audio-status', streaming, error }`
- Real-time Monitoring:
  - `{ type: 'tmux-update', event }` - Real-time TMUX state changes

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
- **Terminal freezes with large output**: The system now has output buffering and flow control to handle tools like Claude Code that produce lots of output

## Performance Notes

The system includes several optimizations for handling large terminal outputs:
- **Server-side buffering**: PTY output is buffered and sent in chunks to prevent WebSocket overflow
- **Flow control**: PTY is paused if WebSocket buffer becomes full, preventing memory issues
- **Client-side buffering**: Terminal writes are batched for smoother rendering
- **Debug logging**: High data rate situations are logged to help identify performance issues

## Development Notes

### Port Configuration
During development, the application uses different ports to avoid conflicts:
- **Frontend dev server**: Port 5174 (instead of default 5173)
- **Backend dev server**: Port 4000 (HTTP) / 4443 (HTTPS)
- **Production ports**: 5173 (frontend) / 3000 (HTTP) / 3443 (HTTPS)

### WebSocket & Audio Streaming
- **WebSocket endpoint**: `/ws` - Terminal session management
- **Audio streaming**: The backend includes audio capture capabilities via ffmpeg
- **Audio debug logs**: Run backend with `--audio` flag to enable audio streaming debug logs

### Project Migration History
The backend was recently migrated from TypeScript/Express to Rust/Axum for better performance and type safety. Legacy TypeScript backend commands are preserved in package.json with "old:" prefix.

### Running Tests
To run the Rust backend tests:
```bash
npm run rust:test
```

### Best Practices Document
The project contains a detailed best practices document (`tmux-web-terminal-best-practices.md`) that outlines:
- Current implementation issues with direct TMUX attachment
- Alternative approaches using `capture-pane` and `pipe-pane`
- Recommended hybrid approach for better session management
- Security and performance considerations

Key implementation considerations:
- The current implementation uses `tmux attach-session` directly which can cause conflicts with multiple clients
- An alternative session manager implementation exists in `backend-rust/src/websocket/session_manager.rs` that uses `send-keys` and `capture-pane` commands to avoid attachment conflicts
- Consider implementing the improved patterns outlined in the best practices document
- WebSocket connections are managed per client with individual PTY processes
- TMUX prefix key is set to Ctrl-A (0x01) for window switching

### Debugging Tips
- **Enable debug logs**: Set `RUST_LOG=debug` environment variable
- **Audio streaming debug**: Run backend with `--audio` flag
- **Check WebSocket messages**: Use browser developer tools to monitor WebSocket frames
- **TMUX session conflicts**: If experiencing issues with interactive applications (like Claude Code), consider using the alternative session manager approach

### Key Files to Know
- **WebSocket handler**: `backend-rust/src/websocket/mod.rs` - Main WebSocket logic
- **Terminal view**: `src/components/TerminalView.vue` - Terminal UI component
- **Window search**: `src/components/SearchBar.vue` - Quick window search feature
- **Session types**: `src/types/index.ts` - TypeScript type definitions

### Common Development Tasks

**Adding a new WebSocket message type:**
1. Add the message type to both client and server type definitions
2. Update WebSocket handler in `backend-rust/src/websocket/mod.rs`
3. Update client handler in `src/composables/useWebSocket.ts`

**Modifying terminal behavior:**
1. Check `src/components/TerminalView.vue` for UI changes
2. Check `backend-rust/src/websocket/mod.rs` for server-side handling
3. Test with both small and large outputs

**Working with TMUX commands:**
1. TMUX logic is in `backend-rust/src/tmux/mod.rs`
2. Use `tmux list-sessions -F` for structured output
3. Remember TMUX prefix is Ctrl-A (0x01)