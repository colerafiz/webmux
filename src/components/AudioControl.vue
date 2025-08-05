<template>
  <div class="mb-3">
    <button
      @click="toggleAudio"
      :disabled="isLoading || !ws.isConnected"
      class="w-full flex items-center justify-between px-3 py-2 text-xs rounded transition-colors hover-bg"
      style="background: var(--bg-secondary); color: var(--text-primary)"
      :class="{ 'opacity-50': isLoading || !ws.isConnected }"
    >
      <div class="flex items-center space-x-2">
        <!-- Speaker icon -->
        <svg 
          class="w-4 h-4"
          :class="{ 'text-green-500': isStreaming }"
          fill="none" 
          stroke="currentColor" 
          viewBox="0 0 24 24"
        >
          <template v-if="isStreaming">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15.536 8.464a5 5 0 010 7.072m2.828-9.9a9 9 0 010 12.728M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z" />
          </template>
          <template v-else>
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z" />
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2" />
          </template>
        </svg>
        
        <span>{{ isStreaming ? 'Audio Streaming' : 'Audio Off' }}</span>
        
        <!-- Loading spinner -->
        <svg 
          v-if="isLoading"
          class="animate-spin h-3 w-3"
          xmlns="http://www.w3.org/2000/svg" 
          fill="none" 
          viewBox="0 0 24 24"
        >
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
        </svg>
      </div>
      
      <!-- Status indicator -->
      <div 
        class="w-2 h-2 rounded-full transition-colors"
        :class="{
          'bg-green-500': isStreaming && !error,
          'bg-red-500': error,
          'bg-gray-500': !isStreaming && !error
        }"
      ></div>
    </button>
    
    <!-- Error message -->
    <div 
      v-if="error"
      class="mt-1 px-3 py-1 text-xs rounded"
      style="background: var(--accent-danger); color: white; opacity: 0.8"
    >
      {{ error }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useWebSocket } from '@/composables/useWebSocket'
import type { AudioControlMessage, AudioStatusMessage, AudioStreamMessage } from '@/types'
import { AudioPlayer, SimpleAudioPlayer } from '@/services/audio'

const ws = useWebSocket()

const isStreaming = ref(false)
const isLoading = ref(false)
const error = ref<string | null>(null)
let audioPlayer: AudioPlayer | SimpleAudioPlayer | null = null
const useSimplePlayer = ref(false)

const toggleAudio = () => {
  if (isLoading.value) return
  
  isLoading.value = true
  error.value = null
  
  const action = isStreaming.value ? 'stop' : 'start'
  const message: AudioControlMessage = {
    type: 'audio-control',
    action
  }
  
  ws.send(message)
  
  // Timeout to prevent infinite loading
  setTimeout(() => {
    if (isLoading.value) {
      isLoading.value = false
      error.value = 'Audio request timed out'
    }
  }, 5000)
}

onMounted(() => {
  // Listen for audio status updates
  ws.onMessage<AudioStatusMessage>('audio-status', (msg) => {
    isStreaming.value = msg.streaming
    isLoading.value = false
    error.value = msg.error || null
    
    if (msg.streaming && !audioPlayer) {
      // Create new audio player when streaming starts
      try {
        // Try MediaSource API first
        audioPlayer = new AudioPlayer()
        useSimplePlayer.value = false
      } catch (err) {
        try {
          // Fall back to simple player
          audioPlayer = new SimpleAudioPlayer()
          useSimplePlayer.value = true
        } catch (fallbackErr) {
          console.error('Failed to create audio player:', fallbackErr)
          error.value = 'Audio not supported in this browser'
        }
      }
    } else if (!msg.streaming && audioPlayer) {
      // Stop and cleanup when streaming stops
      audioPlayer.stop()
      audioPlayer = null
      useSimplePlayer.value = false
    }
  })
  
  // Listen for audio stream data
  ws.onMessage<AudioStreamMessage>('audio-stream', (msg) => {
    if (audioPlayer && msg.data) {
      try {
        audioPlayer.appendAudioData(msg.data)
      } catch (err) {
        console.error('Failed to process audio data:', err)
      }
    }
  })
})

onUnmounted(() => {
  // Clean up audio player
  if (audioPlayer) {
    audioPlayer.stop()
    audioPlayer = null
  }
  
  // Remove message listeners
  ws.offMessage('audio-status')
  ws.offMessage('audio-stream')
})
</script>

<style scoped>
.hover-bg:hover {
  filter: brightness(1.2);
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.animate-spin {
  animation: spin 1s linear infinite;
}
</style>