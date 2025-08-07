<template>
  <div class="session-group">
    <div
      @click="handleSessionClick"
      class="session-item"
      :class="{
        'session-active': isActive,
        'session-collapsed': isCollapsed
      }"
    >
      <!-- Collapsed state - icon only -->
      <div v-if="isCollapsed" class="flex items-center justify-center">
        <div class="collapsed-icon" :class="{ 'collapsed-active': isActive }">
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
          </svg>
          <div v-if="isActive" class="active-dot"></div>
        </div>
      </div>

      <!-- Expanded state -->
      <div v-else class="session-content">
        <!-- Left side: chevron, icon, name -->
        <div class="session-label">
          <!-- Chevron -->
          <svg 
            @click.stop="toggleExpanded"
            class="chevron" 
            :class="{ 'chevron-expanded': showWindows }"
            fill="currentColor" 
            viewBox="0 0 20 20"
          >
            <path fill-rule="evenodd" d="M7.293 14.707a1 1 0 010-1.414L10.586 10 7.293 6.707a1 1 0 011.414-1.414l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0z" clip-rule="evenodd" />
          </svg>
          
          <!-- Terminal icon -->
          <svg class="terminal-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
          </svg>
          
          <!-- Session name -->
          <span v-if="!isEditing" class="session-name">
            {{ session.name }}
          </span>
          <input
            v-else
            v-model="editName"
            @keyup.enter="confirmRename"
            @keyup.escape="cancelEdit"
            @blur="confirmRename"
            ref="editInput"
            class="session-name-input"
          />
          
          <!-- Indicators -->
          <div class="indicators">
            <div v-if="isActive" class="indicator-dot active-indicator" title="Active session"></div>
            <div v-else-if="session.attached" class="indicator-dot attached-indicator" title="Session is attached"></div>
          </div>
        </div>
        
        <!-- Window count badge -->
        <div class="window-count-badge" :title="`${session.windows} ${session.windows === 1 ? 'window' : 'windows'}`">
          <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M4 6a2 2 0 012-2h12a2 2 0 012 2v12a2 2 0 01-2 2H6a2 2 0 01-2-2V6z" />
          </svg>
          <span>{{ session.windows }}</span>
        </div>
        
        <!-- Right side: actions (show on hover) -->
        <div class="session-actions">
          <button
            v-if="showWindows"
            @click.stop="handleCreateWindow"
            class="action-btn"
            title="New window"
          >
            <svg class="w-3.5 h-3.5" fill="currentColor" viewBox="0 0 20 20">
              <path fill-rule="evenodd" d="M10 3a1 1 0 011 1v5h5a1 1 0 110 2h-5v5a1 1 0 11-2 0v-5H4a1 1 0 110-2h5V4a1 1 0 011-1z" clip-rule="evenodd" />
            </svg>
          </button>
          <button
            @click.stop="startEdit"
            class="action-btn"
            title="Rename session"
          >
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
            </svg>
          </button>
          <button
            @click.stop="$emit('kill')"
            class="action-btn"
            title="Kill session"
          >
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
      </div>
    </div>
    
    <WindowList
      v-if="showWindows && !isCollapsed"
      :session-name="session.name"
      :is-active-session="isActive"
      @select-window="(window) => $emit('select-window', window)"
      ref="windowList"
    />
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

// Removed unused formatDate function

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
  // Call createWindow on the WindowList component if it's available
  if (windowList.value) {
    windowList.value.createWindow()
  }
}
</script>

<style scoped>
/* Session item styles */
.session-item {
  @apply relative flex items-center px-2 py-1 cursor-pointer;
  @apply transition-all duration-150;
  min-height: 28px;
  margin: 0 4px;
  border-radius: 4px;
}

.session-item:hover:not(.session-active) {
  background: rgba(255, 255, 255, 0.04);
}

/* Ensure icons don't get affected by hover background */
.session-item:hover svg {
  background: transparent;
}

/* Active session - full width highlight */
.session-item.session-active {
  background: rgba(139, 148, 158, 0.1);
  margin: 0;
  border-radius: 0;
  padding-left: 12px;
  border: 1px solid rgba(139, 148, 158, 0.2);
  border-left: none;
  border-right: none;
}

.session-item.session-active::before {
  content: '';
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 3px;
  background: var(--accent-success);
}

/* Collapsed state */
.session-collapsed {
  @apply px-2 py-2;
}

/* Collapsed active state adjustments */
.session-collapsed.session-active {
  @apply px-2; /* Keep consistent padding in collapsed state */
}

.collapsed-icon {
  @apply relative;
}

.collapsed-icon svg {
  color: var(--text-secondary);
}

.collapsed-icon.collapsed-active svg {
  color: var(--accent-primary);
}

.collapsed-icon .active-dot {
  position: absolute;
  bottom: -2px;
  right: -2px;
  width: 6px;
  height: 6px;
  background: var(--accent-success);
  border-radius: 50%;
  border: 1.5px solid var(--bg-secondary);
}

/* Session content */
.session-content {
  @apply flex items-center w-full gap-2;
}

.session-label {
  @apply flex items-center gap-1.5 min-w-0;
  flex: 1;
}

/* Adjust label spacing for active sessions */
.session-active .session-label {
  margin-left: 2px;
}

/* Chevron */
.chevron {
  @apply w-3 h-3 flex-shrink-0 transition-transform duration-150;
  color: var(--text-tertiary);
  cursor: pointer;
  margin-left: -2px;
  margin-right: 2px;
}

.chevron:hover {
  color: var(--text-secondary);
}

.chevron-expanded {
  transform: rotate(90deg);
}

/* Terminal icon */
.terminal-icon {
  @apply w-4 h-4 flex-shrink-0;
  stroke: var(--text-tertiary);
}

.session-active .terminal-icon {
  stroke: var(--text-primary);
}

/* Session name */
.session-name {
  @apply text-xs font-medium truncate;
  color: var(--text-primary);
  font-size: 13px;
}

.session-name-input {
  @apply px-1 py-0 text-xs w-full;
  background: var(--bg-primary);
  border: 1px solid var(--accent-primary);
  color: var(--text-primary);
  outline: none;
  border-radius: 2px;
  font-size: 13px;
}

/* Indicators */
.indicators {
  @apply flex items-center gap-1.5;
}

.indicator-dot {
  @apply w-1.5 h-1.5 rounded-full flex-shrink-0;
}

.active-indicator {
  background: var(--accent-success);
}

.attached-indicator {
  background: var(--accent-warning);
}

/* Window count badge */
.window-count-badge {
  @apply flex items-center gap-1 px-2 py-0.5 rounded-full;
  background: var(--bg-tertiary);
  color: var(--text-secondary);
  font-size: 11px;
  margin-left: 8px;
  opacity: 0.8;
  transition: opacity 150ms ease;
}

.window-count-badge:hover {
  opacity: 1;
}

.session-active .window-count-badge {
  opacity: 1;
  background: rgba(88, 166, 255, 0.1);
  color: var(--accent-primary);
}

/* Session actions */
.session-actions {
  @apply flex items-center gap-0.5 opacity-0;
  transition: opacity 150ms ease;
}

.session-item:hover .session-actions {
  opacity: 1;
}

.action-btn {
  @apply p-0.5 rounded;
  color: var(--text-tertiary);
  transition: all 150ms ease;
}

.action-btn:hover {
  background: rgba(255, 255, 255, 0.08);
  color: var(--text-secondary);
}

/* Session group */
.session-group {
  @apply relative;
}

/* Add some spacing between session groups */
.session-group + .session-group {
  margin-top: 2px;
}
</style>