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
        class="flex items-center justify-between px-2 py-1 rounded cursor-pointer hover-bg text-xs"
        :class="{ 'bg-opacity-30': window.active }"
        :style="{
          background: window.active ? 'var(--bg-tertiary)' : 'transparent',
          borderLeft: window.active ? '2px solid var(--accent-secondary)' : '2px solid transparent'
        }"
      >
        <div class="flex items-center space-x-2 min-w-0">
          <span class="font-mono" style="color: var(--text-tertiary)">{{ window.index }}:</span>
          <span v-if="!isEditing(window)" class="truncate" :style="{ color: window.active ? 'var(--text-primary)' : 'var(--text-secondary)' }">
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
import { ref, onMounted, nextTick } from 'vue'
import { tmuxApi } from '@/api/tmux'
import type { TmuxWindow } from '@/types'

interface Props {
  sessionName: string
}

const props = defineProps<Props>()

const emit = defineEmits<{
  'select-window': [window: TmuxWindow]
  refresh: []
}>()

const windows = ref<TmuxWindow[]>([])
const loading = ref<boolean>(true)
const error = ref<boolean>(false)
const editingWindow = ref<TmuxWindow | null>(null)
const editingName = ref<string>('')
const editInput = ref<HTMLInputElement | null>(null)

// Modal state for new window
const showCreateModal = ref(false)
const newWindowName = ref('')
const windowNameInput = ref<HTMLInputElement>()

// Modal state for delete confirmation
const showDeleteModal = ref(false)
const windowToDelete = ref<TmuxWindow | null>(null)

const loadWindows = async (): Promise<void> => {
  try {
    loading.value = true
    error.value = false
    windows.value = await tmuxApi.getWindows(props.sessionName)
  } catch (err) {
    error.value = true
    console.error('Failed to load windows:', err)
  } finally {
    loading.value = false
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
  try {
    await tmuxApi.createWindow(props.sessionName, newWindowName.value || undefined)
    await loadWindows()
    emit('refresh')
    showCreateModal.value = false
    newWindowName.value = ''
  } catch (err) {
    console.error('Failed to create window:', err)
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
  
  try {
    await tmuxApi.killWindow(props.sessionName, windowToDelete.value.index)
    await loadWindows()
    emit('refresh')
    showDeleteModal.value = false
    windowToDelete.value = null
  } catch (err) {
    console.error('Failed to kill window:', err)
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
      await tmuxApi.renameWindow(props.sessionName, window.index, editingName.value)
      await loadWindows()
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
})

defineExpose({
  refresh: loadWindows
})
</script>