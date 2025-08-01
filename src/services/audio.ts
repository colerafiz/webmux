import { ref } from 'vue'
import { wsManager } from './websocket'
import type { AudioStreamMessage, AudioStatusMessage } from '@/types'

class AudioPlayer {
  private mediaSource: MediaSource | null = null
  private sourceBuffer: SourceBuffer | null = null
  private audioElement: HTMLAudioElement | null = null
  private queue: Uint8Array[] = []
  private isAppending = false
  private handlersSetup = false
  
  public isStreaming = ref(false)
  public isMuted = ref(false)
  public error = ref<string | null>(null)
  
  constructor() {
    // Don't set up handlers in constructor - wait for WebSocket to be ready
    console.log('AudioPlayer initialized')
  }
  
  private setupWebSocketHandlers(): void {
    if (this.handlersSetup) {
      console.log('WebSocket handlers already set up')
      return
    }
    
    console.log('Setting up WebSocket handlers for audio')
    
    // Handle audio stream data
    wsManager.onMessage<AudioStreamMessage>('audio-stream', (message) => {
      console.log('Received audio-stream message')
      if (message.data) {
        this.handleAudioData(message.data)
      }
    })
    
    // Handle audio status updates
    wsManager.onMessage<AudioStatusMessage>('audio-status', (message) => {
      console.log('Received audio-status message:', message)
      this.isStreaming.value = message.streaming
      if (message.error) {
        this.error.value = message.error
        console.error('Audio streaming error:', message.error)
      }
      
      if (!message.streaming) {
        this.cleanup()
      }
    })
    
    this.handlersSetup = true
  }
  
  async startStreaming(): Promise<void> {
    try {
      this.error.value = null
      console.log('Starting audio streaming...')
      
      // Ensure WebSocket handlers are set up
      this.setupWebSocketHandlers()
      
      // Create audio element
      this.audioElement = new Audio()
      this.audioElement.autoplay = true
      this.audioElement.muted = this.isMuted.value
      console.log('Audio element created, autoplay:', this.audioElement.autoplay, 'muted:', this.audioElement.muted)
      
      // Add event listeners for debugging
      this.audioElement.addEventListener('play', () => console.log('Audio: play event'))
      this.audioElement.addEventListener('playing', () => console.log('Audio: playing event'))
      this.audioElement.addEventListener('pause', () => console.log('Audio: pause event'))
      this.audioElement.addEventListener('error', (e) => console.error('Audio element error:', e))
      this.audioElement.addEventListener('loadstart', () => console.log('Audio: loadstart'))
      this.audioElement.addEventListener('canplay', () => console.log('Audio: canplay'))
      
      // Create MediaSource
      this.mediaSource = new MediaSource()
      const url = URL.createObjectURL(this.mediaSource)
      this.audioElement.src = url
      console.log('MediaSource created, URL:', url)
      
      // Append to DOM to ensure it can play
      document.body.appendChild(this.audioElement)
      
      // Wait for MediaSource to open
      await new Promise<void>((resolve) => {
        this.mediaSource!.addEventListener('sourceopen', () => {
          console.log('MediaSource opened')
          resolve()
        }, { once: true })
      })
      
      // Add source buffer for WebM/Opus
      try {
        this.sourceBuffer = this.mediaSource!.addSourceBuffer('audio/webm; codecs="opus"')
        console.log('SourceBuffer created for audio/webm; codecs="opus"')
      } catch (e) {
        console.error('Failed to create source buffer:', e)
        throw e
      }
      
      this.sourceBuffer.addEventListener('updateend', () => {
        console.log('SourceBuffer updateend, queue length:', this.queue.length)
        this.isAppending = false  // Reset the flag
        this.processQueue()
      })
      
      this.sourceBuffer.addEventListener('error', (e) => {
        console.error('SourceBuffer error:', e)
      })
      
      // Send start command to server
      await wsManager.ensureConnected()
      console.log('WebSocket connected, sending start command')
      wsManager.send({
        type: 'audio-control',
        action: 'start'
      })
      
      console.log('Audio streaming start command sent')
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
      console.log('Received audio data, type:', typeof data, 'length:', 
        typeof data === 'string' ? data.length : data.byteLength)
      
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
        console.log('Decoded base64 to ArrayBuffer, size:', arrayBuffer.byteLength)
      } else {
        arrayBuffer = data
      }
      
      // Add to queue
      this.queue.push(new Uint8Array(arrayBuffer))
      console.log('Added to queue, total items:', this.queue.length)
      
      // Process queue if not already processing
      if (!this.isAppending) {
        console.log('Starting queue processing')
        this.processQueue()
      } else {
        console.log('Already appending, will process later')
      }
    } catch (error) {
      console.error('Error handling audio data:', error)
    }
  }
  
  private processQueue(): void {
    if (this.isAppending || this.queue.length === 0 || !this.sourceBuffer) {
      console.log('ProcessQueue skipped - isAppending:', this.isAppending, 
        'queue:', this.queue.length, 'sourceBuffer:', !!this.sourceBuffer)
      return
    }
    
    // Check if source buffer is ready
    if (this.sourceBuffer.updating) {
      console.log('ProcessQueue skipped - sourceBuffer is updating')
      return
    }
    
    try {
      this.isAppending = true
      const chunk = this.queue.shift()!
      console.log('Appending chunk to sourceBuffer, size:', chunk.byteLength)
      this.sourceBuffer.appendBuffer(chunk)
      
      // Try to play if not already playing
      if (this.audioElement && this.audioElement.paused) {
        console.log('Audio is paused, attempting to play')
        this.audioElement.play().then(() => {
          console.log('Audio playback started')
        }).catch(e => {
          console.error('Failed to start playback:', e)
        })
      }
    } catch (error) {
      console.error('Error appending audio buffer:', error)
      this.isAppending = false
      
      // Try to recover by clearing the buffer
      if (this.sourceBuffer && !this.sourceBuffer.updating) {
        try {
          this.sourceBuffer.abort()
          console.log('Aborted sourceBuffer after error')
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