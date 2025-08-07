<template>
  <div class="session-group">
    <div
      @click="handleSessionClick"
      class="session-item group"
      :class="{
        'session-active': isActive,
        'session-collapsed': isCollapsed
      }"
    >
      <!-- Collapsed state - icon only -->
      <div v-if="isCollapsed" class="flex items-center justify-center">
        <div class="collapsed-icon" :class="{ 'collapsed-active': isActive }">
          <svg v-if="showWindows" class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
            <path d="M2 6a2 2 0 012-2h5l2 2h5a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z" />
          </svg>
          <svg v-else class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
            <path d="M9 2a1 1 0 000 2h2a1 1 0 100-2H9z" />
            <path fill-rule="evenodd" d="M4 5a2 2 0 012-2 1 1 0 000 2H4v10a2 2 0 002 2h8a2 2 0 002-2V5h-2a1 1 0 100-2 2 2 0 012 2v10a4 4 0 01-4 4H6a4 4 0 01-4-4V5z" clip-rule="evenodd" />
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
          
          <!-- Folder icon -->
          <svg v-if="showWindows" class="folder-icon" fill="currentColor" viewBox="0 0 20 20">
            <path d="M2 6a2 2 0 012-2h5l2 2h5a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z" />
          </svg>
          <svg v-else class="folder-icon" fill="currentColor" viewBox="0 0 20 20">
            <path d="M9 2a1 1 0 000 2h2a1 1 0 100-2H9z" />
            <path fill-rule="evenodd" d="M4 5a2 2 0 012-2 1 1 0 000 2H4v10a2 2 0 002 2h8a2 2 0 002-2V5h-2a1 1 0 100-2 2 2 0 012 2v10a4 4 0 01-4 4H6a4 4 0 01-4-4V5z" clip-rule="evenodd" />
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
            <span class="window-count">{{ session.windows }}</span>
          </div>
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

// Helper function to get session initials
const handleCreateWindow = (): void => {
  // Call createWindow on the WindowList component if it's available
  if (windowList.value) {
    windowList.value.createWindow()
  }
}

const getSessionInitials = (name: string): string => {
  if (!name) return '?'
  const words = name.split(/[-_\s]+/).filter(w => w.length > 0)
  if (words.length === 1) {
    return words[0]?.charAt(0).toUpperCase() || '?'
  }
  return words.slice(0, 2).map(w => w.charAt(0).toUpperCase()).join('')
}
</script>

<style scoped>
/* Session item styles */
.session-item {
  @apply relative flex items-center px-1 py-1 mx-1 rounded cursor-pointer;
  @apply transition-all duration-150;
  min-height: 28px;
}

.session-item:hover {
  background: rgba(255, 255, 255, 0.04);
}

.session-item.session-active {
  background: rgba(88, 166, 255, 0.1);
}

.session-item.session-active::before {
  content: '';
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 2px;
  background: var(--accent-primary);
  border-radius: 1px;
}

/* Collapsed state */
.session-collapsed {
  @apply px-2 py-2;
}

.collapsed-icon {
  @apply relative;
  color: var(--text-secondary);
}

.collapsed-icon.collapsed-active {
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
  @apply flex items-center justify-between w-full gap-2;
}

.session-label {
  @apply flex items-center gap-1.5 flex-1 min-w-0;
}

/* Chevron */
.chevron {
  @apply w-3 h-3 flex-shrink-0 transition-transform duration-150;
  color: var(--text-tertiary);
  cursor: pointer;
  margin-left: 2px;
}

.chevron:hover {
  color: var(--text-secondary);
}

.chevron-expanded {
  transform: rotate(90deg);
}

/* Folder icon */
.folder-icon {
  @apply w-4 h-4 flex-shrink-0;
  color: var(--text-tertiary);
}

.session-active .folder-icon {
  color: var(--accent-primary);
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
  @apply flex items-center gap-1.5 ml-auto;
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

.window-count {
  @apply px-1.5 py-0.5 text-xs rounded;
  font-size: 11px;
  background: var(--bg-tertiary);
  color: var(--text-tertiary);
  line-height: 1;
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
</style>