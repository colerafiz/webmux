<template>
  <div class="pl-6 mt-1">
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

<script setup>
import { ref, onMounted, nextTick } from 'vue'
import { tmuxApi } from '../api/tmux'

const props = defineProps({
  sessionName: {
    type: String,
    required: true
  }
})

const emit = defineEmits(['select-window', 'refresh'])

const windows = ref([])
const loading = ref(true)
const error = ref(false)
const editingWindow = ref(null)
const editingName = ref('')
const editInput = ref(null)

const loadWindows = async () => {
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

const createWindow = async () => {
  const name = prompt('Window name (optional):')
  if (name !== null) {
    try {
      await tmuxApi.createWindow(props.sessionName, name || undefined)
      await loadWindows()
      emit('refresh')
    } catch (err) {
      console.error('Failed to create window:', err)
    }
  }
}

const killWindow = async (window) => {
  if (confirm(`Kill window "${window.name}"?`)) {
    try {
      await tmuxApi.killWindow(props.sessionName, window.index)
      await loadWindows()
      emit('refresh')
    } catch (err) {
      console.error('Failed to kill window:', err)
    }
  }
}

const isEditing = (window) => {
  return editingWindow.value?.index === window.index
}

const startEdit = (window) => {
  editingWindow.value = window
  editingName.value = window.name
  nextTick(() => {
    editInput.value?.focus()
    editInput.value?.select()
  })
}

const confirmRename = async (window) => {
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

const cancelEdit = () => {
  editingWindow.value = null
  editingName.value = ''
}

onMounted(() => {
  loadWindows()
})

defineExpose({
  refresh: loadWindows
})
</script>