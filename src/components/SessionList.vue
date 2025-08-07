<template>
  <aside 
    class="flex flex-col border-r transition-all duration-300" 
    :class="[
      isMobile ? (isCollapsed ? 'w-0 overflow-hidden' : 'w-64') : (isCollapsed ? 'w-12' : 'w-64'),
      isMobile && !isCollapsed ? 'shadow-xl' : ''
    ]"
    style="background: var(--bg-secondary); border-color: var(--border-primary)"
  >
    <!-- Modal for delete confirmation -->
    <div v-if="showDeleteModal" class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50" @click.self="cancelDelete">
      <div class="p-6 rounded-lg shadow-xl max-w-sm w-full mx-4" style="background: var(--bg-secondary); border: 1px solid var(--border-primary)">
        <h3 class="text-lg font-semibold mb-4" style="color: var(--text-primary)">{{ deleteModalTitle }}</h3>
        <p class="mb-4" style="color: var(--text-secondary)">
          {{ deleteModalMessage }}
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

    <!-- Modal for session name input -->
    <div v-if="showCreateModal" class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div class="bg-white dark:bg-gray-800 p-6 rounded-lg shadow-xl max-w-sm w-full mx-4">
        <h3 class="text-lg font-semibold mb-4" style="color: var(--text-primary)">Create New Session</h3>
        <input 
          v-model="newSessionName"
          type="text" 
          placeholder="Session name"
          class="w-full px-3 py-2 border rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
          style="background: var(--bg-primary); border-color: var(--border-primary); color: var(--text-primary)"
          @keyup.enter="confirmCreate"
          ref="sessionNameInput"
        />
        <div class="flex justify-end space-x-2 mt-4">
          <button 
            @click="cancelCreate"
            class="px-4 py-2 text-sm border rounded"
            style="background: var(--bg-secondary); border-color: var(--border-primary); color: var(--text-secondary)"
          >
            Cancel
          </button>
          <button 
            @click="confirmCreate"
            class="px-4 py-2 text-sm border rounded"
            style="background: var(--bg-primary); border-color: var(--border-primary); color: var(--text-primary)"
          >
            Create
          </button>
        </div>
      </div>
    </div>
    <div class="p-3 border-b" style="border-color: var(--border-primary)">
      <div class="flex items-center justify-between mb-3">
        <h2 
          v-if="!isCollapsed || isMobile" 
          class="text-xs font-medium" 
          style="color: var(--text-secondary)"
        >
          Sessions ({{ sessions.length }})
        </h2>
        
      </div>
      
      <button
        v-if="!isCollapsed || isMobile"
        @click="handleCreate"
        class="w-full px-3 py-1.5 text-xs border rounded transition-colors mb-3"
        style="background: var(--bg-primary); border-color: var(--border-primary); color: var(--text-primary)"
        :class="'hover:border-opacity-80'"
      >
        New Session
      </button>
      
      <!-- Collapsed state new session button (desktop only) -->
      <button
        v-else-if="!isMobile"
        @click="handleCreate"
        class="w-full p-1.5 hover-bg rounded transition-colors flex items-center justify-center mb-3"
        style="color: var(--text-tertiary)"
        title="New Session"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
        </svg>
      </button>
      
      <!-- Audio Control -->
      <AudioControl v-if="!isCollapsed || isMobile" />
    </div>

    <div class="flex-1 overflow-y-auto">
      <div v-if="isLoading && sessions.length === 0" class="p-6 text-center">
        <div class="animate-pulse">
          <div v-if="!isCollapsed" class="text-xs" style="color: var(--text-tertiary)">Loading sessions...</div>
          <div v-else class="w-8 h-8 mx-auto rounded-full" style="background: var(--bg-tertiary)"></div>
        </div>
      </div>
      
      <div v-else-if="sessions.length === 0" class="p-6 text-center">
        <p v-if="!isCollapsed" class="text-xs" style="color: var(--text-tertiary)">No sessions</p>
        <div v-else class="text-xs" style="color: var(--text-tertiary)" title="No sessions">—</div>
      </div>
      
      <div v-else class="py-1">
        <SessionItem
          v-for="session in sessions"
          :key="session.name"
          v-memo="[session.name, session.windows, currentSession === session.name, isCollapsed && !isMobile]"
          :session="session"
          :isActive="currentSession === session.name"
          :isCollapsed="isCollapsed && !isMobile"
          :isMobile="isMobile"
          @select="$emit('select', session.name)"
          @kill="handleKill(session.name)"
          @rename="(newName) => emit('rename', session.name, newName)"
          @select-window="(window) => $emit('select-window', session.name, window)"
        />
      </div>
    </div>
    
    <!-- CRON Section -->
    <CronSection :isCollapsed="isCollapsed && !isMobile" />
    
    <!-- Dotfiles Section -->
    <DotfilesSection :isCollapsed="isCollapsed && !isMobile" />
  </aside>
</template>

<script setup lang="ts">
import { ref, nextTick } from 'vue'
import SessionItem from './SessionItem.vue'
import AudioControl from './AudioControl.vue'
import CronSection from './CronSection.vue'
import DotfilesSection from './DotfilesSection.vue'
import type { TmuxSession, TmuxWindow } from '@/types'

interface Props {
  sessions: TmuxSession[]
  currentSession: string | null
  isCollapsed: boolean
  isMobile: boolean
  isLoading: boolean
}

const props = withDefaults(defineProps<Props>(), {
  sessions: () => [],
  currentSession: null,
  isCollapsed: false,
  isMobile: false,
  isLoading: false
})

const emit = defineEmits<{
  select: [sessionName: string]
  kill: [sessionName: string]
  rename: [sessionName: string, newName: string]
  create: [sessionName: string]
  'select-window': [sessionName: string, window: TmuxWindow]
  'toggle-sidebar': []
}>()

// Modal state
const showCreateModal = ref(false)
const newSessionName = ref('')
const sessionNameInput = ref<HTMLInputElement>()

// Delete modal state
const showDeleteModal = ref(false)
const sessionToDelete = ref<string | null>(null)
const deleteModalTitle = ref('')
const deleteModalMessage = ref('')

const handleCreate = (): void => {
  console.log('handleCreate called')
  showCreateModal.value = true
  newSessionName.value = `s${Date.now().toString().slice(-6)}`
  nextTick(() => {
    sessionNameInput.value?.focus()
    sessionNameInput.value?.select()
  })
}

const confirmCreate = (): void => {
  if (newSessionName.value.trim()) {
    console.log('Creating session with name:', newSessionName.value)
    emit('create', newSessionName.value.trim())
    showCreateModal.value = false
    newSessionName.value = ''
  }
}

const cancelCreate = (): void => {
  showCreateModal.value = false
  newSessionName.value = ''
}

const handleKill = (sessionName: string): void => {
  console.log('handleKill called for session:', sessionName)
  const session = props.sessions.find(s => s.name === sessionName)
  if (!session) {
    console.error('Session not found:', sessionName)
    return
  }
  
  sessionToDelete.value = sessionName
  deleteModalTitle.value = session.windows === 1 ? 'Close Session' : 'Kill Session'
  deleteModalMessage.value = session.windows === 1 
    ? `Are you sure you want to close session "${sessionName}"?`
    : `Are you sure you want to kill session "${sessionName}"? This will close all ${session.windows} windows.`
  
  showDeleteModal.value = true
}

const confirmDelete = (): void => {
  if (sessionToDelete.value) {
    console.log('User confirmed kill for session:', sessionToDelete.value)
    emit('kill', sessionToDelete.value)
    showDeleteModal.value = false
    sessionToDelete.value = null
  }
}

const cancelDelete = (): void => {
  console.log('User cancelled delete')
  showDeleteModal.value = false
  sessionToDelete.value = null
}
</script>