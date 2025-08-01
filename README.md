# WebMux

A high-performance web-based TMUX session viewer built with Rust and Vue.js. Access and control your TMUX sessions through a browser interface with full PWA support for mobile devices!

## Features

- **Web-based Terminal**: Full terminal emulation in your browser using xterm.js
- **TMUX Session Management**: Create, attach, rename, and kill TMUX sessions
- **Window Management**: Create, switch, rename, and kill windows within sessions
- **Real-time Updates**: WebSocket-based communication for live terminal output
- **Audio Streaming**: System audio capture and streaming (experimental)
- **Responsive UI**: Clean, modern interface built with Vue 3 and Tailwind CSS
- **Performance Optimized**: Handles large outputs with buffering and flow control
- **PWA Support**: Install to home screen on mobile devices
- **HTTPS Enabled**: Secure connections with self-signed certificates
- **Mobile Optimized**: Touch-friendly interface with iOS safe area support
- **Network Accessible**: Access via Tailscale or local network IPs
- **Rust Backend**: High-performance server built with Axum web framework

## Prerequisites

- Node.js (v14 or higher)
- npm or yarn
- Rust (latest stable version) - Install from [rustup.rs](https://rustup.rs/)
- cargo-watch (optional, for development) - Install with `cargo install cargo-watch`
- TMUX installed on your system
- ffmpeg (optional, for audio streaming)
- Modern web browser

## Installation

1. Clone the repository:
```bash
git clone https://github.com/colerafiz/webmux.git
cd webmux
```

2. Install dependencies:
```bash
npm install
```

## Usage

### Development

Run both the Rust backend server and Vue frontend client in development mode:
```bash
# HTTP mode
npm run dev

# HTTPS mode (required for PWA features)
npm run dev:https
```

This will start:
- Rust backend server on `http://localhost:4000` (HTTP) or `https://localhost:4443` (HTTPS) in development
- Frontend client on `http://localhost:5174` in development
- In production mode, servers run on ports 3000/3443 (backend) and 5173 (frontend)

### HTTPS Setup

Generate self-signed certificates for HTTPS:
```bash
npm run setup-certs
```

### Installing as PWA

#### iOS (iPhone/iPad)
1. Open Safari and navigate to the app (HTTPS required)
2. Tap the Share button (square with arrow)
3. Scroll down and tap "Add to Home Screen"
4. Name the app and tap "Add"
5. The app will now run fullscreen without browser UI

#### Android
1. Open Chrome and navigate to the app (HTTPS required)
2. Tap the menu (three dots)
3. Tap "Add to Home Screen" or "Install App"
4. Follow the prompts to install

#### Desktop Chrome
1. Look for the install icon in the address bar
2. Click "Install" when prompted

### Production

Build both backend and frontend for production:
```bash
npm run build
```

This will:
- Build the Rust backend with optimizations (`cargo build --release`)
- Build the Vue frontend for production

Preview the production build:
```bash
npm run preview
```

### Individual Components

Run only the Rust backend server:
```bash
npm run rust:dev  # With auto-restart on changes
# or
cd backend-rust && cargo run
```

Run only the frontend client:
```bash
npm run client
```

### Testing

Run Rust backend tests:
```bash
npm run rust:test
```

Type-check frontend:
```bash
npm run type-check
```

Lint frontend code:
```bash
npm run lint
```

## Architecture

### Backend (Rust)
- **Axum** web framework for high-performance async HTTP and WebSocket handling
- **tokio** async runtime for concurrent operations
- **portable-pty** for cross-platform pseudo-terminal creation and TMUX attachment
- **tokio-tungstenite** for WebSocket communication
- RESTful API for session and window management
- Alternative session manager using `tmux send-keys` and `capture-pane` to avoid attachment conflicts
- Audio streaming support via ffmpeg integration

### Frontend (Vue.js)
- **Vue 3** with Composition API and TypeScript
- **Vite** for fast development and optimized builds
- **xterm.js** for terminal emulation
- **Tailwind CSS** for styling
- **@tanstack/vue-query** for server state management

## API Reference

### REST Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/sessions` | List all TMUX sessions |
| POST | `/api/sessions` | Create new TMUX session |
| POST | `/api/sessions/:name/kill` | Kill a session |
| POST | `/api/sessions/:name/rename` | Rename a session |
| GET | `/api/sessions/:name/windows` | List windows in a session |
| POST | `/api/sessions/:name/windows` | Create new window |
| DELETE | `/api/sessions/:sessionName/windows/:windowIndex` | Kill a window |
| POST | `/api/sessions/:sessionName/windows/:windowIndex/rename` | Rename a window |
| POST | `/api/sessions/:sessionName/windows/:windowIndex/select` | Select a window |
| GET | `/api/stats` | System statistics |

### WebSocket Protocol

Client to Server messages:
```javascript
{ type: 'attach-session', sessionName, cols, rows }
{ type: 'input', data }
{ type: 'resize', cols, rows }
{ type: 'list-windows', sessionName }
{ type: 'select-window', sessionName, windowIndex }
```

Server to Client messages:
```javascript
{ type: 'output', data }
{ type: 'attached', sessionName }
{ type: 'disconnected' }
{ type: 'windows-list', windows }
```

## Troubleshooting

### Keyboard input not working
Click anywhere in the terminal area to ensure it has focus.

### Session not responding
Refresh the page and re-select the session from the list.

### Window switching fails
Ensure you're attached to the session first before attempting to switch windows.

### Terminal freezes with large output
The system includes output buffering and flow control to handle applications that produce lots of output. Check the browser console for debug logs if issues persist.

### Issues with interactive applications (like Claude Code)
If experiencing conflicts with interactive applications, the backend includes an alternative session manager that uses `tmux send-keys` and `capture-pane` commands instead of direct attachment. This can be found in `backend-rust/src/websocket/session_manager.rs`.

### Enable debug logging
Set the `RUST_LOG=debug` environment variable when running the backend for detailed logging.

### Audio streaming issues
Run the backend with the `--audio` flag to enable audio streaming debug logs. Ensure ffmpeg is installed on your system.

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Backend built with [Rust](https://www.rust-lang.org/) and [Axum](https://github.com/tokio-rs/axum)
- Frontend built with [Vue.js](https://vuejs.org/)
- Terminal emulation by [xterm.js](https://xtermjs.org/)
- Styled with [Tailwind CSS](https://tailwindcss.com/)
- TMUX integration via [portable-pty](https://github.com/wez/portable-pty)
- Real-time communication with [tokio-tungstenite](https://github.com/snapview/tokio-tungstenite)