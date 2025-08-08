<template>
  <div class="window-list">
    <!-- Modals - keep existing modal code -->
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

    <!-- Status messages -->
    <div v-if="loading" class="window-status">
      Loading...
    </div>
    <div v-else-if="error" class="window-status">
      Error
    </div>
    <div v-else-if="windows.length === 0" class="window-status">
      No windows
    </div>
    <!-- Window list -->
    <div v-else>
      <div
        v-for="window in windows"
        :key="window.index"
        @click="$emit('select-window', window)"
        class="window-item"
        :class="{ 'active': window.active && props.isActiveSession }"
      >
        <span v-if="!isEditing(window)" class="window-name">
          {{ window.name }}{{ window.panes > 1 ? ` (${window.panes}p)` : '' }}
        </span>
        <input
          v-else
          v-model="editingName"
          @keyup.enter="confirmRename(window)"
          @keyup.escape="cancelEdit"
          @blur="confirmRename(window)"
          ref="editInput"
          class="window-name-input"
          @click.stop
        />
        
        <div class="window-actions">
          <button
            @click.stop="startEdit(window)"
            class="window-action-btn"
            title="Rename"
          >
            <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M11 4H4a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2v-7M18.5 2.5a2.121 2.121 0 013 3L12 15l-4 1 1-4 9.5-9.5z"/>
            </svg>
          </button>
          <button
            @click.stop="killWindow(window)"
            class="window-action-btn"
            title="Kill"
          >
            <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M18 6L6 18M6 6l12 12"/>
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
const loading = ref<boolean>(false)
const error = ref<boolean>(false)
const editingWindow = ref<TmuxWindow | null>(null)
const editingName = ref<string>('')
const editInput = ref<HTMLInputElement | null>(null)

let hasLoadedInitial = false

const showCreateModal = ref(false)
const newWindowName = ref('')
const windowNameInput = ref<HTMLInputElement>()

const showDeleteModal = ref(false)
const windowToDelete = ref<TmuxWindow | null>(null)

const ws = useWebSocket()

const loadWindows = async (showLoading: boolean = true): Promise<void> => {
  const loadingForSession = props.sessionName
  
  try {
    if (showLoading && !hasLoadedInitial) {
      loading.value = true
    }
    error.value = false
    const loadedWindows = await websocketApi.getWindows(props.sessionName)
    
    if (props.sessionName === loadingForSession) {
      windows.value = loadedWindows
      hasLoadedInitial = true
    }
  } catch (err) {
    if (props.sessionName === loadingForSession) {
      error.value = true
      console.error('Failed to load windows for session:', props.sessionName, err)
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
  const optimisticWindow: TmuxWindow = {
    index: windows.value.length,
    name: newWindowName.value || `Window ${windows.value.length}`,
    active: false,
    panes: 1
  }
  
  windows.value = [...windows.value, optimisticWindow]
  showCreateModal.value = false
  const savedName = newWindowName.value
  newWindowName.value = ''
  
  try {
    await websocketApi.createWindow(props.sessionName, savedName || undefined)
  } catch (err) {
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
  
  windows.value = windows.value.filter(w => w.index !== windowToRemove.index)
  showDeleteModal.value = false
  windowToDelete.value = null
  
  try {
    await websocketApi.killWindow(props.sessionName, windowToRemove.index)
  } catch (err) {
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
      await loadWindows(false)
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
  showCreateModal.value = false
  showDeleteModal.value = false
  windowToDelete.value = null
  newWindowName.value = ''
  
  loadWindows()
  
  ws.onMessage<WindowSelectedMessage>('window-selected', (data) => {
    if (data.success) {
      setTimeout(() => {
        loadWindows(false)
      }, 100)
    }
  })
  
  ws.onMessage<WindowsListMessage>('windows-list', (data) => {
    if (data.sessionName === props.sessionName) {
      windows.value = data.windows
      error.value = false
      loading.value = false
    }
  })
})

onUnmounted(() => {
  ws.offMessage('window-selected')
  ws.offMessage('windows-list')
  
  if (sessionChangeTimeout) {
    clearTimeout(sessionChangeTimeout)
    sessionChangeTimeout = null
  }
})

let sessionChangeTimeout: ReturnType<typeof setTimeout> | null = null

watch(() => props.sessionName, (newSessionName, oldSessionName) => {
  if (newSessionName !== oldSessionName) {
    if (sessionChangeTimeout) {
      clearTimeout(sessionChangeTimeout)
    }
    
    hasLoadedInitial = false
    windows.value = []
    error.value = false
    loading.value = true
    
    sessionChangeTimeout = setTimeout(() => {
      loadWindows()
    }, 100)
  }
})

defineExpose({
  refresh: () => loadWindows(false),
  createWindow
})
</script>

<style scoped>
.window-list {
}

.window-status {
  padding: 0 16px;
  font-size: 11px;
  color: var(--text-tertiary);
  opacity: 0.6;
}

.window-item {
  display: flex;
  align-items: center;
  height: 24px;
  padding: 0 16px;
  cursor: pointer;
  position: relative;
  color: var(--text-tertiary);
  font-size: 12px;
  transition: background 100ms ease;
}

.window-item:hover {
  background: rgba(255, 255, 255, 0.02);
  color: var(--text-secondary);
}

.window-item.active {
  background: rgba(88, 166, 255, 0.06);
  color: var(--text-primary);
}

.window-item.active::before {
  content: '';
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 1px;
  background: var(--accent-primary);
}

.window-name {
  flex: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.window-name-input {
  flex: 1;
  padding: 1px 3px;
  font-size: 12px;
  background: var(--bg-primary);
  border: 1px solid var(--accent-primary);
  color: var(--text-primary);
  outline: none;
  border-radius: 2px;
}


.window-actions {
  display: flex;
  gap: 2px;
  margin-left: 8px;
  opacity: 0;
  transition: opacity 100ms ease;
}

.window-item:hover .window-actions {
  opacity: 1;
}

.window-action-btn {
  width: 16px;
  height: 16px;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: none;
  color: inherit;
  opacity: 0.6;
  border-radius: 2px;
  transition: all 100ms ease;
}

.window-action-btn:hover {
  opacity: 1;
  background: rgba(255, 255, 255, 0.06);
}
</style>