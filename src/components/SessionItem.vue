<template>
  <div class="session-wrapper">
    <div
      @click="handleSessionClick"
      class="session-item"
      :class="{ 'active': isActive, 'collapsed': isCollapsed }"
    >
      <!-- Collapsed state -->
      <template v-if="isCollapsed">
        <span class="collapsed-text">{{ session.name.charAt(0).toUpperCase() }}</span>
      </template>

      <!-- Expanded state -->
      <template v-else>
        <button
          @click.stop="toggleExpanded"
          class="expand-toggle"
          :class="{ 'expanded': showWindows }"
        >
          <svg width="8" height="8" viewBox="0 0 8 8" fill="currentColor">
            <path d="M1 2.5L4 5.5L7 2.5" stroke-width="1.5" stroke-linejoin="round" stroke-linecap="round"/>
          </svg>
        </button>
        
        <span v-if="!isEditing" class="session-name">{{ session.name }} ({{ session.windows }})</span>
        <input
          v-else
          v-model="editName"
          @keyup.enter="confirmRename"
          @keyup.escape="cancelEdit"
          @blur="confirmRename"
          ref="editInput"
          class="name-input"
          @click.stop
        />
        
        <div class="session-actions">
          <button
            v-if="showWindows"
            @click.stop="handleCreateWindow"
            class="action-btn"
            title="New window"
          >
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M12 5v14m7-7H5"/>
            </svg>
          </button>
          <button
            @click.stop="startEdit"
            class="action-btn"
            title="Rename"
          >
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M11 4H4a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2v-7M18.5 2.5a2.121 2.121 0 013 3L12 15l-4 1 1-4 9.5-9.5z"/>
            </svg>
          </button>
          <button
            @click.stop="$emit('kill')"
            class="action-btn"
            title="Kill"
          >
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M18 6L6 18M6 6l12 12"/>
            </svg>
          </button>
        </div>
      </template>
    </div>
    
    <div v-if="showWindows && !isCollapsed" class="windows">
      <WindowList
        :session-name="session.name"
        :is-active-session="isActive"
        @select-window="(window) => $emit('select-window', window)"
        ref="windowList"
      />
    </div>
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
  emit('select')
  showWindows.value = true
  if (windowList.value) {
    nextTick(() => windowList.value!.refresh())
  }
}

watch(() => props.isActive, (newVal) => {
  if (newVal && !showWindows.value && !props.isCollapsed) {
    showWindows.value = true
    if (windowList.value) {
      nextTick(() => windowList.value!.refresh())
    }
  }
})

const handleCreateWindow = (): void => {
  if (windowList.value) {
    windowList.value.createWindow()
  }
}
</script>

<style scoped>
.session-wrapper {
  margin-bottom: 2px;
}

.session-item {
  display: flex;
  align-items: center;
  height: 28px;
  padding: 0 12px;
  cursor: pointer;
  position: relative;
  color: var(--text-secondary);
  font-size: 13px;
  transition: background 100ms ease;
}

.session-item:hover {
  background: rgba(255, 255, 255, 0.04);
}

.session-item.active {
  background: rgba(88, 166, 255, 0.08);
  color: var(--text-primary);
}

.session-item.active::before {
  content: '';
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 2px;
  background: var(--accent-primary);
}

/* Collapsed */
.collapsed-text {
  font-weight: 500;
  font-size: 11px;
}

/* Expand toggle */
.expand-toggle {
  width: 16px;
  height: 16px;
  margin-right: 4px;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: none;
  color: inherit;
  opacity: 0.5;
  transition: transform 100ms ease;
}

.expand-toggle:hover {
  opacity: 0.8;
}

.expand-toggle.expanded {
  transform: rotate(0deg);
}

.expand-toggle:not(.expanded) {
  transform: rotate(-90deg);
}

/* Session name */
.session-name {
  flex: 1;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.name-input {
  flex: 1;
  padding: 2px 4px;
  font-size: 13px;
  font-weight: 500;
  background: var(--bg-primary);
  border: 1px solid var(--accent-primary);
  color: var(--text-primary);
  outline: none;
  border-radius: 2px;
}


/* Actions */
.session-actions {
  display: flex;
  gap: 4px;
  margin-left: 8px;
  opacity: 0;
  transition: opacity 100ms ease;
}

.session-item:hover .session-actions {
  opacity: 1;
}

.action-btn {
  width: 20px;
  height: 20px;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: none;
  color: inherit;
  opacity: 0.6;
  border-radius: 3px;
  transition: all 100ms ease;
}

.action-btn:hover {
  opacity: 1;
  background: rgba(255, 255, 255, 0.08);
}

/* Windows */
.windows {
  margin-left: 16px;
  border-left: 1px solid rgba(255, 255, 255, 0.06);
}
</style>