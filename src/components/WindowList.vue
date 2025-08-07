<template>
  <div class="window-list">
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
    <div v-if="loading" class="window-status">
      Loading windows...
    </div>
    <div v-else-if="error" class="window-status error">
      Error loading windows
    </div>
    <div v-else-if="windows.length === 0" class="window-status">
      No windows
    </div>
    <div v-else class="window-items">
      <div
        v-for="window in windows"
        :key="window.index"
        v-memo="[window.name, window.active && props.isActiveSession, window.panes, isEditing(window)]"
        @click="$emit('select-window', window)"
        class="window-item"
        :class="{ 'window-active': window.active && props.isActiveSession }"
      >
        <!-- Left content group -->
        <div class="window-content">
          <div class="window-label">
            <!-- Window icon (square brackets) -->
            <svg class="window-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
              <path stroke-linecap="round" stroke-linejoin="round" d="M4 6a2 2 0 012-2h12a2 2 0 012 2v12a2 2 0 01-2 2H6a2 2 0 01-2-2V6z" />
            </svg>
            
            <!-- Window name -->
            <span v-if="!isEditing(window)" class="window-name">
              {{ window.name }}
            </span>
            <input
              v-else
              v-model="editingName"
              @keyup.enter="confirmRename(window)"
              @keyup.escape="cancelEdit"
              @blur="confirmRename(window)"
              ref="editInput"
              class="window-name-input"
            />
          </div>
          
          <!-- Pane count badge -->
          <div class="pane-count-badge" :title="`${window.panes} ${window.panes === 1 ? 'pane' : 'panes'}`">
            <svg class="w-2.5 h-2.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
              <path stroke-linecap="round" stroke-linejoin="round" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h2m10-16h-2a2 2 0 00-2 2v12a2 2 0 002 2h2m-6-16v16" />
            </svg>
            <span>{{ window.panes }}</span>
          </div>
        </div>
        
        <!-- Actions (right side) -->
        <div class="window-actions">
          <button
            @click.stop="startEdit(window)"
            class="action-btn"
            title="Rename window"
          >
            <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
            </svg>
          </button>
          <button
            @click.stop="killWindow(window)"
            class="action-btn"
            title="Kill window"
          >
            <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
      </div>
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

defineEmits<{
  'select-window': [window: TmuxWindow]
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
      // Session changed while loading windows, ignoring stale data
    }
  } catch (err) {
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
  } catch (err) {
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
  } catch (err) {
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
      // Failed to rename window
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
      // Window selected, refresh windows for session
      // Small delay to ensure tmux has updated
      setTimeout(() => {
        loadWindows(false) // Don't show loading for updates
      }, 100)
    }
  })
  
  // Listen for windows-list broadcasts but only update if it's for our session
  ws.onMessage<WindowsListMessage>('windows-list', (data) => {
    if (data.sessionName === props.sessionName) {
      // Received window list update for our session
      windows.value = data.windows
      error.value = false
      loading.value = false
    } else {
      // Ignoring window list for different session
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
    // Session changed - reload windows
    
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
  refresh: () => loadWindows(false), // Don't show loading on manual refresh
  createWindow
})
</script>

<style scoped>
/* Window list container */
.window-list {
  /* Remove padding - windows will handle their own spacing */
}

/* Status messages */
.window-status {
  @apply text-xs py-1;
  padding-left: 32px;
  color: var(--text-tertiary);
}

.window-status.error {
  color: var(--accent-danger);
}

/* Window items container */
.window-items {
  @apply space-y-0;
}

/* Window item */
.window-item {
  @apply relative flex items-center justify-between py-1 cursor-pointer;
  @apply transition-all duration-150;
  min-height: 26px;
  margin: 2px 0;
  padding-left: 28px;
  padding-right: 8px;
  border-radius: 0 4px 4px 0;
  gap: 8px;
}

.window-item:hover:not(.window-active) {
  background: rgba(255, 255, 255, 0.02);
  padding-left: 30px;
}

.window-item.window-active {
  background: rgba(88, 166, 255, 0.1);
  padding-left: 32px;
  margin: 2px 4px;
  border-radius: 4px;
  border: 1px solid rgba(88, 166, 255, 0.2);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.window-item.window-active::before {
  content: '';
  position: absolute;
  left: 8px;
  top: 50%;
  transform: translateY(-50%);
  width: 4px;
  height: 4px;
  border-radius: 50%;
  background: var(--accent-primary);
  box-shadow: 0 0 8px rgba(88, 166, 255, 0.4);
}

/* Window content wrapper */
.window-content {
  @apply flex items-center gap-2 flex-1;
}

/* Window label */
.window-label {
  @apply flex items-center gap-1.5 min-w-0;
}

/* Window icon */
.window-icon {
  @apply w-3.5 h-3.5 flex-shrink-0;
  stroke: var(--text-tertiary);
  transition: stroke 150ms ease;
}

.window-active .window-icon {
  stroke: var(--text-primary);
}

/* Window name */
.window-name {
  @apply text-xs truncate;
  color: var(--text-secondary);
  font-size: 12px;
  transition: color 150ms ease;
  flex: 1;
}

.window-active .window-name {
  color: var(--text-primary);
  font-weight: 500;
}

.window-name-input {
  @apply px-1 py-0 text-xs flex-1;
  background: var(--bg-primary);
  border: 1px solid var(--accent-primary);
  color: var(--text-primary);
  outline: none;
  border-radius: 2px;
  font-size: 12px;
  min-width: 100px;
}

/* Pane count badge */
.pane-count-badge {
  @apply flex items-center gap-0.5 px-1.5 py-0.5 rounded-full;
  background: var(--bg-tertiary);
  color: var(--text-tertiary);
  font-size: 10px;
  opacity: 0.6;
  transition: all 150ms ease;
}

.pane-count-badge:hover {
  opacity: 0.8;
}

.window-active .pane-count-badge {
  opacity: 0.9;
  background: rgba(88, 166, 255, 0.08);
  color: var(--text-secondary);
}

/* Window actions */
.window-actions {
  @apply flex items-center gap-0.5 opacity-0;
  transition: opacity 150ms ease;
}

.window-item:hover .window-actions {
  opacity: 1;
}

.action-btn {
  @apply p-0.5 rounded;
  color: var(--text-tertiary);
  transition: all 150ms ease;
}

.action-btn:hover {
  background: rgba(255, 255, 255, 0.08);
  color: var(--text-secondary);
}

</style>