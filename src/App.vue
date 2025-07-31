<template>
  <div class="min-h-screen" style="background: var(--bg-primary)">
    <header class="border-b safe-area-top" style="background: var(--bg-secondary); border-color: var(--border-primary)">
      <div class="px-4 safe-area-left safe-area-right">
        <div class="flex items-center justify-between h-12">
          <div class="flex items-center space-x-3 md:space-x-6">
            <button
              v-if="isMobile"
              @click="sidebarCollapsed = false"
              class="p-1.5 hover-bg rounded md:hidden"
              style="color: var(--text-tertiary)"
            >
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
              </svg>
            </button>
            <h1 class="text-sm font-medium">webmux</h1>
            <div class="hidden sm:flex items-center space-x-4 text-xs" style="color: var(--text-secondary)">
              <span class="hidden md:inline">{{ stats.hostname }}</span>
              <span>{{ stats.platform }}/{{ stats.arch }}</span>
            </div>
          </div>
          
          <div class="flex items-center space-x-3 md:space-x-6 text-xs">
            <div class="flex items-center space-x-2 md:space-x-4">
              <div class="flex items-center space-x-1 md:space-x-2">
                <span class="hidden sm:inline" style="color: var(--text-tertiary)">CPU</span>
                <span class="stat-badge">{{ stats.cpu.loadAvg?.[0]?.toFixed(2) || '0.00' }}</span>
              </div>
              <div class="flex items-center space-x-1 md:space-x-2">
                <span class="hidden sm:inline" style="color: var(--text-tertiary)">MEM</span>
                <span class="stat-badge">{{ formatBytes(stats.memory.used) }}</span>
                <span class="hidden md:inline" style="color: var(--text-tertiary)">/ {{ formatBytes(stats.memory.total) }}</span>
                <span class="text-xs" style="color: var(--text-tertiary)">({{ stats.memory.percent }}%)</span>
              </div>
              <div class="hidden sm:flex items-center space-x-2">
                <span style="color: var(--text-tertiary)">UP</span>
                <span class="stat-badge">{{ formatUptime(stats.uptime) }}</span>
              </div>
            </div>
            <div class="text-xs" style="color: var(--text-tertiary)">
              {{ currentTime }}
            </div>
          </div>
        </div>
      </div>
    </header>

    <div class="flex h-[calc(100vh-3rem)]">
      <!-- Mobile: Show backdrop when sidebar is open -->
      <div 
        v-if="isMobile && !sidebarCollapsed" 
        class="fixed top-12 left-0 right-0 bottom-0 bg-black bg-opacity-50 z-40"
        @click="sidebarCollapsed = true"
      ></div>
      
      <SessionList 
        v-show="!isMobile || !sidebarCollapsed"
        :sessions="sessions" 
        :currentSession="currentSession"
        :isCollapsed="sidebarCollapsed && !isMobile"
        :isMobile="isMobile"
        @select="selectSession"
        @refresh="refetch"
        @create="handleCreateSession"
        @kill="handleKillSession"
        @rename="handleRenameSession"
        @select-window="handleSelectWindow"
        @toggle-sidebar="toggleSidebar"
        :class="isMobile ? 'fixed left-0 top-12 bottom-0 z-50 w-64' : ''"
      />
      
      <main class="flex-1 min-w-0 overflow-hidden" style="background: var(--bg-primary)">
        <TerminalView 
          v-if="currentSession"
          :session="currentSession"
          :ws="ws"
          class="h-full"
        />
        <div v-else class="flex items-center justify-center h-full">
          <div class="text-center p-4">
            <p class="text-sm mb-2" style="color: var(--text-secondary)">No active session</p>
            <p class="text-xs mb-4" style="color: var(--text-tertiary)">Select or create a tmux session</p>
            <button
              v-if="isMobile"
              @click="sidebarCollapsed = false"
              class="px-4 py-2 text-sm border rounded"
              style="background: var(--bg-secondary); border-color: var(--border-primary); color: var(--text-primary)"
            >
              Show Sessions
            </button>
          </div>
        </div>
      </main>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { useWebSocket } from './composables/useWebSocket'
import { tmuxApi } from './api/tmux'
import SessionList from './components/SessionList.vue'
import TerminalView from './components/TerminalView.vue'

const queryClient = useQueryClient()
const currentSession = ref(null)
const sidebarCollapsed = ref(false)
const windowWidth = ref(window.innerWidth)
const ws = useWebSocket()
const currentTime = ref('')
const stats = ref({
  cpu: { loadAvg: [0, 0, 0] },
  memory: { total: 0, used: 0, percent: '0' },
  uptime: 0,
  hostname: '',
  platform: '',
  arch: ''
})

// Mobile detection
const isMobile = computed(() => windowWidth.value < 768) // md breakpoint

// Fetch system stats
const fetchStats = async () => {
  try {
    const response = await fetch('/api/stats')
    stats.value = await response.json()
  } catch (error) {
    console.error('Failed to fetch stats:', error)
  }
}

// Update clock and stats
let updateInterval
onMounted(() => {
  // Initialize sidebar state for mobile
  sidebarCollapsed.value = isMobile.value
  
  fetchStats()
  updateInterval = setInterval(() => {
    currentTime.value = new Date().toLocaleTimeString('en-US', { 
      hour12: false,
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit'
    })
    fetchStats()
  }, 1000)
  
  // Handle window resize for mobile detection
  const handleResize = () => {
    windowWidth.value = window.innerWidth
  }
  window.addEventListener('resize', handleResize)
})

onUnmounted(() => {
  if (updateInterval) clearInterval(updateInterval)
})

// Format helpers
const formatBytes = (bytes) => {
  if (!bytes) return '0B'
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(1024))
  return `${(bytes / Math.pow(1024, i)).toFixed(1)}${units[i]}`
}

const formatUptime = (seconds) => {
  if (!seconds) return '0s'
  const days = Math.floor(seconds / 86400)
  const hours = Math.floor((seconds % 86400) / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)
  if (days > 0) return `${days}d ${hours}h`
  if (hours > 0) return `${hours}h ${minutes}m`
  return `${minutes}m`
}

const { data: sessions = [], refetch } = useQuery({
  queryKey: ['sessions'],
  queryFn: tmuxApi.getSessions,
  refetchInterval: 5000
})


const handleCreateSession = async (sessionName) => {
  try {
    const result = await tmuxApi.createSession(sessionName)
    queryClient.invalidateQueries(['sessions'])
    // Auto-select the new session immediately
    currentSession.value = result.sessionName
  } catch (error) {
    console.error('Failed to create session:', error)
    alert('Failed to create session. It may already exist.')
  }
}

const handleKillSession = async (sessionName) => {
  try {
    await tmuxApi.killSession(sessionName)
    if (currentSession.value === sessionName) {
      currentSession.value = null
    }
    queryClient.invalidateQueries(['sessions'])
  } catch (error) {
    console.error('Failed to kill session:', error)
  }
}

const handleRenameSession = async (sessionName, newName) => {
  try {
    await tmuxApi.renameSession(sessionName, newName)
    if (currentSession.value === sessionName) {
      currentSession.value = newName
    }
    queryClient.invalidateQueries(['sessions'])
  } catch (error) {
    console.error('Failed to rename session:', error)
    alert('Failed to rename session. The name may already be in use.')
  }
}

const handleSelectWindow = (sessionName, window) => {
  console.log('Selecting window:', window.index, 'in session:', sessionName)
  
  // If switching to a different session, select it first
  if (currentSession.value !== sessionName) {
    currentSession.value = sessionName
  }
  
  // Send the window selection command
  if (ws.isConnected.value) {
    ws.send({
      type: 'select-window',
      sessionName: sessionName,
      windowIndex: window.index
    })
  }
}

ws.onMessage('sessions-list', (data) => {
  queryClient.setQueryData(['sessions'], data.sessions)
})

ws.onMessage('window-selected', (data) => {
  if (data.success) {
    console.log('Window selected successfully:', data.windowIndex)
  } else {
    console.error('Failed to select window:', data.error)
  }
})

const toggleSidebar = () => {
  sidebarCollapsed.value = !sidebarCollapsed.value
}

// Auto-collapse sidebar on mobile when session is selected
const selectSession = (sessionName) => {
  currentSession.value = sessionName
  if (isMobile.value) {
    sidebarCollapsed.value = true
  }
}

</script>