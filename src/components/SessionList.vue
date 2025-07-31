<template>
  <aside class="w-64 flex flex-col border-r" style="background: var(--bg-secondary); border-color: var(--border-primary)">
    <div class="p-3 border-b" style="border-color: var(--border-primary)">
      <div class="flex items-center justify-between mb-3">
        <h2 class="text-xs font-medium" style="color: var(--text-secondary)">Sessions ({{ sessions.length }})</h2>
        <button
          @click="$emit('refresh')"
          class="p-1 hover-bg rounded text-xs"
          style="color: var(--text-tertiary)"
          title="Refresh"
        >
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
        </button>
      </div>
      
      <button
        @click="handleCreate"
        class="w-full px-3 py-1.5 text-xs border rounded transition-colors"
        style="background: var(--bg-primary); border-color: var(--border-primary); color: var(--text-primary)"
        :class="'hover:border-opacity-80'"
      >
        New Session
      </button>
    </div>

    <div class="flex-1 overflow-y-auto">
      <div v-if="sessions.length === 0" class="p-6 text-center">
        <p class="text-xs" style="color: var(--text-tertiary)">No sessions</p>
      </div>
      
      <div v-else class="py-1">
        <SessionItem
          v-for="session in sessions"
          :key="session.name"
          :session="session"
          :isActive="currentSession === session.name"
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

<script setup>
import { ref } from 'vue'
import SessionItem from './SessionItem.vue'

defineProps({
  sessions: {
    type: Array,
    default: () => []
  },
  currentSession: {
    type: String,
    default: null
  }
})

const emit = defineEmits(['select', 'refresh', 'kill', 'rename', 'create', 'select-window'])

const handleCreate = () => {
  const sessionName = prompt('Session name:', `s${Date.now().toString().slice(-6)}`)
  if (sessionName) {
    emit('create', sessionName)
  }
}

const handleKill = (sessionName) => {
  if (confirm(`Are you sure you want to kill session "${sessionName}"?`)) {
    emit('kill', sessionName)
  }
}
</script>