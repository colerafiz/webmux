<template>
  <div class="p-3 border-t" style="border-color: var(--border-primary)">
    <div class="flex items-center justify-between">
      <div class="flex items-center space-x-2">
        <button
          @click="toggleAudio"
          :disabled="isLoading"
          class="p-2 hover-bg rounded transition-colors flex items-center space-x-2"
          :class="[
            isStreaming ? 'text-green-500' : '',
            isLoading ? 'opacity-50 cursor-not-allowed' : ''
          ]"
          :title="audioButtonTitle"
        >
          <!-- Speaker icon -->
          <svg v-if="!isMuted" class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
              d="M15.536 8.464a5 5 0 010 7.072m2.828-9.9a9 9 0 010 12.728M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z" />
          </svg>
          
          <!-- Muted speaker icon -->
          <svg v-else class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
              d="M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z" />
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2" />
          </svg>
          
          <span v-if="!isCollapsed" class="text-xs">
            {{ isStreaming ? 'Stop Audio' : 'Start Audio' }}
          </span>
        </button>
        
        <!-- Mute toggle button (only shown when streaming) -->
        <button
          v-if="isStreaming"
          @click="toggleMute"
          class="p-1.5 hover-bg rounded transition-colors"
          :title="isMuted ? 'Unmute' : 'Mute'"
        >
          <svg v-if="!isMuted" class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
              d="M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z" />
          </svg>
          <svg v-else class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
              d="M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z" />
            <line x1="17" y1="9" x2="23" y2="15" stroke="currentColor" stroke-width="2"/>
          </svg>
        </button>
      </div>
      
      <!-- Status indicator -->
      <div v-if="isStreaming && !isCollapsed" class="flex items-center space-x-1">
        <div class="w-2 h-2 bg-green-500 rounded-full animate-pulse"></div>
        <span class="text-xs" style="color: var(--text-tertiary)">Live</span>
      </div>
      
      <!-- Test button for debugging -->
      <button
        v-if="!isCollapsed"
        @click="testAudio"
        class="p-1 hover-bg rounded text-xs"
        style="color: var(--text-tertiary)"
        title="Test Audio"
      >
        Test
      </button>
    </div>
    
    <!-- Error message -->
    <div v-if="error && !isCollapsed" class="mt-2 text-xs text-red-500">
      {{ error }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { audioPlayer } from '@/services/audio'
import { testAudioPlayback, testMediaSource } from '@/services/audio-test'

interface Props {
  isCollapsed: boolean
}

withDefaults(defineProps<Props>(), {
  isCollapsed: false
})

// Use audio player state
const isStreaming = audioPlayer.isStreaming
const isMuted = audioPlayer.isMuted
const error = audioPlayer.error
const isLoading = ref(false)

const audioButtonTitle = computed(() => {
  if (isLoading.value) return 'Loading...'
  if (isStreaming.value) return 'Stop audio streaming'
  return 'Start audio streaming from server'
})

const toggleAudio = async () => {
  if (isLoading.value) return
  
  isLoading.value = true
  error.value = null
  
  try {
    if (isStreaming.value) {
      audioPlayer.stopStreaming()
    } else {
      await audioPlayer.startStreaming()
    }
  } catch (err: any) {
    console.error('Audio control error:', err)
    error.value = err.message || 'Failed to control audio'
  } finally {
    isLoading.value = false
  }
}

const toggleMute = () => {
  audioPlayer.toggleMute()
}

const testAudio = () => {
  console.log('Running audio tests...')
  testAudioPlayback()
  setTimeout(() => {
    testMediaSource()
  }, 2000)
}
</script>

<style scoped>
.hover-bg:hover {
  background-color: var(--bg-hover);
}

@keyframes pulse {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.5;
  }
}

.animate-pulse {
  animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
}
</style>