<template>
  <div class="group">
    <div
      @click="handleSessionClick"
      class="cursor-pointer transition-colors hover-bg"
      :class="[
        { 'bg-opacity-50': isActive },
        isCollapsed ? 'px-2 py-2' : 'px-3 py-2'
      ]"
      :style="{
        background: isActive ? 'var(--bg-tertiary)' : 'transparent',
        borderLeft: isActive ? '2px solid var(--accent-primary)' : '2px solid transparent'
      }"
      :title="isCollapsed ? `${session.name} (${session.windows}w)` : ''"
    >
      <!-- Collapsed state - show only initials -->
      <div v-if="isCollapsed" class="flex items-center justify-center">
        <div 
          class="w-8 h-8 rounded-full flex items-center justify-center text-xs font-bold"
          :style="{
            background: isActive ? 'var(--accent-primary)' : 'var(--bg-tertiary)',
            color: isActive ? 'white' : 'var(--text-primary)'
          }"
        >
          {{ getSessionInitials(session.name) }}
        </div>
      </div>

      <!-- Expanded state - show full info -->
      <div v-else class="flex items-center justify-between">
        <div class="flex-1 min-w-0">
          <div class="flex items-center space-x-2">
            <button
              @click.stop="toggleExpanded"
              class="p-0.5 hover-bg rounded transition-all duration-200"
              :style="{ 
                transform: showWindows ? 'rotate(90deg)' : 'rotate(0deg)',
                color: showWindows ? 'var(--text-secondary)' : 'var(--text-tertiary)'
              }"
              title="Toggle windows"
            >
              <svg class="w-3 h-3" fill="currentColor" viewBox="0 0 20 20">
                <path fill-rule="evenodd" d="M7.293 14.707a1 1 0 010-1.414L10.586 10 7.293 6.707a1 1 0 011.414-1.414l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0z" clip-rule="evenodd" />
              </svg>
            </button>
            <div v-if="!isEditing" class="text-xs font-medium truncate" :style="{ color: isActive ? 'var(--text-primary)' : 'var(--text-secondary)' }">
              {{ session.name }}
            </div>
            <input
              v-else
              v-model="editName"
              @keyup.enter="confirmRename"
              @keyup.escape="cancelEdit"
              @blur="confirmRename"
              ref="editInput"
              class="px-1 py-0.5 text-xs w-full focus:outline-none border"
              style="background: var(--bg-primary); border-color: var(--border-primary); color: var(--text-primary)"
            />
            <div v-if="session.attached" class="w-1.5 h-1.5 rounded-full" style="background: var(--accent-warning)"></div>
          </div>
          
          <div class="flex items-center space-x-3 text-xs mt-0.5" style="color: var(--text-tertiary)">
            <span>{{ session.windows }}w</span>
            <span v-if="session.dimensions">{{ session.dimensions }}</span>
          </div>
        </div>
        
        <div class="flex items-center space-x-0.5" @click.stop>
          <button
            @click="startEdit"
            class="p-1 hover-bg rounded"
            style="color: var(--text-tertiary)"
            title="Rename"
          >
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
            </svg>
          </button>
          <button
            @click.stop="$emit('kill')"
            class="p-1 hover-bg rounded"
            style="color: var(--text-tertiary)"
            title="Kill"
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
const getSessionInitials = (name: string): string => {
  if (!name) return '?'
  const words = name.split(/[-_\s]+/).filter(w => w.length > 0)
  if (words.length === 1) {
    return words[0]?.charAt(0).toUpperCase() || '?'
  }
  return words.slice(0, 2).map(w => w.charAt(0).toUpperCase()).join('')
}
</script>