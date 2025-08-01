import { ref } from 'vue'
import { wsManager } from './websocket'
import type { AudioStreamMessage, AudioStatusMessage } from '@/types'

class SimpleAudioPlayer {
  private audioElement: HTMLAudioElement | null = null
  private chunks: string[] = [] // Store base64 chunks
  private isPlaying = false
  
  public isStreaming = ref(false)
  public error = ref<string | null>(null)
  
  async startStreaming(): Promise<void> {
    console.log('SimpleAudioPlayer: Starting...')
    this.error.value = null
    this.chunks = []
    
    // Create audio element
    this.audioElement = new Audio()
    this.audioElement.volume = 0.5
    document.body.appendChild(this.audioElement)
    
    // Ensure WebSocket is connected first
    await wsManager.ensureConnected()
    
    // Set up WebSocket handlers after connection is established
    console.log('Setting up audio-stream handler')
    wsManager.onMessage<AudioStreamMessage>('audio-stream', (message) => {
      console.log('Received audio chunk, length:', message.data.length, 'chunks buffered:', this.chunks.length)
      this.chunks.push(message.data)
      
      // If we have enough data and not playing, create a blob and play
      if (!this.isPlaying && this.chunks.length > 5) {
        console.log('Starting playback with', this.chunks.length, 'chunks')
        this.playChunks()
      }
    })
    
    wsManager.onMessage<AudioStatusMessage>('audio-status', (message) => {
      console.log('Audio status:', message)
      this.isStreaming.value = message.streaming
      if (message.error) {
        this.error.value = message.error
      }
    })
    
    // Send start command after handlers are set up
    console.log('Sending audio-control start message')
    wsManager.send({
      type: 'audio-control',
      action: 'start'
    })
  }
  
  private async playChunks() {
    if (!this.audioElement || this.chunks.length === 0) return
    
    this.isPlaying = true
    console.log('Creating blob from', this.chunks.length, 'chunks')
    
    try {
      // Combine all chunks into one base64 string
      const combined = this.chunks.join('')
      
      // Convert to blob
      const binaryString = atob(combined)
      const bytes = new Uint8Array(binaryString.length)
      for (let i = 0; i < binaryString.length; i++) {
        bytes[i] = binaryString.charCodeAt(i)
      }
      
      const blob = new Blob([bytes], { type: 'audio/webm' })
      const url = URL.createObjectURL(blob)
      
      console.log('Playing audio blob, size:', blob.size)
      this.audioElement.src = url
      
      await this.audioElement.play()
      console.log('Audio playing!')
      
      // Clear chunks
      this.chunks = []
      
      // When ended, check for more chunks
      this.audioElement.onended = () => {
        console.log('Audio ended')
        this.isPlaying = false
        if (this.chunks.length > 0) {
          this.playChunks()
        }
      }
    } catch (e) {
      console.error('Play error:', e)
      this.isPlaying = false
    }
  }
  
  stopStreaming(): void {
    console.log('Stopping audio...')
    
    if (this.audioElement) {
      this.audioElement.pause()
      this.audioElement.remove()
      this.audioElement = null
    }
    
    this.chunks = []
    this.isPlaying = false
    this.isStreaming.value = false
    
    wsManager.send({
      type: 'audio-control', 
      action: 'stop'
    })
    
    // Remove handlers
    wsManager.offMessage('audio-stream')
    wsManager.offMessage('audio-status')
  }
}

export const simpleAudioPlayer = new SimpleAudioPlayer()