import type { WsMessage } from '@/types'

type MessageHandler<T = any> = (data: T) => void

// Singleton WebSocket manager to ensure single connection
class WebSocketManager {
  private ws: WebSocket | null = null
  public isConnected: boolean = false
  private messageHandlers: Map<string, MessageHandler[]> = new Map()
  private connectionPromise: Promise<void> | null = null
  private pingInterval: number | null = null
  private reconnectAttempts: number = 0
  private readonly maxReconnectAttempts: number = 5

  connect(): Promise<void> {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      return Promise.resolve()
    }

    if (this.connectionPromise) {
      return this.connectionPromise
    }

    this.connectionPromise = new Promise((resolve) => {
      // Always use the current host for WebSocket connections
      // This works for localhost, network IPs, and Tailscale IPs
      const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
      let wsUrl: string
      
      if (import.meta.env.DEV) {
        // In development, always use the Vite server's proxy
        // This ensures mobile connections work through the same port
        wsUrl = `${protocol}//${window.location.host}/ws`
      } else {
        // Production mode - use same host and port as current page
        wsUrl = `${protocol}//${window.location.host}/ws`
      }
      
      console.log('Connecting to WebSocket:', wsUrl)
      this.ws = new WebSocket(wsUrl)
      
      this.ws.onopen = () => {
        this.isConnected = true
        this.connectionPromise = null
        this.reconnectAttempts = 0
        console.log('WebSocket connected')
        
        // Start ping to keep connection alive
        this.startPing()
        
        resolve()
      }
      
      this.ws.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data) as WsMessage
          console.log('WebSocket message received:', data.type, data.type === 'audio-stream' ? '(audio data)' : data)
          const handlers = this.messageHandlers.get(data.type) || []
          console.log(`Handlers for ${data.type}:`, handlers.length)
          handlers.forEach(handler => handler(data))
        } catch (error) {
          console.error('Error parsing WebSocket message:', error)
        }
      }
      
      this.ws.onerror = (error) => {
        console.error('WebSocket error:', error)
      }
      
      this.ws.onclose = (event) => {
        console.log('WebSocket disconnected:', event.code, event.reason)
        this.isConnected = false
        this.ws = null
        this.connectionPromise = null
        this.stopPing()
        
        // Only reconnect if we haven't exceeded max attempts
        if (this.reconnectAttempts < this.maxReconnectAttempts) {
          this.reconnectAttempts++
          const delay = event.code === 1000 ? 3000 : 1000 // 1s for errors, 3s for normal close
          console.log(`Reconnect attempt ${this.reconnectAttempts}/${this.maxReconnectAttempts} in ${delay}ms`)
          setTimeout(() => this.connect(), delay)
        } else {
          console.error('Max reconnection attempts reached')
        }
      }
    })

    return this.connectionPromise
  }

  send(data: WsMessage): void {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      try {
        this.ws.send(JSON.stringify(data))
      } catch (err) {
        console.error('WebSocket send failed:', err)
        // Force reconnect on send failure
        this.connect()
      }
    } else {
      console.warn('WebSocket not connected, message not sent:', data)
      // Try to reconnect
      this.connect()
    }
  }

  onMessage<T = any>(type: string, handler: MessageHandler<T>): void {
    if (!this.messageHandlers.has(type)) {
      this.messageHandlers.set(type, [])
    }
    this.messageHandlers.get(type)!.push(handler)
  }

  offMessage<T = any>(type: string, handler?: MessageHandler<T>): void {
    if (!handler) {
      // Remove all handlers for this type
      this.messageHandlers.delete(type)
      return
    }
    
    if (this.messageHandlers.has(type)) {
      const handlers = this.messageHandlers.get(type)!
      const index = handlers.indexOf(handler)
      if (index > -1) {
        handlers.splice(index, 1)
      }
    }
  }

  private startPing(): void {
    this.stopPing()
    this.pingInterval = window.setInterval(() => {
      if (this.ws && this.ws.readyState === WebSocket.OPEN) {
        try {
          this.ws.send(JSON.stringify({ type: 'ping' }))
        } catch (err) {
          console.warn('Ping failed:', err)
          this.connect() // Try to reconnect
        }
      }
    }, 30000) // Ping every 30 seconds
  }
  
  private stopPing(): void {
    if (this.pingInterval) {
      clearInterval(this.pingInterval)
      this.pingInterval = null
    }
  }
  
  close(): void {
    this.stopPing()
    if (this.ws) {
      this.ws.close()
    }
  }
  
  ensureConnected(): Promise<void> {
    if (this.isConnected) {
      return Promise.resolve()
    }
    return this.connect()
  }
}

// Export singleton instance
export const wsManager = new WebSocketManager()