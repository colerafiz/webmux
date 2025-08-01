<template>
  <aside 
    class="flex flex-col border-r transition-all duration-300" 
    :class="[
      isMobile ? (isCollapsed ? 'w-0 overflow-hidden' : 'w-64') : (isCollapsed ? 'w-12' : 'w-64'),
      isMobile && !isCollapsed ? 'shadow-xl' : ''
    ]"
    style="background: var(--bg-secondary); border-color: var(--border-primary)"
  >
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
        
        <div class="flex items-center space-x-1">
          <button
            @click="$emit('refresh')"
            class="p-1 hover-bg rounded text-xs"
            style="color: var(--text-tertiary)"
            :title="isCollapsed ? 'Refresh Sessions' : 'Refresh'"
          >
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
          </button>
          
          <button
            @click="$emit('toggle-sidebar')"
            class="p-1 hover-bg rounded text-xs"
            style="color: var(--text-tertiary)"
            :title="isCollapsed ? 'Expand Sidebar' : 'Collapse Sidebar'"
          >
            <svg class="w-3.5 h-3.5 transition-transform duration-200" :class="{ 'rotate-180': isCollapsed }" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
            </svg>
          </button>
        </div>
      </div>
      
      <button
        v-if="!isCollapsed || isMobile"
        @click="handleCreate"
        class="w-full px-3 py-1.5 text-xs border rounded transition-colors"
        style="background: var(--bg-primary); border-color: var(--border-primary); color: var(--text-primary)"
        :class="'hover:border-opacity-80'"
      >
        New Session
      </button>
      
      <!-- Collapsed state new session button (desktop only) -->
      <button
        v-else-if="!isMobile"
        @click="handleCreate"
        class="w-full p-1.5 hover-bg rounded transition-colors flex items-center justify-center"
        style="color: var(--text-tertiary)"
        title="New Session"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
        </svg>
      </button>
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
          :session="session"
          :isActive="currentSession === session.name"
          :isCollapsed="isCollapsed && !isMobile"
          :isMobile="isMobile"
          @select="$emit('select', session.name)"
          @kill="handleKill(session.name)"
          @rename="(newName) => emit('rename', session.name, newName)"
          @select-window="(window) => $emit('select-window', session.name, window)"
          @refresh="$emit('refresh')"
        />
      </div>
    </div>
    
    <!-- Audio control at bottom of sidebar -->
    <AudioControl :isCollapsed="isCollapsed && !isMobile" />
  </aside>
</template>

<script setup lang="ts">
import { ref, nextTick } from 'vue'
import SessionItem from './SessionItem.vue'
import AudioControl from './AudioControl.vue'
import { tmuxApi } from '@/api/tmux'
import type { TmuxSession, TmuxWindow } from '@/types'

interface Props {
  sessions: TmuxSession[]
  currentSession: string | null
  isCollapsed: boolean
  isMobile: boolean
  isLoading: boolean
}

withDefaults(defineProps<Props>(), {
  sessions: () => [],
  currentSession: null,
  isCollapsed: false,
  isMobile: false,
  isLoading: false
})

const emit = defineEmits<{
  select: [sessionName: string]
  refresh: []
  rename: [sessionName: string, newName: string]
  create: [sessionName: string]
  'select-window': [sessionName: string, window: TmuxWindow]
  'toggle-sidebar': []
}>()

// Modal state
const showCreateModal = ref(false)
const newSessionName = ref('')
const sessionNameInput = ref<HTMLInputElement>()

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

const handleKill = async (sessionName: string): Promise<void> => {
  const session = props.sessions.find(s => s.name === sessionName)
  if (!session) return
  
  const confirmMessage = session.windows === 1 
    ? `Close session "${sessionName}"?`
    : `Kill session "${sessionName}"?\n\nThis will close all ${session.windows} windows.`
    
  if (confirm(confirmMessage)) {
    try {
      if (session.windows === 1) {
        // If only one window, kill the window instead (which kills the session)
        console.log('Killing window 0 in session:', sessionName)
        await tmuxApi.killWindow(sessionName, 0)
      } else {
        // Multiple windows, kill the entire session
        console.log('Killing session:', sessionName)
        await tmuxApi.killSession(sessionName)
      }
      
      // Wait a moment for tmux to process
      await new Promise(resolve => setTimeout(resolve, 100))
      
      // If we killed the current session, clear it
      if (props.currentSession === sessionName) {
        emit('select', '')
      }
      
      // Force refresh the sessions list
      emit('refresh')
    } catch (error: any) {
      console.error('Failed to kill session:', error)
      alert(`Failed to kill session: ${error.response?.data?.error || error.message}`)
    }
  }
}
</script>