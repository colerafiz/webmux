# Feature Request: Core WebMux Enhancements for Developer Workflow

## Summary
As a heavy TMUX/Claude Code user, WebMux has great potential as a web-based development environment. Here are essential features that would transform it into a complete "dev server/machine/webOS" while remaining easy to implement.

## Core Features Needed

### 1. Split Pane Support (Priority: HIGH)
- **Feature**: Native TMUX pane splitting in the web interface
- **Implementation**: Send TMUX split commands (`split-window -h/-v`) via existing WebSocket
- **UI**: Draggable pane dividers, keyboard shortcuts (Ctrl-A % and Ctrl-A ")

### 2. File Browser Integration (Priority: HIGH)
- **Feature**: VSCode-style file tree sidebar
- **Implementation**: Add WebSocket messages for `ls`, `tree` commands
- **Actions**: Click to open in editor, right-click context menu
- **Bonus**: Drag & drop file upload to current directory

### 3. Command Palette (Priority: HIGH)
- **Feature**: Quick command execution (like VSCode's Cmd+Shift+P)
- **Implementation**: Searchable list of common commands, recent history
- **Shortcuts**: Cmd/Ctrl+P for files, Cmd/Ctrl+Shift+P for commands

### 4. Clipboard Synchronization (Priority: HIGH)
- **Feature**: Seamless copy/paste between host and web client
- **Implementation**: Use Clipboard API for web, send via WebSocket
- **Security**: Permission prompt for clipboard access

### 5. Session Persistence & Recovery (Priority: MEDIUM)
- **Feature**: Auto-save session state, restore on reconnect
- **Implementation**: Store window/pane layout, working directories
- **UX**: "Restore previous session?" prompt on load

### 6. Terminal Themes & Customization (Priority: MEDIUM)
- **Feature**: Theme selector with popular schemes (Dracula, Nord, etc.)
- **Implementation**: xterm.js theme API, save preference to localStorage
- **Options**: Font size, cursor style, background opacity

### 7. Quick Actions Bar (Priority: MEDIUM)
- **Feature**: Customizable toolbar with common actions
- **Actions**: New window, split pane, kill pane, detach session
- **Implementation**: Floating or docked toolbar with icon buttons

### 8. File Transfer UI (Priority: MEDIUM)
- **Feature**: Upload/download files through the web interface
- **Implementation**: Drag & drop upload, click to download
- **Progress**: Show transfer progress for large files

### 9. Multi-Session View (Priority: LOW)
- **Feature**: Tab interface to switch between multiple attached sessions
- **Implementation**: Multiple WebSocket connections, tab UI
- **Memory**: Lazy loading of inactive sessions

### 10. Keyboard Shortcut Customization (Priority: LOW)
- **Feature**: Remap keyboard shortcuts to match local environment
- **Implementation**: Settings panel with key binding editor
- **Presets**: VSCode, Vim, Emacs, TMUX default modes

## Technical Considerations

### Easy Wins (Can implement quickly):
1. Command palette - mostly frontend work
2. Terminal themes - xterm.js already supports this
3. Quick actions bar - simple UI addition
4. Split pane support - TMUX commands already available

### Moderate Effort:
1. File browser - needs new WebSocket messages
2. Clipboard sync - browser API + WebSocket protocol
3. Session persistence - state management required

### Future Enhancements:
- Monaco editor integration for file editing
- Git status indicators in file browser
- Terminal broadcast mode (type in multiple panes)
- SSH key management UI
- Docker container status sidebar

## User Story
As a developer using Claude Code on a remote server, I want to:
1. Split my terminal to run multiple commands simultaneously
2. Browse and open files without leaving the web interface
3. Copy code from my local machine and paste into the terminal
4. Quickly access common commands without typing
5. Resume my work exactly where I left off after connection drops

## Success Metrics
- Reduced context switching between terminal and file browser
- Faster command execution via shortcuts and palette
- Improved workflow for remote development
- Feature parity with native terminal + TMUX experience

## References
- VSCode's integrated terminal as inspiration
- Terminator/iTerm2 for split pane UX
- TMUX native keybindings for muscle memory compatibility