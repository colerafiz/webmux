**Title:** [Feature Request] Essential Developer Features for WebMux as a Web-based DevOS

**Labels:** enhancement, feature-request, ux

**Description:**

As a TMUX/Claude Code power user, I see huge potential in WebMux becoming a complete web-based development environment. Here are the core features that would make it indispensable:

## 🎯 Top Priority Features

### 1. **Split Pane Support**
- Native TMUX pane splitting with visual dividers
- Keyboard shortcuts: `Ctrl-A %` (horizontal), `Ctrl-A "` (vertical)
- Draggable pane borders for resizing
- *Easy to implement: Just send TMUX split commands via WebSocket*

### 2. **Integrated File Browser**
- VSCode-style file tree in left sidebar
- Click files to `cat`/`less` them in terminal
- Right-click → Open with vim/nano/editor
- *Implementation: Add `ls`/`tree` WebSocket messages*

### 3. **Command Palette** 
- `Cmd/Ctrl+Shift+P` for quick command access
- Fuzzy search through command history
- Pin frequently used commands
- *Frontend-only feature using existing infrastructure*

### 4. **Web ↔ Host Clipboard Sync**
- Copy from terminal → paste locally
- Copy locally → paste in terminal
- *Use Clipboard API + WebSocket protocol*

## 🚀 Quick Wins (Low effort, high impact)

### 5. **Terminal Themes**
```javascript
// Already supported by xterm.js
terminal.setOption('theme', {
  background: '#1e1e2e',
  foreground: '#cdd6f4',
  // ... other colors
});
```

### 6. **Quick Actions Toolbar**
- Floating buttons: New Window | Split | Kill Pane | Detach
- Customizable position (top/bottom/floating)

### 7. **Drag & Drop File Upload**
- Drop files anywhere to upload to current directory
- Progress indicator for large files

## 💡 Game-Changing Features

### 8. **Session State Persistence**
- Auto-save: window layout, pane arrangement, working directories
- Restore prompt: "Resume previous session?"
- *Critical for connection drops*

### 9. **Multi-tab Sessions**
- Browser tabs → TMUX sessions
- Quick switch with `Ctrl+1-9`
- Visual indicators for activity

### 10. **Vim-style Keyboard Mode**
- Modal editing for terminal interaction
- Visual selection mode for copying
- Search mode with highlighting

## 📊 Why These Matter

**Current Pain Points:**
- No visual file browsing (constant `ls` commands)
- Can't split panes through UI
- Clipboard sync requires manual workarounds
- Lost work on connection drops
- No quick command access

**With These Features:**
- Full IDE-like experience in browser
- Faster development workflow
- Better than SSH + native terminal for many use cases
- True "development OS" in the browser

## 🔧 Implementation Priority

**Week 1:** Terminal themes, quick actions bar, command palette  
**Week 2:** Split pane support, basic file browser  
**Week 3:** Clipboard sync, file upload  
**Week 4:** Session persistence, polish  

---

*These features would make WebMux the go-to solution for remote development, especially for Claude Code users who need a powerful, accessible interface to their dev servers.*