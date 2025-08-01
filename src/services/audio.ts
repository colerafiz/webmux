import { ref } from 'vue'
import { wsManager } from './websocket'
import type { AudioStreamMessage, AudioStatusMessage } from '@/types'

class AudioPlayer {
  private mediaSource: MediaSource | null = null
  private sourceBuffer: SourceBuffer | null = null
  private audioElement: HTMLAudioElement | null = null
  private queue: Uint8Array[] = []
  private isAppending = false
  
  public isStreaming = ref(false)
  public isMuted = ref(false)
  public error = ref<string | null>(null)
  
  constructor() {
    this.setupWebSocketHandlers()
  }
  
  private setupWebSocketHandlers(): void {
    // Handle audio stream data
    wsManager.onMessage<AudioStreamMessage>('audio-stream', (message) => {
      if (message.data) {
        this.handleAudioData(message.data)
      }
    })
    
    // Handle audio status updates
    wsManager.onMessage<AudioStatusMessage>('audio-status', (message) => {
      this.isStreaming.value = message.streaming
      if (message.error) {
        this.error.value = message.error
        console.error('Audio streaming error:', message.error)
      }
      
      if (!message.streaming) {
        this.cleanup()
      }
    })
  }
  
  async startStreaming(): Promise<void> {
    try {
      this.error.value = null
      
      // Create audio element
      this.audioElement = new Audio()
      this.audioElement.autoplay = true
      this.audioElement.muted = this.isMuted.value
      
      // Create MediaSource
      this.mediaSource = new MediaSource()
      this.audioElement.src = URL.createObjectURL(this.mediaSource)
      
      // Wait for MediaSource to open
      await new Promise<void>((resolve) => {
        this.mediaSource!.addEventListener('sourceopen', () => resolve(), { once: true })
      })
      
      // Add source buffer for WebM/Opus
      this.sourceBuffer = this.mediaSource!.addSourceBuffer('audio/webm; codecs="opus"')
      this.sourceBuffer.addEventListener('updateend', () => {
        this.processQueue()
      })
      
      // Send start command to server
      await wsManager.ensureConnected()
      wsManager.send({
        type: 'audio-control',
        action: 'start'
      })
      
      console.log('Audio streaming started')
    } catch (error) {
      console.error('Failed to start audio streaming:', error)
      this.error.value = 'Failed to start audio streaming'
      this.cleanup()
    }
  }
  
  stopStreaming(): void {
    // Send stop command to server
    wsManager.send({
      type: 'audio-control',
      action: 'stop'
    })
    
    this.cleanup()
    console.log('Audio streaming stopped')
  }
  
  toggleMute(): void {
    this.isMuted.value = !this.isMuted.value
    if (this.audioElement) {
      this.audioElement.muted = this.isMuted.value
    }
  }
  
  private handleAudioData(data: ArrayBuffer | string): void {
    try {
      // Convert base64 to ArrayBuffer if needed
      let arrayBuffer: ArrayBuffer
      if (typeof data === 'string') {
        // Decode base64
        const binaryString = atob(data)
        const bytes = new Uint8Array(binaryString.length)
        for (let i = 0; i < binaryString.length; i++) {
          bytes[i] = binaryString.charCodeAt(i)
        }
        arrayBuffer = bytes.buffer
      } else {
        arrayBuffer = data
      }
      
      // Add to queue
      this.queue.push(new Uint8Array(arrayBuffer))
      
      // Process queue if not already processing
      if (!this.isAppending) {
        this.processQueue()
      }
    } catch (error) {
      console.error('Error handling audio data:', error)
    }
  }
  
  private processQueue(): void {
    if (this.isAppending || this.queue.length === 0 || !this.sourceBuffer) {
      return
    }
    
    // Check if source buffer is ready
    if (this.sourceBuffer.updating) {
      return
    }
    
    try {
      this.isAppending = true
      const chunk = this.queue.shift()!
      this.sourceBuffer.appendBuffer(chunk)
    } catch (error) {
      console.error('Error appending audio buffer:', error)
      this.isAppending = false
      
      // Try to recover by clearing the buffer
      if (this.sourceBuffer && !this.sourceBuffer.updating) {
        try {
          this.sourceBuffer.abort()
        } catch (e) {
          // Ignore abort errors
        }
      }
    }
  }
  
  private cleanup(): void {
    this.isStreaming.value = false
    this.queue = []
    this.isAppending = false
    
    if (this.audioElement) {
      this.audioElement.pause()
      this.audioElement.src = ''
      this.audioElement = null
    }
    
    if (this.sourceBuffer) {
      try {
        if (!this.sourceBuffer.updating && this.mediaSource?.readyState === 'open') {
          this.mediaSource.removeSourceBuffer(this.sourceBuffer)
        }
      } catch (e) {
        // Ignore errors during cleanup
      }
      this.sourceBuffer = null
    }
    
    if (this.mediaSource) {
      try {
        if (this.mediaSource.readyState === 'open') {
          this.mediaSource.endOfStream()
        }
      } catch (e) {
        // Ignore errors during cleanup
      }
      this.mediaSource = null
    }
  }
}

// Export singleton instance
export const audioPlayer = new AudioPlayer()