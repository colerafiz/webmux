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
    <div ref="terminalContainer" class="flex-1 overflow-hidden" tabindex="0" style="background: #000" @click="focusTerminal"></div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted, watch } from 'vue'
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import '@xterm/xterm/css/xterm.css'

const props = defineProps({
  session: {
    type: String,
    required: true
  },
  ws: {
    type: Object,
    required: true
  }
})

const terminalContainer = ref(null)
let terminal = null
let fitAddon = null
let focusInterval = null
const terminalSize = ref({ cols: 80, rows: 24 })

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
      selection: 'rgba(88, 166, 255, 0.3)',
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
    bellStyle: 'none',
    drawBoldTextInBrightColors: true,
    lineHeight: 1.2
  })

  fitAddon = new FitAddon()
  terminal.loadAddon(fitAddon)
  
  terminal.open(terminalContainer.value)
  
  // Initial fit with a small delay to ensure container is properly sized
  setTimeout(() => {
    fitAddon.fit()
    terminal.focus()
  }, 100)

  terminal.onData((data) => {
    if (props.ws.isConnected.value) {
      props.ws.send({
        type: 'input',
        data: data
      })
    }
  })

  terminal.onResize((size) => {
    terminalSize.value = { cols: size.cols, rows: size.rows }
    if (props.ws.isConnected.value) {
      props.ws.send({
        type: 'resize',
        cols: size.cols,
        rows: size.rows
      })
    }
  })

  props.ws.onMessage('output', (data) => {
    terminal.write(data.data)
  })

  props.ws.onMessage('disconnected', () => {
    terminal.write('\r\n\r\n[Session disconnected]\r\n')
  })

  props.ws.onMessage('attached', () => {
    terminal.focus()
    handleResize()
  })
  
  // Global focus management
  // Focus terminal on click
  terminalContainer.value.addEventListener('click', () => {
    terminal.focus()
  })
  
  // Remove the focus interval - it's too aggressive

  attachToSession()

  window.addEventListener('resize', debouncedResize)
  
  // Also observe the terminal container for size changes
  const resizeObserver = new ResizeObserver(debouncedResize)
  resizeObserver.observe(terminalContainer.value)
})

onUnmounted(() => {
  if (terminal) {
    terminal.dispose()
  }
  props.ws.offMessage('output')
  props.ws.offMessage('disconnected')
  props.ws.offMessage('attached')
  window.removeEventListener('resize', debouncedResize)
  clearTimeout(resizeTimeout)
})

watch(() => props.session, () => {
  if (terminal) {
    terminal.clear()
  }
  attachToSession()
})

const attachToSession = () => {
  let cols = 120
  let rows = 40
  
  if (fitAddon) {
    const dimensions = fitAddon.proposeDimensions()
    if (dimensions) {
      cols = dimensions.cols
      rows = dimensions.rows
    }
  }
  
  props.ws.send({
    type: 'attach-session',
    sessionName: props.session,
    cols: cols,
    rows: rows
  })
}

const handleResize = () => {
  if (fitAddon && terminal) {
    try {
      fitAddon.fit()
      // Send the new dimensions to the server
      const dimensions = fitAddon.proposeDimensions()
      if (dimensions) {
        terminalSize.value = { cols: dimensions.cols, rows: dimensions.rows }
        if (props.ws.isConnected.value) {
          props.ws.send({
            type: 'resize',
            cols: dimensions.cols,
            rows: dimensions.rows
          })
        }
      }
    } catch (e) {
      console.error('Error resizing terminal:', e)
    }
  }
}

// Add a debounced resize handler for better performance
let resizeTimeout = null
const debouncedResize = () => {
  clearTimeout(resizeTimeout)
  resizeTimeout = setTimeout(handleResize, 100)
}

const focusTerminal = () => {
  if (terminal) {
    terminal.focus()
    console.log('Terminal focused on click')
  }
}
</script>