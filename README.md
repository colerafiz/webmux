# WebMux

Web-based TMUX session viewer that allows users to interact with TMUX sessions through a browser interface. Now with PWA support for mobile devices!

## Features

- **Web-based Terminal**: Full terminal emulation in your browser using xterm.js
- **TMUX Session Management**: Create, attach, rename, and kill TMUX sessions
- **Window Management**: Create, switch, rename, and kill windows within sessions
- **Real-time Updates**: WebSocket-based communication for live terminal output
- **Responsive UI**: Clean, modern interface built with Vue 3 and Tailwind CSS
- **Performance Optimized**: Handles large outputs with buffering and flow control
- **PWA Support**: Install to home screen on mobile devices
- **HTTPS Enabled**: Secure connections with self-signed certificates
- **Mobile Optimized**: Touch-friendly interface with iOS safe area support
- **Network Accessible**: Access via Tailscale or local network IPs

## Prerequisites

- Node.js (v14 or higher)
- npm or yarn
- TMUX installed on your system
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

Run both the backend server and frontend client in development mode:
```bash
# HTTP mode
npm run dev

# HTTPS mode (required for PWA features)
npm run dev:https
```

This will start:
- Backend server on `http://localhost:3000` (HTTP) or `https://localhost:3443` (HTTPS)
- Frontend client on `http://localhost:5173` (HTTP) or `https://localhost:5173` (HTTPS)

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

Build the frontend for production:
```bash
npm run build
```

Preview the production build:
```bash
npm run preview
```

### Individual Components

Run only the backend server:
```bash
npm run server
```

Run only the frontend client:
```bash
npm run client
```

## Architecture

### Backend
- **Express.js** server with WebSocket support
- **node-pty** for pseudo-terminal creation and TMUX attachment
- **ws** library for WebSocket communication
- RESTful API for session and window management

### Frontend
- **Vue 3** with Composition API
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

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Built with [Vue.js](https://vuejs.org/)
- Terminal emulation by [xterm.js](https://xtermjs.org/)
- Styled with [Tailwind CSS](https://tailwindcss.com/)
- TMUX integration via [node-pty](https://github.com/microsoft/node-pty)