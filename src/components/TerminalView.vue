<template>
  <div class="h-full flex flex-col">
    <div class="px-3 py-2 flex-shrink-0 border-b" 
         style="background: var(--bg-secondary); border-color: var(--border-primary)">
      <div class="flex items-center justify-between">
        <div class="flex items-center space-x-3 text-xs">
          <span style="color: var(--text-tertiary)">Session:</span>
          <span style="color: var(--text-primary)" class="font-medium">{{ session }}</span>
        </div>
        <div class="text-xs" style="color: var(--text-tertiary)">
          <span>{{ terminalSize.cols }}×{{ terminalSize.rows }}</span>
        </div>
      </div>
    </div>
    
    <!-- Terminal area with mobile controls -->
    <div class="flex-1 relative overflow-hidden">
      <!-- Mobile control bar - fixed at top of terminal area -->
      <div v-if="isMobile" class="absolute top-0 left-0 right-0 z-20 px-2 py-1.5 border-b overflow-x-auto mobile-controls-scrollbar shadow-md" 
           style="background: var(--bg-secondary); border-color: var(--border-primary); pointer-events: auto;">
      <div class="flex space-x-1 text-xs whitespace-nowrap">
        <button 
          @click="sendKey('Escape')" 
          class="px-3 py-1.5 rounded hover-bg"
          style="background: var(--bg-tertiary); color: var(--text-primary)"
        >
          ESC
        </button>
        <button 
          @click="sendKey('Tab')" 
          class="px-3 py-1.5 rounded hover-bg"
          style="background: var(--bg-tertiary); color: var(--text-primary)"
        >
          TAB
        </button>
        <button 
          @click="toggleCtrl" 
          :class="ctrlPressed ? 'bg-green-600' : ''"
          class="px-3 py-1.5 rounded hover-bg"
          :style="ctrlPressed ? 'background: #10b981; color: white' : 'background: var(--bg-tertiary); color: var(--text-primary)'"
        >
          CTRL {{ ctrlPressed ? '●' : '' }}
        </button>
        <button 
          @click="sendKey('ArrowUp')" 
          class="px-3 py-1.5 rounded hover-bg"
          style="background: var(--bg-tertiary); color: var(--text-primary)"
        >
          ↑
        </button>
        <button 
          @click="sendKey('ArrowDown')" 
          class="px-3 py-1.5 rounded hover-bg"
          style="background: var(--bg-tertiary); color: var(--text-primary)"
        >
          ↓
        </button>
        <button 
          @click="sendKey('ArrowLeft')" 
          class="px-3 py-1.5 rounded hover-bg"
          style="background: var(--bg-tertiary); color: var(--text-primary)"
        >
          ←
        </button>
        <button 
          @click="sendKey('ArrowRight')" 
          class="px-3 py-1.5 rounded hover-bg"
          style="background: var(--bg-tertiary); color: var(--text-primary)"
        >
          →
        </button>
        <button 
          @click="sendCtrlKey('c')" 
          class="px-3 py-1.5 rounded hover-bg"
          style="background: var(--bg-tertiary); color: var(--text-primary)"
        >
          ^C
        </button>
        <button 
          @click="sendCtrlKey('d')" 
          class="px-3 py-1.5 rounded hover-bg"
          style="background: var(--bg-tertiary); color: var(--text-primary)"
        >
          ^D
        </button>
        <button 
          @click="sendCtrlKey('z')" 
          class="px-3 py-1.5 rounded hover-bg"
          style="background: var(--bg-tertiary); color: var(--text-primary)"
        >
          ^Z
        </button>
        <button 
          @click="sendCtrlKey('a')" 
          class="px-3 py-1.5 rounded hover-bg"
          style="background: var(--bg-tertiary); color: var(--text-primary)"
        >
          ^A
        </button>
      </div>
    </div>
    
    <!-- Terminal container -->
    <div 
      ref="terminalContainer" 
      class="absolute inset-0 overflow-hidden touch-manipulation z-10" 
      tabindex="0" 
      :style="`background: #000; -webkit-user-select: none; user-select: none; ${isMobile ? 'padding-top: 48px;' : ''}`" 
      @click="focusTerminal"
      @touchstart="handleTouchStart"
      @touchend="handleTouchEnd"
    ></div>
  </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, computed } from 'vue'
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import '@xterm/xterm/css/xterm.css'
import type { TerminalSize, OutputMessage, AttachSessionMessage, ResizeMessage, InputMessage } from '@/types'
import type { UseWebSocketReturn } from '@/composables/useWebSocket'

interface Props {
  session: string
  ws: UseWebSocketReturn
}

const props = defineProps<Props>()

const terminalContainer = ref<HTMLDivElement | null>(null)
let terminal: Terminal | null = null
let fitAddon: FitAddon | null = null
// Removed unused focusInterval
const terminalSize = ref<TerminalSize>({ cols: 80, rows: 24 })
const ctrlPressed = ref<boolean>(false)
const isMobile = computed(() => window.innerWidth < 768)

onMounted(() => {
  terminal = new Terminal({
    cursorBlink: true,
    fontSize: 13,
    fontFamily: 'JetBrains Mono, SF Mono, Monaco, Inconsolata, Fira Code, monospace',
    theme: {
      background: '#000000',
      foreground: '#c9d1d9',
      cursor: '#c9d1d9',
      cursorAccent: '#000000',
      selectionBackground: 'rgba(88, 166, 255, 0.3)',
      black: '#000000',
      red: '#ff7b72',
      green: '#7ee787',
      yellow: '#ffa657',
      blue: '#79c0ff',
      magenta: '#d2a8ff',
      cyan: '#a5d6ff',
      white: '#c9d1d9',
      brightBlack: '#6e7681',
      brightRed: '#ffa198',
      brightGreen: '#56d364',
      brightYellow: '#ffdf5d',
      brightBlue: '#79c0ff',
      brightMagenta: '#d2a8ff',
      brightCyan: '#a5d6ff',
      brightWhite: '#ffffff'
    },
    scrollback: 10000,
    tabStopWidth: 8,
    // @ts-ignore - bellStyle is a valid option but not in types
    bellStyle: 'none',
    drawBoldTextInBrightColors: true,
    lineHeight: 1.2
  })

  fitAddon = new FitAddon()
  terminal.loadAddon(fitAddon)
  
  if (terminalContainer.value) {
    terminal.open(terminalContainer.value)
  }
  
  // Initial fit with a small delay to ensure container is properly sized
  setTimeout(() => {
    if (fitAddon) fitAddon.fit()
    if (terminal) terminal.focus()
  }, 100)

  if (terminal) {
    terminal.onData((data) => {
      if (props.ws.isConnected.value) {
        // If CTRL is toggled on mobile, modify the input
        if (ctrlPressed.value && data.length === 1) {
          const code = data.toUpperCase().charCodeAt(0) - 64
          data = String.fromCharCode(code)
          ctrlPressed.value = false // Auto-release after use
        }
        
        const message: InputMessage = {
          type: 'input',
          data: data
        }
        props.ws.send(message)
      }
    })
  }

  if (terminal) {
    terminal.onResize((size) => {
      terminalSize.value = { cols: size.cols, rows: size.rows }
      if (props.ws.isConnected.value) {
        const message: ResizeMessage = {
          type: 'resize',
          cols: size.cols,
          rows: size.rows
        }
        props.ws.send(message)
      }
    })
  }

  // Direct terminal writing - no client buffering to avoid freeze issues
  props.ws.onMessage<OutputMessage>('output', (data) => {
    if (terminal) {
      try {
        terminal.write(data.data)
      } catch (err) {
        console.warn('Error writing to terminal:', err)
        // If terminal write fails, try to recover
        setTimeout(() => {
          if (terminal) {
            try {
              terminal.write(data.data)
            } catch (retryErr) {
              console.error('Terminal write retry failed:', retryErr)
            }
          }
        }, 100)
      }
    }
  })

  props.ws.onMessage('disconnected', () => {
    if (terminal) terminal.write('\r\n\r\n[Session disconnected]\r\n')
  })

  props.ws.onMessage('attached', () => {
    if (terminal) terminal.focus()
    handleResize()
  })
  
  // Global focus management
  // Focus terminal on click
  if (terminalContainer.value) {
    terminalContainer.value.addEventListener('click', () => {
      if (terminal) terminal.focus()
    })
  }
  
  // Remove the focus interval - it's too aggressive

  attachToSession()

  window.addEventListener('resize', debouncedResize)
  
  // Also observe the terminal container for size changes
  const resizeObserver = new ResizeObserver(debouncedResize)
  if (terminalContainer.value) {
    resizeObserver.observe(terminalContainer.value)
  }
})

onUnmounted(() => {
  if (terminal) {
    terminal.dispose()
  }
  props.ws.offMessage('output')
  props.ws.offMessage('disconnected')
  props.ws.offMessage('attached')
  window.removeEventListener('resize', debouncedResize)
  if (resizeTimeout) clearTimeout(resizeTimeout)
})

watch(() => props.session, () => {
  if (terminal) {
    terminal.clear()
  }
  attachToSession()
})

const attachToSession = async (): Promise<void> => {
  // Ensure WebSocket is connected
  await props.ws.ensureConnected()
  
  let cols = 80
  let rows = 24
  
  if (fitAddon && terminal) {
    const dimensions = fitAddon.proposeDimensions()
    if (dimensions && dimensions.cols > 0 && dimensions.rows > 0) {
      cols = dimensions.cols
      rows = dimensions.rows
    } else {
      // Use terminal dimensions as fallback
      cols = terminal.cols || 80
      rows = terminal.rows || 24
    }
  }
  
  const message: AttachSessionMessage = {
    type: 'attach-session',
    sessionName: props.session,
    cols: cols,
    rows: rows
  }
  props.ws.send(message)
}

const handleResize = (): void => {
  if (fitAddon && terminal && terminalContainer.value) {
    try {
      // Ensure container has valid dimensions before fitting
      const rect = terminalContainer.value.getBoundingClientRect()
      if (rect.width > 0 && rect.height > 0) {
        fitAddon.fit()
        // Send the new dimensions to the server
        const dimensions = fitAddon.proposeDimensions()
        if (dimensions && dimensions.cols > 0 && dimensions.rows > 0) {
          terminalSize.value = { cols: dimensions.cols, rows: dimensions.rows }
          if (props.ws.isConnected.value) {
            const message: ResizeMessage = {
              type: 'resize',
              cols: dimensions.cols,
              rows: dimensions.rows
            }
            props.ws.send(message)
          }
        }
      }
    } catch (e) {
      console.error('Error resizing terminal:', e)
    }
  }
}

// Add a debounced resize handler for better performance
let resizeTimeout: ReturnType<typeof setTimeout> | null = null
const debouncedResize = (): void => {
  if (resizeTimeout) clearTimeout(resizeTimeout)
  resizeTimeout = setTimeout(handleResize, 100)
}

const focusTerminal = (): void => {
  if (terminal) {
    terminal.focus()
    console.log('Terminal focused on click')
  }
}

// Mobile touch handling
let touchStartTime = 0
const handleTouchStart = (e: TouchEvent): void => {
  touchStartTime = Date.now()
  // Prevent default to avoid scrolling issues
  if (e.target === terminalContainer.value) {
    focusTerminal()
  }
}

const handleTouchEnd = (_e: TouchEvent): void => {
  const touchDuration = Date.now() - touchStartTime
  // Only focus if it's a quick tap, not a scroll
  if (touchDuration < 200) {
    focusTerminal()
  }
}

// Mobile keyboard control methods
const sendKey = (key: string): void => {
  if (!terminal || !props.ws.isConnected.value) return
  
  const keyMap: Record<string, string> = {
    'Escape': '\x1b',
    'Tab': '\t',
    'ArrowUp': '\x1b[A',
    'ArrowDown': '\x1b[B',
    'ArrowLeft': '\x1b[D',
    'ArrowRight': '\x1b[C',
  }
  
  const data = keyMap[key] || key
  
  // Send through WebSocket
  const message: InputMessage = {
    type: 'input',
    data: data
  }
  props.ws.send(message)
  
  terminal.focus()
}

const sendCtrlKey = (key: string): void => {
  console.log('sendCtrlKey called with:', key)
  if (!terminal || !props.ws.isConnected.value) {
    console.log('Terminal or WebSocket not ready')
    return
  }
  
  // Convert letter to control character
  const code = key.toUpperCase().charCodeAt(0) - 64
  const ctrlChar = String.fromCharCode(code)
  
  console.log('Sending Ctrl+' + key + ' as char code:', code)
  
  // Send through WebSocket
  const message: InputMessage = {
    type: 'input',
    data: ctrlChar
  }
  props.ws.send(message)
  
  terminal.focus()
}

const toggleCtrl = (): void => {
  ctrlPressed.value = !ctrlPressed.value
  if (terminal) terminal.focus()
  
  // Auto-release after 5 seconds
  if (ctrlPressed.value) {
    setTimeout(() => {
      ctrlPressed.value = false
    }, 5000)
  }
}
</script>