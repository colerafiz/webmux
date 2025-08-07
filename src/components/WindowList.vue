<template>
  <div class="window-list">
    <!-- Modal for window name input -->
    <div v-if="showCreateModal" class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50" @click.self="cancelCreateWindow">
      <div class="p-6 rounded-xl shadow-2xl max-w-sm w-full mx-4" style="background: linear-gradient(135deg, rgba(30, 41, 59, 0.95) 0%, rgba(30, 41, 59, 0.85) 100%); border: 1px solid rgba(148, 163, 184, 0.2); backdrop-filter: blur(20px)">
        <h3 class="text-lg font-semibold mb-4" style="color: var(--text-primary)">Create New Window</h3>
        <input 
          v-model="newWindowName"
          type="text" 
          placeholder="Window name (optional)"
          class="w-full px-3 py-2 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 transition-all duration-150"
          style="background: rgba(0, 0, 0, 0.3); border: 1px solid rgba(148, 163, 184, 0.2); color: var(--text-primary)"
          @keyup.enter="confirmCreateWindow"
          ref="windowNameInput"
        />
        <div class="flex justify-end space-x-2 mt-4">
          <button 
            @click="cancelCreateWindow"
            class="px-4 py-2 text-sm rounded-lg transition-all duration-150"
            style="background: rgba(148, 163, 184, 0.1); color: var(--text-secondary)"
            onmouseover="this.style.background='rgba(148, 163, 184, 0.2)'"
            onmouseout="this.style.background='rgba(148, 163, 184, 0.1)'"
          >
            Cancel
          </button>
          <button 
            @click="confirmCreateWindow"
            class="px-4 py-2 text-sm rounded-lg transition-all duration-150 font-medium"
            style="background: rgba(59, 130, 246, 0.9); color: white"
            onmouseover="this.style.background='rgba(59, 130, 246, 1)'"
            onmouseout="this.style.background='rgba(59, 130, 246, 0.9)'"
          >
            Create
          </button>
        </div>
      </div>
    </div>
    
    <!-- Modal for delete confirmation -->
    <div v-if="showDeleteModal" class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50" @click.self="cancelDelete">
      <div class="p-6 rounded-xl shadow-2xl max-w-sm w-full mx-4" style="background: linear-gradient(135deg, rgba(30, 41, 59, 0.95) 0%, rgba(30, 41, 59, 0.85) 100%); border: 1px solid rgba(148, 163, 184, 0.2); backdrop-filter: blur(20px)">
        <h3 class="text-lg font-semibold mb-4" style="color: var(--text-primary)">Delete Window</h3>
        <p class="mb-4" style="color: var(--text-secondary)">
          Are you sure you want to kill window "{{ windowToDelete?.name }}"?
        </p>
        <div class="flex justify-end space-x-2">
          <button 
            @click="cancelDelete"
            class="px-4 py-2 text-sm rounded-lg transition-all duration-150"
            style="background: rgba(148, 163, 184, 0.1); color: var(--text-secondary)"
            onmouseover="this.style.background='rgba(148, 163, 184, 0.2)'"
            onmouseout="this.style.background='rgba(148, 163, 184, 0.1)'"
          >
            Cancel
          </button>
          <button 
            @click="confirmDelete"
            class="px-4 py-2 text-sm rounded-lg transition-all duration-150 font-medium"
            style="background: rgba(248, 81, 73, 0.9); color: white"
            onmouseover="this.style.background='rgba(248, 81, 73, 1)'"
            onmouseout="this.style.background='rgba(248, 81, 73, 0.9)'"
          >
            Delete
          </button>
        </div>
      </div>
    </div>

    <div v-if="loading" class="window-status">
      <div class="loading-spinner"></div>
      <span>Loading windows...</span>
    </div>
    <div v-else-if="error" class="window-status error">
      <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
      </svg>
      <span>Error loading windows</span>
    </div>
    <div v-else-if="windows.length === 0" class="window-status">
      <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4" />
      </svg>
      <span>No windows</span>
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
        <!-- Window Content -->
        <div class="window-main">
          <!-- Window Icon -->
          <div class="window-icon-wrapper">
            <svg class="window-icon" fill="none" viewBox="0 0 20 20" stroke="currentColor" stroke-width="1.5">
              <path stroke-linecap="round" stroke-linejoin="round" d="M3 4a1 1 0 011-1h12a1 1 0 011 1v12a1 1 0 01-1 1H4a1 1 0 01-1-1V4z" />
              <path stroke-linecap="round" stroke-linejoin="round" d="M3 9h14" />
            </svg>
          </div>
          
          <!-- Window Info -->
          <div class="window-info">
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
            <span class="window-meta">
              {{ window.panes }} {{ window.panes === 1 ? 'pane' : 'panes' }}
            </span>
          </div>
        </div>
        
        <!-- Actions -->
        <div class="window-actions">
          <button
            @click.stop="startEdit(window)"
            class="window-action-btn"
            title="Rename window"
          >
            <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 20 20" stroke="currentColor" stroke-width="2">
              <path stroke-linecap="round" stroke-linejoin="round" d="M13.586 3.586a2 2 0 112.828 2.828l-.793.793-2.828-2.828.793-.793zM11.379 5.793L3 14.172V17h2.828l8.38-8.379-2.83-2.828z" />
            </svg>
          </button>
          <button
            @click.stop="killWindow(window)"
            class="window-action-btn danger"
            title="Kill window"
          >
            <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 20 20" stroke="currentColor" stroke-width="2">
              <path stroke-linecap="round" stroke-linejoin="round" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" />
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
  @apply px-3 pb-3;
}

/* Status messages */
.window-status {
  @apply flex items-center justify-center gap-2 py-8 text-xs;
  color: var(--text-tertiary);
  opacity: 0.6;
}

.window-status.error {
  color: var(--accent-danger);
}

/* Loading spinner */
.loading-spinner {
  @apply w-4 h-4 border-2 rounded-full;
  border-color: var(--text-tertiary);
  border-top-color: transparent;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

/* Window items container */
.window-items {
  @apply space-y-2;
}

/* Window item - modern nested card */
.window-item {
  @apply relative px-4 py-3 rounded-lg cursor-pointer overflow-hidden;
  @apply transition-all duration-200 ease-out;
  background: rgba(203, 213, 225, 0.05);
  border: 1px solid rgba(203, 213, 225, 0.08);
  margin-left: 20px;
}

.window-item::before {
  content: '';
  position: absolute;
  left: -20px;
  top: 50%;
  width: 16px;
  height: 1px;
  background: rgba(148, 163, 184, 0.2);
}

.window-item:hover {
  background: rgba(203, 213, 225, 0.08);
  border-color: rgba(203, 213, 225, 0.15);
  transform: translateX(2px);
}

.window-item.window-active {
  background: linear-gradient(135deg, rgba(59, 130, 246, 0.08) 0%, rgba(59, 130, 246, 0.03) 100%);
  border-color: rgba(59, 130, 246, 0.2);
  box-shadow: inset 0 1px 2px rgba(59, 130, 246, 0.05);
}

/* Window main content */
.window-main {
  @apply flex items-center gap-3;
}

/* Window icon */
.window-icon-wrapper {
  @apply p-1.5 rounded;
  background: rgba(148, 163, 184, 0.08);
}

.window-icon {
  @apply w-4 h-4;
  color: var(--text-tertiary);
}

.window-active .window-icon-wrapper {
  background: rgba(59, 130, 246, 0.1);
}

.window-active .window-icon {
  color: #3b82f6;
}

/* Window info */
.window-info {
  @apply flex-1 min-w-0;
}

.window-name {
  @apply block text-sm font-medium truncate mb-0.5;
  color: var(--text-primary);
}

.window-active .window-name {
  color: #3b82f6;
}

.window-name-input {
  @apply px-2 py-0.5 text-sm font-medium w-full rounded;
  background: rgba(0, 0, 0, 0.2);
  border: 1px solid rgba(59, 130, 246, 0.5);
  color: var(--text-primary);
  outline: none;
  margin-bottom: 2px;
}

.window-meta {
  @apply text-xs;
  color: var(--text-tertiary);
  opacity: 0.7;
}

/* Window actions */
.window-actions {
  @apply flex items-center gap-1 ml-auto opacity-0 transition-opacity duration-200;
}

.window-item:hover .window-actions {
  opacity: 1;
}

.window-action-btn {
  @apply p-1 rounded transition-all duration-150;
  background: rgba(148, 163, 184, 0.08);
  color: var(--text-tertiary);
}

.window-action-btn:hover {
  background: rgba(148, 163, 184, 0.15);
  color: var(--text-secondary);
  transform: scale(1.1);
}

.window-action-btn.danger:hover {
  background: rgba(239, 68, 68, 0.1);
  color: #ef4444;
}
</style>