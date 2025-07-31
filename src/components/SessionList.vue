<template>
  <aside 
    class="flex flex-col border-r transition-all duration-300" 
    :class="[
      isMobile ? (isCollapsed ? 'w-0 overflow-hidden' : 'w-64') : (isCollapsed ? 'w-12' : 'w-64'),
      isMobile && !isCollapsed ? 'shadow-xl' : ''
    ]"
    style="background: var(--bg-secondary); border-color: var(--border-primary)"
  >
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
      <div v-if="sessions.length === 0" class="p-6 text-center">
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
  </aside>
</template>

<script setup lang="ts">
import SessionItem from './SessionItem.vue'
import type { TmuxSession, TmuxWindow } from '@/types'

interface Props {
  sessions: TmuxSession[]
  currentSession: string | null
  isCollapsed: boolean
  isMobile: boolean
}

withDefaults(defineProps<Props>(), {
  sessions: () => [],
  currentSession: null,
  isCollapsed: false,
  isMobile: false
})

const emit = defineEmits<{
  select: [sessionName: string]
  refresh: []
  kill: [sessionName: string]
  rename: [sessionName: string, newName: string]
  create: [sessionName: string]
  'select-window': [sessionName: string, window: TmuxWindow]
  'toggle-sidebar': []
}>()

const handleCreate = (): void => {
  const sessionName = prompt('Session name:', `s${Date.now().toString().slice(-6)}`)
  if (sessionName) {
    emit('create', sessionName)
  }
}

const handleKill = (sessionName: string): void => {
  if (confirm(`Are you sure you want to kill session "${sessionName}"?`)) {
    emit('kill', sessionName)
  }
}
</script>