<template>
  <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-[60]" @click.self="close">
    <div 
      class="p-6 rounded-lg shadow-xl max-w-2xl w-full mx-4 max-h-[80vh] overflow-y-auto" 
      style="background: var(--bg-secondary); border: 1px solid var(--border-primary)"
    >
      <h3 class="text-lg font-semibold mb-4" style="color: var(--text-primary)">
        Version History - {{ file.name }}
      </h3>

      <!-- Loading state -->
      <div v-if="isLoading" class="py-8 text-center">
        <div class="animate-pulse text-sm" style="color: var(--text-tertiary)">
          Loading history...
        </div>
      </div>

      <!-- History list -->
      <div v-else-if="versions.length > 0" class="space-y-3">
        <div
          v-for="(version, index) in versions"
          :key="version.timestamp"
          class="p-3 rounded border"
          style="background: var(--bg-tertiary); border-color: var(--border-primary)"
        >
          <div class="flex items-start justify-between">
            <div class="flex-1">
              <div class="flex items-center space-x-2">
                <span class="text-sm font-medium" style="color: var(--text-primary)">
                  Version {{ versions.length - index }}
                </span>
                <span v-if="index === 0" class="text-xs px-2 py-0.5 rounded bg-green-500 bg-opacity-20 text-green-500">
                  Current
                </span>
              </div>
              
              <div class="mt-1 text-xs" style="color: var(--text-secondary)">
                {{ formatDate(version.timestamp) }}
              </div>
              
              <div class="mt-1 text-xs" style="color: var(--text-tertiary)">
                {{ formatSize(version.size) }} • {{ version.hash.slice(0, 8) }}
              </div>
            </div>
            
            <div class="flex items-center space-x-2 ml-4">
              <!-- Preview button -->
              <button
                @click="previewVersion(version)"
                class="p-1 rounded hover-bg"
                style="color: var(--text-secondary)"
                title="Preview"
              >
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                </svg>
              </button>
              
              <!-- Restore button -->
              <button
                v-if="index > 0"
                @click="restoreVersion(version)"
                class="px-3 py-1 text-xs rounded"
                style="background: var(--bg-primary); color: var(--text-primary)"
              >
                Restore
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- Empty state -->
      <div v-else class="py-8 text-center">
        <p class="text-sm" style="color: var(--text-tertiary)">
          No version history available
        </p>
      </div>

      <!-- Close button -->
      <div class="mt-6 flex justify-end">
        <button
          @click="close"
          class="px-4 py-2 text-sm border rounded"
          style="background: var(--bg-secondary); border-color: var(--border-primary); color: var(--text-secondary)"
        >
          Close
        </button>
      </div>
    </div>

    <!-- Preview Modal -->
    <div v-if="previewingVersion" class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-[70]" @click.self="closePreview">
      <div 
        class="p-6 rounded-lg shadow-xl max-w-4xl w-full mx-4 max-h-[80vh] flex flex-col" 
        style="background: var(--bg-secondary); border: 1px solid var(--border-primary)"
      >
        <h4 class="text-md font-semibold mb-3" style="color: var(--text-primary)">
          Version Preview - {{ formatDate(previewingVersion.timestamp) }}
        </h4>
        
        <div class="flex-1 overflow-auto p-4 rounded font-mono text-sm" style="background: var(--bg-primary)">
          <pre style="color: var(--text-primary)">{{ previewingVersion.content }}</pre>
        </div>
        
        <div class="mt-4 flex justify-end space-x-2">
          <button
            @click="closePreview"
            class="px-4 py-2 text-sm border rounded"
            style="background: var(--bg-secondary); border-color: var(--border-primary); color: var(--text-secondary)"
          >
            Close
          </button>
          <button
            @click="restoreVersion(previewingVersion)"
            class="px-4 py-2 text-sm border rounded"
            style="background: var(--bg-primary); border-color: var(--border-primary); color: var(--text-primary)"
          >
            Restore This Version
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useWebSocket } from '@/composables/useWebSocket'
import type { DotFile, FileVersion, ServerMessage } from '@/types'

interface Props {
  file: DotFile
}

const props = defineProps<Props>()

const emit = defineEmits<{
  restore: [content: string]
  close: []
}>()

const ws = useWebSocket()

// State
const isLoading = ref(true)
const versions = ref<FileVersion[]>([])
const previewingVersion = ref<FileVersion | null>(null)

// Load version history
const loadHistory = () => {
  isLoading.value = true
  ws.send({
    type: 'get-dotfile-history',
    path: props.file.path
  })
}

// Format date
const formatDate = (timestamp: string) => {
  const date = new Date(timestamp)
  const now = new Date()
  const diff = now.getTime() - date.getTime()
  
  // Less than 1 minute
  if (diff < 60000) return 'Just now'
  
  // Less than 1 hour
  if (diff < 3600000) {
    const minutes = Math.floor(diff / 60000)
    return `${minutes} minute${minutes > 1 ? 's' : ''} ago`
  }
  
  // Less than 24 hours
  if (diff < 86400000) {
    const hours = Math.floor(diff / 3600000)
    return `${hours} hour${hours > 1 ? 's' : ''} ago`
  }
  
  // Less than 7 days
  if (diff < 604800000) {
    const days = Math.floor(diff / 86400000)
    return `${days} day${days > 1 ? 's' : ''} ago`
  }
  
  // Full date
  return date.toLocaleDateString() + ' ' + date.toLocaleTimeString()
}

// Format file size
const formatSize = (bytes: number) => {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1048576) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / 1048576).toFixed(1)} MB`
}

// Preview version
const previewVersion = (version: FileVersion) => {
  previewingVersion.value = version
}

// Close preview
const closePreview = () => {
  previewingVersion.value = null
}

// Restore version
const restoreVersion = (version: FileVersion) => {
  if (confirm(`Restore this version from ${formatDate(version.timestamp)}?`)) {
    emit('restore', version.content)
    close()
  }
}

// Close modal
const close = () => {
  emit('close')
}

// WebSocket handler
const handleDotfileHistory = (msg: Extract<ServerMessage, { type: 'dotfile-history' }>) => {
  versions.value = msg.versions
  isLoading.value = false
}

// Setup
onMounted(() => {
  loadHistory()
  const unsub = ws.onMessage('dotfile-history', handleDotfileHistory)
  
  // Cleanup
  return () => {
    unsub()
  }
})
</script>

<style scoped>
.hover-bg:hover {
  filter: brightness(1.2);
}

pre {
  white-space: pre-wrap;
  word-wrap: break-word;
}
</style>