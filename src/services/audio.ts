/**
 * Primary audio player using MediaSource API for low-latency streaming
 * Supports real-time audio streaming with WebM/Opus codec
 */
export class AudioPlayer {
  private mediaSource: MediaSource | null = null
  private sourceBuffer: SourceBuffer | null = null
  private audioElement: HTMLAudioElement | null = null
  private queue: string[] = []
  private isAppending = false
  private isInitialized = false
  private isStreamActive = false

  constructor() {
    this.initialize()
  }

  private initialize() {
    try {
      // Create audio element
      this.audioElement = new Audio()
      this.audioElement.autoplay = true
      this.audioElement.volume = 1.0
      
      // Create MediaSource
      this.mediaSource = new MediaSource()
      this.audioElement.src = URL.createObjectURL(this.mediaSource)
      
      this.mediaSource.addEventListener('sourceopen', () => {
        this.onSourceOpen()
      })
      
      this.mediaSource.addEventListener('sourceended', () => {
        this.isStreamActive = false
      })
      
      this.mediaSource.addEventListener('sourceclose', () => {
        this.isStreamActive = false
      })
      
      this.mediaSource.addEventListener('error', (e) => {
        console.error('MediaSource error:', e)
        this.isStreamActive = false
      })
      
      // Add audio element event listeners
      this.audioElement.addEventListener('error', (e) => {
        console.error('Audio element error:', e)
      })
      
      this.audioElement.addEventListener('canplay', () => {
        this.audioElement?.play().catch(err => {
          console.error('Failed to start playback:', err)
        })
      })
      
    } catch (err) {
      console.error('Failed to initialize audio player:', err)
    }
  }

  private onSourceOpen() {
    if (!this.mediaSource || this.mediaSource.readyState !== 'open') return
    
    try {
      // WebM with Opus codec (as specified by the backend)
      const mimeType = 'audio/webm; codecs="opus"'
      
      if (!MediaSource.isTypeSupported(mimeType)) {
        console.error('Audio codec not supported:', mimeType)
        return
      }
      
      this.sourceBuffer = this.mediaSource.addSourceBuffer(mimeType)
      this.isInitialized = true
      this.isStreamActive = true
      
      this.sourceBuffer.addEventListener('updateend', () => {
        this.isAppending = false
        this.processQueue()
      })
      
      this.sourceBuffer.addEventListener('error', (e) => {
        console.error('SourceBuffer error:', e)
        this.isAppending = false
      })
      
      this.sourceBuffer.addEventListener('abort', () => {
        this.isAppending = false
      })
      
      // Set mode to sequence for streaming
      this.sourceBuffer.mode = 'sequence'
      
      // Process any queued data
      this.processQueue()
      
    } catch (err) {
      console.error('Failed to create source buffer:', err)
    }
  }

  appendAudioData(base64Data: string) {
    if (!base64Data) return
    
    // Add to queue
    this.queue.push(base64Data)
    
    // Process if not currently appending
    if (!this.isAppending) {
      this.processQueue()
    }
  }

  private processQueue() {
    if (this.isAppending || this.queue.length === 0 || !this.isInitialized || !this.isStreamActive) {
      return
    }
    
    if (!this.sourceBuffer || this.sourceBuffer.updating) {
      return
    }
    
    // Check if MediaSource is still open
    if (!this.mediaSource || this.mediaSource.readyState !== 'open') {
      this.queue = []
      return
    }
    
    const base64Data = this.queue.shift()
    if (!base64Data) return
    
    try {
      // Convert base64 to ArrayBuffer
      const binaryString = atob(base64Data)
      const bytes = new Uint8Array(binaryString.length)
      for (let i = 0; i < binaryString.length; i++) {
        bytes[i] = binaryString.charCodeAt(i)
      }
      
      this.isAppending = true
      this.sourceBuffer.appendBuffer(bytes.buffer)
      
      // Play if paused
      if (this.audioElement?.paused) {
        this.audioElement.play().catch(err => {
          console.error('Failed to play audio:', err)
        })
      }
      
    } catch (err) {
      console.error('Failed to append audio data:', err)
      this.isAppending = false
      
      // If it's an InvalidStateError, the MediaSource was closed
      if (err instanceof DOMException && err.name === 'InvalidStateError') {
        this.isStreamActive = false
        this.queue = []
        return
      }
      
      // Try next item in queue
      if (this.queue.length > 0) {
        setTimeout(() => this.processQueue(), 100)
      }
    }
  }

  stop() {
    this.isStreamActive = false
    
    // Clear queue first
    this.queue = []
    this.isAppending = false
    
    // Close source buffer
    if (this.sourceBuffer && this.mediaSource?.readyState === 'open') {
      try {
        this.sourceBuffer.abort()
      } catch (err) {
        console.warn('Error aborting source buffer:', err)
      }
    }
    
    // End media source
    if (this.mediaSource && this.mediaSource.readyState === 'open') {
      try {
        this.mediaSource.endOfStream()
      } catch (err) {
        console.warn('Error ending media source:', err)
      }
    }
    
    // Stop audio playback
    if (this.audioElement) {
      this.audioElement.pause()
      // Revoke the object URL to free memory
      if (this.audioElement.src.startsWith('blob:')) {
        URL.revokeObjectURL(this.audioElement.src)
      }
      this.audioElement.src = ''
    }
    
    // Clean up
    this.sourceBuffer = null
    this.mediaSource = null
    this.audioElement = null
    this.isInitialized = false
  }
}

/**
 * Fallback audio player for browsers that don't support MediaSource API
 * Collects audio chunks and plays them as complete segments
 */
export class SimpleAudioPlayer {
  private audioContext: AudioContext | null = null
  private isPlaying = false
  private chunkQueue: string[] = []
  private currentAudio: HTMLAudioElement | null = null

  constructor() {
    try {
      // @ts-ignore - AudioContext might have vendor prefix
      const AudioContextClass = window.AudioContext || window.webkitAudioContext
      this.audioContext = new AudioContextClass()
    } catch (err) {
      console.error('Failed to create AudioContext:', err)
    }
  }

  appendAudioData(base64Data: string) {
    if (!base64Data) return
    
    // For simple player, we'll collect chunks and try to play them as complete segments
    this.chunkQueue.push(base64Data)
    
    // If we have enough data (e.g., 5 chunks), try to play
    if (this.chunkQueue.length >= 5 && !this.isPlaying) {
      this.playCollectedChunks()
    }
  }

  private playCollectedChunks() {
    if (this.chunkQueue.length === 0 || this.isPlaying) return
    
    try {
      // Combine multiple chunks
      const combinedData = this.chunkQueue.splice(0, 5).join('')
      const dataUrl = `data:audio/webm;base64,${combinedData}`
      
      // Create new audio element
      const audio = new Audio()
      audio.autoplay = true
      audio.volume = 1.0
      
      audio.addEventListener('play', () => {
        this.isPlaying = true
      })
      
      audio.addEventListener('ended', () => {
        this.isPlaying = false
        // Play next chunks if available
        if (this.chunkQueue.length > 0) {
          setTimeout(() => this.playCollectedChunks(), 100)
        }
      })
      
      audio.addEventListener('error', (e) => {
        this.isPlaying = false
        console.error('Audio playback error:', e)
      })
      
      // Clean up previous audio if exists
      if (this.currentAudio) {
        this.currentAudio.pause()
        this.currentAudio.src = ''
      }
      
      this.currentAudio = audio
      audio.src = dataUrl
      
      audio.play().catch(err => {
        console.error('Failed to play audio:', err)
        this.isPlaying = false
      })
      
    } catch (err) {
      console.error('Failed to play collected chunks:', err)
      this.isPlaying = false
    }
  }

  stop() {
    this.isPlaying = false
    this.chunkQueue = []
    
    if (this.currentAudio) {
      this.currentAudio.pause()
      this.currentAudio.src = ''
      this.currentAudio = null
    }
    
    if (this.audioContext) {
      this.audioContext.close()
      this.audioContext = null
    }
  }
}