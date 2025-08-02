<template>
  <div class="pl-6 mt-1">
    <!-- Modal for window name input -->
    <div v-if="showCreateModal" class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50" @click.self="cancelCreateWindow">
      <div class="p-6 rounded-lg shadow-xl max-w-sm w-full mx-4" style="background: var(--bg-secondary); border: 1px solid var(--border-primary)">
        <h3 class="text-lg font-semibold mb-4" style="color: var(--text-primary)">Create New Window</h3>
        <input 
          v-model="newWindowName"
          type="text" 
          placeholder="Window name (optional)"
          class="w-full px-3 py-2 border rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
          style="background: var(--bg-primary); border-color: var(--border-primary); color: var(--text-primary)"
          @keyup.enter="confirmCreateWindow"
          ref="windowNameInput"
        />
        <div class="flex justify-end space-x-2 mt-4">
          <button 
            @click="cancelCreateWindow"
            class="px-4 py-2 text-sm border rounded"
            style="background: var(--bg-secondary); border-color: var(--border-primary); color: var(--text-secondary)"
          >
            Cancel
          </button>
          <button 
            @click="confirmCreateWindow"
            class="px-4 py-2 text-sm border rounded"
            style="background: var(--bg-primary); border-color: var(--border-primary); color: var(--text-primary)"
          >
            Create
          </button>
        </div>
      </div>
    </div>
    
    <!-- Modal for delete confirmation -->
    <div v-if="showDeleteModal" class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50" @click.self="cancelDelete">
      <div class="p-6 rounded-lg shadow-xl max-w-sm w-full mx-4" style="background: var(--bg-secondary); border: 1px solid var(--border-primary)">
        <h3 class="text-lg font-semibold mb-4" style="color: var(--text-primary)">Delete Window</h3>
        <p class="mb-4" style="color: var(--text-secondary)">
          Are you sure you want to kill window "{{ windowToDelete?.name }}"?
        </p>
        <div class="flex justify-end space-x-2">
          <button 
            @click="cancelDelete"
            class="px-4 py-2 text-sm border rounded"
            style="background: var(--bg-secondary); border-color: var(--border-primary); color: var(--text-secondary)"
          >
            Cancel
          </button>
          <button 
            @click="confirmDelete"
            class="px-4 py-2 text-sm border rounded"
            style="background: #f85149; border-color: #f85149; color: white"
          >
            Delete
          </button>
        </div>
      </div>
    </div>
    <div v-if="loading" class="text-xs" style="color: var(--text-tertiary)">
      Loading windows...
    </div>
    <div v-else-if="error" class="text-xs text-red-500">
      Error loading windows
    </div>
    <div v-else-if="windows.length === 0" class="text-xs" style="color: var(--text-tertiary)">
      No windows
    </div>
    <div v-else class="space-y-0.5">
      <div
        v-for="window in windows"
        :key="window.index"
        @click="$emit('select-window', window)"
        class="flex items-center justify-between px-2 py-1 rounded cursor-pointer hover-bg text-xs transition-all duration-150"
        :class="{ 'bg-opacity-30': window.active && props.isActiveSession }"
        :style="{
          background: window.active && props.isActiveSession ? 'var(--bg-tertiary)' : 'transparent',
          borderLeft: window.active && props.isActiveSession ? '2px solid var(--accent-secondary)' : '2px solid transparent'
        }"
      >
        <div class="flex items-center space-x-2 min-w-0">
          <span class="font-mono" style="color: var(--text-tertiary)">{{ window.index }}:</span>
          <span v-if="!isEditing(window)" class="truncate" :style="{ color: window.active && props.isActiveSession ? 'var(--text-primary)' : 'var(--text-secondary)' }">
            {{ window.name }}
          </span>
          <input
            v-else
            v-model="editingName"
            @keyup.enter="confirmRename(window)"
            @keyup.escape="cancelEdit"
            @blur="confirmRename(window)"
            ref="editInput"
            class="px-1 py-0.5 text-xs w-full focus:outline-none border"
            style="background: var(--bg-primary); border-color: var(--border-primary); color: var(--text-primary)"
          />
          <span style="color: var(--text-tertiary)">({{ window.panes }}p)</span>
        </div>
        
        <div class="flex items-center space-x-0.5 opacity-0 group-hover:opacity-100 transition-opacity" @click.stop>
          <button
            @click="startEdit(window)"
            class="p-0.5 hover-bg rounded"
            style="color: var(--text-tertiary)"
            title="Rename Window"
          >
            <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
            </svg>
          </button>
          <button
            @click="killWindow(window)"
            class="p-0.5 hover-bg rounded"
            style="color: var(--text-tertiary)"
            title="Kill Window"
          >
            <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
      </div>
      
      <button
        @click="createWindow"
        class="w-full px-2 py-1 text-xs hover-bg rounded flex items-center justify-center space-x-1"
        style="color: var(--text-tertiary); border: 1px dashed; border-color: var(--border-secondary)"
      >
        <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
        </svg>
        <span>New Window</span>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, nextTick, watch, onUnmounted } from 'vue'
import { websocketApi } from '@/api/websocket-api'
import { useWebSocket } from '@/composables/useWebSocket'
import type { TmuxWindow, WindowSelectedMessage, WindowsListMessage } from '@/types'

interface Props {
  sessionName: string
  isActiveSession?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  isActiveSession: false
})

const emit = defineEmits<{
  'select-window': [window: TmuxWindow]
  refresh: []
}>()

const windows = ref<TmuxWindow[]>([])
const loading = ref<boolean>(false) // Start with false to prevent flicker
const error = ref<boolean>(false)
const editingWindow = ref<TmuxWindow | null>(null)
const editingName = ref<string>('')
const editInput = ref<HTMLInputElement | null>(null)

// Track if we've loaded windows for this session
let hasLoadedInitial = false

// Modal state for new window
const showCreateModal = ref(false)
const newWindowName = ref('')
const windowNameInput = ref<HTMLInputElement>()

// Modal state for delete confirmation
const showDeleteModal = ref(false)
const windowToDelete = ref<TmuxWindow | null>(null)

// WebSocket for real-time updates
const ws = useWebSocket()

const loadWindows = async (showLoading: boolean = true): Promise<void> => {
  // Store the session name we're loading for
  const loadingForSession = props.sessionName
  
  try {
    // Only show loading on initial load or if explicitly requested
    if (showLoading && !hasLoadedInitial) {
      loading.value = true
    }
    error.value = false
    const loadedWindows = await websocketApi.getWindows(props.sessionName)
    
    // Only update if we're still showing the same session
    if (props.sessionName === loadingForSession) {
      windows.value = loadedWindows
      hasLoadedInitial = true
    } else {
      console.log('Session changed while loading windows, ignoring stale data')
    }
  } catch (err: any) {
    // Only show error if we're still on the same session
    if (props.sessionName === loadingForSession) {
      error.value = true
      console.error('Failed to load windows for session:', props.sessionName, err)
      // If the session doesn't exist or there's an error, clear the windows
      windows.value = []
    }
  } finally {
    if (props.sessionName === loadingForSession) {
      loading.value = false
    }
  }
}

const createWindow = (): void => {
  showCreateModal.value = true
  newWindowName.value = ''
  nextTick(() => {
    windowNameInput.value?.focus()
  })
}

const confirmCreateWindow = async (): Promise<void> => {
  // Optimistically add the window
  const optimisticWindow: TmuxWindow = {
    index: windows.value.length,
    name: newWindowName.value || `Window ${windows.value.length}`,
    active: false,
    panes: 1
  }
  
  // Add to UI immediately
  windows.value = [...windows.value, optimisticWindow]
  showCreateModal.value = false
  const savedName = newWindowName.value
  newWindowName.value = ''
  
  try {
    await websocketApi.createWindow(props.sessionName, savedName || undefined)
    // Real update will come through WebSocket
    emit('refresh')
  } catch (err) {
    console.error('Failed to create window:', err)
    // Revert optimistic update
    windows.value = windows.value.filter(w => w.index !== optimisticWindow.index)
    alert('Failed to create window. Please try again.')
  }
}

const cancelCreateWindow = (): void => {
  showCreateModal.value = false
  newWindowName.value = ''
}

const killWindow = (window: TmuxWindow): void => {
  windowToDelete.value = window
  showDeleteModal.value = true
}

const confirmDelete = async (): Promise<void> => {
  if (!windowToDelete.value) return
  
  const windowToRemove = windowToDelete.value
  const originalWindows = [...windows.value]
  
  // Optimistically remove the window
  windows.value = windows.value.filter(w => w.index !== windowToRemove.index)
  showDeleteModal.value = false
  windowToDelete.value = null
  
  try {
    await websocketApi.killWindow(props.sessionName, windowToRemove.index)
    // Real update will come through WebSocket
    emit('refresh')
  } catch (err) {
    console.error('Failed to kill window:', err)
    // Revert optimistic update
    windows.value = originalWindows
    alert('Failed to delete window. Please try again.')
  }
}

const cancelDelete = (): void => {
  showDeleteModal.value = false
  windowToDelete.value = null
}

const isEditing = (window: TmuxWindow): boolean => {
  return editingWindow.value?.index === window.index
}

const startEdit = (window: TmuxWindow): void => {
  editingWindow.value = window
  editingName.value = window.name
  nextTick(() => {
    editInput.value?.focus()
    editInput.value?.select()
  })
}

const confirmRename = async (window: TmuxWindow): Promise<void> => {
  if (editingName.value && editingName.value !== window.name) {
    try {
      await websocketApi.renameWindow(props.sessionName, window.index, editingName.value)
      await loadWindows(false) // Don't show loading for rename
    } catch (err) {
      console.error('Failed to rename window:', err)
    }
  }
  cancelEdit()
}

const cancelEdit = (): void => {
  editingWindow.value = null
  editingName.value = ''
}

onMounted(() => {
  // Ensure modals are closed on mount
  showCreateModal.value = false
  showDeleteModal.value = false
  windowToDelete.value = null
  newWindowName.value = ''
  
  loadWindows()
  
  // Listen for window selection success to refresh
  ws.onMessage<WindowSelectedMessage>('window-selected', (data) => {
    if (data.success) {
      console.log('Window selected, refreshing windows for session:', props.sessionName)
      // Small delay to ensure tmux has updated
      setTimeout(() => {
        loadWindows(false) // Don't show loading for updates
      }, 100)
    }
  })
  
  // Listen for windows-list broadcasts but only update if it's for our session
  ws.onMessage<WindowsListMessage>('windows-list', (data) => {
    if (data.sessionName === props.sessionName) {
      console.log('Received window list update for our session:', props.sessionName)
      windows.value = data.windows
      error.value = false
      loading.value = false
    } else {
      console.log('Ignoring window list for different session:', data.sessionName, 'we are viewing:', props.sessionName)
    }
  })
})

onUnmounted(() => {
  // Clean up WebSocket listeners
  ws.offMessage('window-selected')
  ws.offMessage('windows-list')
  
  // Clear any pending session change timeout
  if (sessionChangeTimeout) {
    clearTimeout(sessionChangeTimeout)
    sessionChangeTimeout = null
  }
})

// Add a debounced session watcher to prevent rapid switching issues
let sessionChangeTimeout: ReturnType<typeof setTimeout> | null = null

// Watch for session name changes and reload windows
watch(() => props.sessionName, (newSessionName, oldSessionName) => {
  if (newSessionName !== oldSessionName) {
    console.log('Session changed from', oldSessionName, 'to', newSessionName, '- reloading windows')
    
    // Cancel any pending load
    if (sessionChangeTimeout) {
      clearTimeout(sessionChangeTimeout)
    }
    
    // Reset state immediately
    hasLoadedInitial = false
    windows.value = [] // Clear immediately to prevent showing stale data
    error.value = false
    loading.value = true
    
    // Debounce the actual load to prevent rapid switches
    sessionChangeTimeout = setTimeout(() => {
      loadWindows()
    }, 100)
  }
})

defineExpose({
  refresh: () => loadWindows(false) // Don't show loading on manual refresh
})
</script>