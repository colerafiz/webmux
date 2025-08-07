<template>
  <div class="session-card">
    <div
      @click="handleSessionClick"
      class="session-header"
      :class="{
        'session-active': isActive,
        'session-collapsed': isCollapsed
      }"
    >
      <!-- Collapsed state -->
      <div v-if="isCollapsed" class="collapsed-content">
        <div class="session-avatar" :class="{ active: isActive }">
          {{ session.name.substring(0, 2).toUpperCase() }}
        </div>
      </div>

      <!-- Expanded state -->
      <div v-else class="session-content">
        <div class="session-main">
          <!-- Status indicator -->
          <div class="status-indicator" :class="{ active: isActive }"></div>
          
          <!-- Session Icon with gradient background -->
          <div class="session-icon-wrapper">
            <svg class="session-icon" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" d="M6.75 7.5l3 2.25-3 2.25m4.5 0h3m-9 8.25h13.5A2.25 2.25 0 0021 18V6a2.25 2.25 0 00-2.25-2.25H5.25A2.25 2.25 0 003 6v12a2.25 2.25 0 002.25 2.25z" />
            </svg>
          </div>
          
          <!-- Session Info -->
          <div class="session-info">
            <div class="session-name-row">
              <h3 v-if="!isEditing" class="session-name">
                {{ session.name }}
              </h3>
              <input
                v-else
                v-model="editName"
                @keyup.enter="confirmRename"
                @keyup.escape="cancelEdit"
                @blur="confirmRename"
                ref="editInput"
                class="session-name-input"
              />
              <span v-if="session.attached" class="attached-pill">
                <span class="attached-dot"></span>
                Attached
              </span>
            </div>
            <div class="session-stats">
              <div class="stat-item">
                <svg class="stat-icon" fill="none" viewBox="0 0 20 20" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 7h.01M7 3h5a4 4 0 014 4v5a4 4 0 01-4 4H7a4 4 0 01-4-4V7a4 4 0 014-4z" />
                </svg>
                <span>{{ session.windows }} {{ session.windows === 1 ? 'window' : 'windows' }}</span>
              </div>
              <div class="stat-item">
                <svg class="stat-icon" fill="none" viewBox="0 0 20 20" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                <span>Active</span>
              </div>
            </div>
          </div>
        </div>
        
        <!-- Actions -->
        <div class="session-actions">
          <button
            @click.stop="toggleExpanded"
            class="expand-btn"
            :class="{ expanded: showWindows }"
          >
            <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" d="M19.5 8.25l-7.5 7.5-7.5-7.5" />
            </svg>
          </button>
          <div class="action-group">
            <button
              v-if="showWindows"
              @click.stop="handleCreateWindow"
              class="action-btn primary"
              title="New window"
            >
              <svg class="w-4 h-4" fill="none" viewBox="0 0 20 20" stroke="currentColor" stroke-width="2">
                <path stroke-linecap="round" stroke-linejoin="round" d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
              </svg>
            </button>
            <button
              @click.stop="startEdit"
              class="action-btn"
              title="Rename"
            >
              <svg class="w-4 h-4" fill="none" viewBox="0 0 20 20" stroke="currentColor" stroke-width="2">
                <path stroke-linecap="round" stroke-linejoin="round" d="M13.586 3.586a2 2 0 112.828 2.828l-.793.793-2.828-2.828.793-.793zM11.379 5.793L3 14.172V17h2.828l8.38-8.379-2.83-2.828z" />
              </svg>
            </button>
            <button
              @click.stop="$emit('kill')"
              class="action-btn danger"
              title="Kill session"
            >
              <svg class="w-4 h-4" fill="none" viewBox="0 0 20 20" stroke="currentColor" stroke-width="2">
                <path stroke-linecap="round" stroke-linejoin="round" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" />
              </svg>
            </button>
          </div>
        </div>
      </div>
    </div>
    
    <!-- Windows List -->
    <transition
      name="windows-slide"
      @enter="onWindowsEnter"
      @leave="onWindowsLeave"
    >
      <WindowList
        v-if="showWindows && !isCollapsed"
        :session-name="session.name"
        :is-active-session="isActive"
        @select-window="(window) => $emit('select-window', window)"
        ref="windowList"
        class="windows-container"
      />
    </transition>
  </div>
</template>

<script setup lang="ts">
import { ref, nextTick, watch } from 'vue'
import WindowList from './WindowList.vue'
import type { TmuxSession, TmuxWindow } from '@/types'

interface Props {
  session: TmuxSession
  isActive: boolean
  isCollapsed: boolean
  isMobile: boolean
}

const props = withDefaults(defineProps<Props>(), {
  isActive: false,
  isCollapsed: false,
  isMobile: false
})

const emit = defineEmits<{
  select: []
  kill: []
  rename: [newName: string]
  'select-window': [window: TmuxWindow]
}>()

const isEditing = ref<boolean>(false)
const editName = ref<string>('')
const editInput = ref<HTMLInputElement | null>(null)
const showWindows = ref<boolean>(false)
const windowList = ref<InstanceType<typeof WindowList> | null>(null)

const startEdit = (): void => {
  isEditing.value = true
  editName.value = props.session.name
  nextTick(() => {
    editInput.value?.focus()
    editInput.value?.select()
  })
}

const confirmRename = (): void => {
  if (editName.value && editName.value !== props.session.name) {
    emit('rename', editName.value)
  }
  cancelEdit()
}

const cancelEdit = (): void => {
  isEditing.value = false
  editName.value = ''
}

const toggleExpanded = (): void => {
  showWindows.value = !showWindows.value
  if (showWindows.value && windowList.value) {
    nextTick(() => windowList.value!.refresh())
  }
}

const handleSessionClick = (): void => {
  if (isEditing.value) return
  
  // Emit select to mark this session as active
  emit('select')
  
  // Also expand the session to show windows
  showWindows.value = true
  if (windowList.value) {
    nextTick(() => windowList.value!.refresh())
  }
}

// Auto-expand when session becomes active (but not when collapsed)
watch(() => props.isActive, (newVal) => {
  if (newVal && !showWindows.value && !props.isCollapsed) {
    showWindows.value = true
    if (windowList.value) {
      nextTick(() => windowList.value!.refresh())
    }
  }
})

// Helper function to handle create window
const handleCreateWindow = (): void => {
  if (windowList.value) {
    windowList.value.createWindow()
  }
}

// Animation handlers
const onWindowsEnter = (el: Element): void => {
  const element = el as HTMLElement
  element.style.maxHeight = '0px'
  element.style.opacity = '0'
  
  requestAnimationFrame(() => {
    element.style.transition = 'all 300ms cubic-bezier(0.4, 0, 0.2, 1)'
    element.style.maxHeight = element.scrollHeight + 'px'
    element.style.opacity = '1'
  })
}

const onWindowsLeave = (el: Element): void => {
  const element = el as HTMLElement
  element.style.transition = 'all 200ms cubic-bezier(0.4, 0, 0.2, 1)'
  element.style.maxHeight = '0px'
  element.style.opacity = '0'
}
</script>

<style scoped>
/* Session card container */
.session-card {
  @apply mb-3;
}

/* Session header - modern card design */
.session-header {
  @apply relative mx-3 rounded-xl cursor-pointer overflow-hidden;
  @apply transition-all duration-300 ease-out;
  background: linear-gradient(135deg, rgba(30, 41, 59, 0.5) 0%, rgba(30, 41, 59, 0.3) 100%);
  backdrop-filter: blur(12px);
  border: 1px solid rgba(148, 163, 184, 0.1);
  box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06);
}

.session-header:hover {
  transform: translateY(-2px);
  box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -2px rgba(0, 0, 0, 0.05);
  border-color: rgba(148, 163, 184, 0.2);
}

.session-header.session-active {
  background: linear-gradient(135deg, rgba(59, 130, 246, 0.15) 0%, rgba(59, 130, 246, 0.05) 100%);
  border-color: rgba(59, 130, 246, 0.3);
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1), 0 10px 15px -3px rgba(59, 130, 246, 0.1);
}

/* Collapsed state */
.collapsed-content {
  @apply p-3 flex items-center justify-center;
}

.session-avatar {
  @apply w-10 h-10 rounded-lg flex items-center justify-center text-sm font-bold;
  background: linear-gradient(135deg, rgba(148, 163, 184, 0.2) 0%, rgba(148, 163, 184, 0.1) 100%);
  color: var(--text-secondary);
  transition: all 200ms ease;
}

.session-avatar.active {
  background: linear-gradient(135deg, rgba(59, 130, 246, 0.3) 0%, rgba(59, 130, 246, 0.1) 100%);
  color: #3b82f6;
}

/* Session content */
.session-content {
  @apply p-4 flex items-center justify-between gap-3;
}

.session-main {
  @apply flex items-center gap-3 flex-1 min-w-0;
}

/* Status indicator */
.status-indicator {
  @apply w-1 h-8 rounded-full bg-gray-600 transition-all duration-300;
}

.status-indicator.active {
  @apply bg-blue-500;
  box-shadow: 0 0 8px rgba(59, 130, 246, 0.5);
}

/* Session icon */
.session-icon-wrapper {
  @apply relative p-2.5 rounded-lg;
  background: linear-gradient(135deg, rgba(148, 163, 184, 0.1) 0%, rgba(148, 163, 184, 0.05) 100%);
}

.session-icon {
  @apply w-5 h-5;
  color: var(--text-secondary);
}

.session-active .session-icon-wrapper {
  background: linear-gradient(135deg, rgba(59, 130, 246, 0.2) 0%, rgba(59, 130, 246, 0.1) 100%);
}

.session-active .session-icon {
  color: #3b82f6;
}

/* Session info */
.session-info {
  @apply flex-1 min-w-0;
}

.session-name-row {
  @apply flex items-center gap-2 mb-1;
}

.session-name {
  @apply text-base font-semibold truncate;
  color: var(--text-primary);
  letter-spacing: -0.02em;
}

.session-name-input {
  @apply px-2 py-1 text-base font-semibold w-full rounded-md;
  background: rgba(0, 0, 0, 0.2);
  border: 2px solid rgba(59, 130, 246, 0.5);
  color: var(--text-primary);
  outline: none;
}

/* Attached pill */
.attached-pill {
  @apply flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium;
  background: rgba(34, 197, 94, 0.1);
  color: #22c55e;
}

.attached-dot {
  @apply w-1.5 h-1.5 rounded-full bg-green-500;
  animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

/* Session stats */
.session-stats {
  @apply flex items-center gap-3;
}

.stat-item {
  @apply flex items-center gap-1 text-xs;
  color: var(--text-tertiary);
}

.stat-icon {
  @apply w-3.5 h-3.5;
}

/* Session actions */
.session-actions {
  @apply flex items-center gap-2;
}

.expand-btn {
  @apply p-2 rounded-lg transition-all duration-200;
  background: rgba(148, 163, 184, 0.1);
  color: var(--text-secondary);
}

.expand-btn:hover {
  background: rgba(148, 163, 184, 0.2);
}

.expand-btn.expanded {
  transform: rotate(180deg);
}

.action-group {
  @apply flex items-center gap-1 opacity-0 transition-opacity duration-200;
}

.session-header:hover .action-group {
  opacity: 1;
}

.action-btn {
  @apply p-1.5 rounded-lg transition-all duration-200;
  background: rgba(148, 163, 184, 0.1);
  color: var(--text-secondary);
}

.action-btn:hover {
  background: rgba(148, 163, 184, 0.2);
  transform: scale(1.1);
}

.action-btn.primary {
  background: rgba(59, 130, 246, 0.1);
  color: #3b82f6;
}

.action-btn.primary:hover {
  background: rgba(59, 130, 246, 0.2);
}

.action-btn.danger:hover {
  background: rgba(239, 68, 68, 0.1);
  color: #ef4444;
}

/* Windows container */
.windows-container {
  overflow: hidden;
}
</style>