# Development Branch Configuration

This is the development branch of WebMux, configured to run on different ports to allow simultaneous operation with the main branch.

## Port Configuration

### Development Branch Ports:
- Backend HTTP: **4000** (main uses 3000)
- Backend HTTPS: **4443** (main uses 3443)
- Frontend: **5174** (main uses 5173)

### Main Branch Ports:
- Backend HTTP: 3000
- Backend HTTPS: 3443
- Frontend: 5173

## Running Both Branches Simultaneously

1. **In the main branch** (`/home/cyrus/git/swve/webmux`):
   ```bash
   npm run dev
   ```
   Access at: https://localhost:5173

2. **In the development branch** (`/home/cyrus/git/swve/webmux-dev`):
   ```bash
   npm run dev
   ```
   Access at: https://localhost:5174

## Changes Made for Port Separation

1. **server.ts**: Updated ports from 3000/3443 to 4000/4443
2. **vite.config.ts**: 
   - Updated dev server port from 5173 to 5174
   - Updated proxy targets to use 4443 instead of 3443

## Notes

- Both branches can run simultaneously without conflicts
- Ensure you have separate terminal sessions for each branch
- The WebSocket connections are automatically routed through Vite's proxy