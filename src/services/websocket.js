// Singleton WebSocket manager to ensure single connection
class WebSocketManager {
  constructor() {
    this.ws = null
    this.isConnected = false
    this.messageHandlers = new Map()
    this.connectionPromise = null
  }

  connect() {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      return Promise.resolve()
    }

    if (this.connectionPromise) {
      return this.connectionPromise
    }

    this.connectionPromise = new Promise((resolve) => {
      const wsUrl = import.meta.env.DEV 
        ? 'ws://localhost:3000/ws'
        : `${window.location.protocol === 'https:' ? 'wss:' : 'ws:'}//${window.location.host}/ws`
      
      this.ws = new WebSocket(wsUrl)
      
      this.ws.onopen = () => {
        this.isConnected = true
        this.connectionPromise = null
        console.log('WebSocket connected')
        resolve()
      }
      
      this.ws.onmessage = (event) => {
        const data = JSON.parse(event.data)
        const handlers = this.messageHandlers.get(data.type) || []
        handlers.forEach(handler => handler(data))
      }
      
      this.ws.onerror = (error) => {
        console.error('WebSocket error:', error)
      }
      
      this.ws.onclose = () => {
        console.log('WebSocket disconnected, reconnecting...')
        this.isConnected = false
        this.ws = null
        this.connectionPromise = null
        setTimeout(() => this.connect(), 3000)
      }
    })

    return this.connectionPromise
  }

  send(data) {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(data))
    } else {
      console.warn('WebSocket not connected, message not sent:', data)
    }
  }

  onMessage(type, handler) {
    if (!this.messageHandlers.has(type)) {
      this.messageHandlers.set(type, [])
    }
    this.messageHandlers.get(type).push(handler)
  }

  offMessage(type, handler) {
    if (this.messageHandlers.has(type)) {
      const handlers = this.messageHandlers.get(type)
      const index = handlers.indexOf(handler)
      if (index > -1) {
        handlers.splice(index, 1)
      }
    }
  }

  close() {
    if (this.ws) {
      this.ws.close()
    }
  }
  
  ensureConnected() {
    if (this.isConnected) {
      return Promise.resolve()
    }
    return this.connect()
  }
}

// Export singleton instance
export const wsManager = new WebSocketManager()