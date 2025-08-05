<template>
  <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50" @click.self="close">
    <div 
      class="rounded-lg shadow-xl w-full mx-4 flex flex-col"
      :class="isFullscreen ? 'h-full max-w-none' : 'max-w-4xl max-h-[90vh]'"
      style="background: var(--bg-secondary); border: 1px solid var(--border-primary)"
    >
      <!-- Header -->
      <div class="p-4 border-b flex items-center justify-between" style="border-color: var(--border-primary)">
        <div class="flex items-center space-x-3">
          <h3 class="text-lg font-semibold" style="color: var(--text-primary)">
            {{ file.name }}
          </h3>
          <span v-if="!file.exists" class="text-xs px-2 py-1 rounded bg-yellow-500 bg-opacity-20 text-yellow-500">
            New File
          </span>
          <span v-if="file.exists && !file.writable" class="text-xs px-2 py-1 rounded bg-red-500 bg-opacity-20 text-red-500">
            Read Only
          </span>
        </div>
        
        <div class="flex items-center space-x-2">
          <!-- Vim mode toggle -->
          <button
            @click="toggleVimMode"
            class="px-3 py-1 text-xs rounded"
            :class="vimMode ? 'bg-green-600 text-white' : ''"
            :style="!vimMode ? 'background: var(--bg-tertiary); color: var(--text-secondary)' : ''"
          >
            Vim: {{ vimMode ? 'ON' : 'OFF' }}
          </button>
          
          <!-- Fullscreen toggle -->
          <button
            @click="isFullscreen = !isFullscreen"
            class="p-1 rounded hover-bg"
            style="color: var(--text-secondary)"
            title="Toggle fullscreen"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path 
                v-if="!isFullscreen"
                stroke-linecap="round" 
                stroke-linejoin="round" 
                stroke-width="2" 
                d="M4 8V4m0 0h4M4 4l5 5m11-1V4m0 0h-4m4 0l-5 5M4 16v4m0 0h4m-4 0l5-5m11 5l-5-5m5 5v-4m0 4h-4" 
              />
              <path 
                v-else
                stroke-linecap="round" 
                stroke-linejoin="round" 
                stroke-width="2" 
                d="M6 18L18 6M6 6l12 12" 
              />
            </svg>
          </button>
          
          <!-- Close button -->
          <button
            @click="close"
            class="p-1 rounded hover-bg"
            style="color: var(--text-secondary)"
            title="Close"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
      </div>

      <!-- Editor container -->
      <div class="flex-1 overflow-hidden">
        <div ref="editorContainer" class="h-full"></div>
      </div>

      <!-- Footer -->
      <div class="p-4 border-t flex items-center justify-between" style="border-color: var(--border-primary)">
        <div class="flex items-center space-x-3 text-xs" style="color: var(--text-tertiary)">
          <span>{{ lineCount }} lines</span>
          <span>•</span>
          <span>{{ characterCount }} characters</span>
          <span v-if="hasChanges" class="text-yellow-500">• Modified</span>
        </div>
        
        <div class="flex items-center space-x-2">
          <!-- History button -->
          <button
            @click="showHistory"
            class="px-3 py-1.5 text-xs border rounded"
            style="background: var(--bg-secondary); border-color: var(--border-primary); color: var(--text-secondary)"
          >
            History
          </button>
          
          <!-- Cancel button -->
          <button
            @click="close"
            class="px-3 py-1.5 text-xs border rounded"
            style="background: var(--bg-secondary); border-color: var(--border-primary); color: var(--text-secondary)"
          >
            Cancel
          </button>
          
          <!-- Save button -->
          <button
            @click="save"
            :disabled="!hasChanges || !file.writable"
            class="px-3 py-1.5 text-xs border rounded"
            :class="{ 'opacity-50 cursor-not-allowed': !hasChanges || !file.writable }"
            style="background: var(--bg-primary); border-color: var(--border-primary); color: var(--text-primary)"
          >
            Save
          </button>
        </div>
      </div>
    </div>

    <!-- History Modal -->
    <DotfileHistory
      v-if="showingHistory"
      :file="file"
      @restore="restoreVersion"
      @close="showingHistory = false"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, nextTick } from 'vue'
import { EditorView, basicSetup } from 'codemirror'
import { EditorState } from '@codemirror/state'
import { vim } from '@replit/codemirror-vim'
import { oneDark } from '@codemirror/theme-one-dark'
import { javascript } from '@codemirror/lang-javascript'
import { python } from '@codemirror/lang-python'
import { StreamLanguage } from '@codemirror/language'
import { shell } from '@codemirror/legacy-modes/mode/shell'
import { yaml } from '@codemirror/legacy-modes/mode/yaml'
import { toml } from '@codemirror/legacy-modes/mode/toml'
import DotfileHistory from './DotfileHistory.vue'
import type { DotFile } from '@/types'

interface Props {
  file: DotFile
  content: string
}

const props = defineProps<Props>()

const emit = defineEmits<{
  save: [content: string]
  close: []
}>()

// State
const editorContainer = ref<HTMLElement>()
const hasChanges = ref(false)
const isFullscreen = ref(false)
const vimMode = ref(false)
const showingHistory = ref(false)
let editor: EditorView | null = null
let originalContent = ''

// Computed
const lineCount = computed(() => {
  if (!editor) return 0
  return editor.state.doc.lines
})

const characterCount = computed(() => {
  if (!editor) return 0
  return editor.state.doc.length
})

// Get language extension based on file type
const getLanguageExtension = () => {
  const fileName = props.file.name.toLowerCase()
  
  if (fileName.endsWith('.js') || fileName.endsWith('.jsx')) {
    return javascript({ jsx: true })
  }
  if (fileName.endsWith('.ts') || fileName.endsWith('.tsx')) {
    return javascript({ jsx: true, typescript: true })
  }
  if (fileName.endsWith('.py')) {
    return python()
  }
  if (fileName.endsWith('.sh') || fileName.endsWith('.bash') || 
      fileName.endsWith('.bashrc') || fileName.endsWith('.zshrc') ||
      fileName.endsWith('.profile')) {
    return StreamLanguage.define(shell)
  }
  if (fileName.endsWith('.yml') || fileName.endsWith('.yaml')) {
    return StreamLanguage.define(yaml)
  }
  if (fileName.endsWith('.toml') || fileName.endsWith('.conf') || 
      fileName.endsWith('.gitconfig') || fileName.endsWith('.tmux.conf')) {
    return StreamLanguage.define(toml)
  }
  
  // Default to shell for most dotfiles
  return StreamLanguage.define(shell)
}

// Initialize editor
const initEditor = async () => {
  if (!editorContainer.value) return
  
  console.log('Initializing editor with content length:', props.content?.length || 0)
  console.log('Content preview:', props.content?.substring(0, 100) || 'NO CONTENT')
  
  originalContent = props.content
  
  const extensions = [
    basicSetup,
    oneDark,
    getLanguageExtension(),
    EditorView.updateListener.of((update) => {
      if (update.docChanged) {
        hasChanges.value = editor?.state.doc.toString() !== originalContent
      }
    })
  ]
  
  // Add vim mode if enabled
  if (vimMode.value) {
    extensions.push(vim())
  }
  
  const state = EditorState.create({
    doc: props.content,
    extensions
  })
  
  editor = new EditorView({
    state,
    parent: editorContainer.value
  })
  
  // Focus editor
  editor.focus()
}

// Toggle vim mode
const toggleVimMode = async () => {
  vimMode.value = !vimMode.value
  
  if (editor) {
    editor.destroy()
    await nextTick()
    initEditor()
  }
}

// Save file
const save = () => {
  if (!editor || !hasChanges.value || !props.file.writable) return
  
  const content = editor.state.doc.toString()
  emit('save', content)
}

// Close editor
const close = () => {
  if (hasChanges.value) {
    if (!confirm('You have unsaved changes. Are you sure you want to close?')) {
      return
    }
  }
  emit('close')
}

// Show history
const showHistory = () => {
  showingHistory.value = true
}

// Restore version from history
const restoreVersion = (newContent: string) => {
  if (editor) {
    editor.dispatch({
      changes: {
        from: 0,
        to: editor.state.doc.length,
        insert: newContent
      }
    })
  }
  showingHistory.value = false
}

// Keyboard shortcuts
const handleKeydown = (e: KeyboardEvent) => {
  // Cmd/Ctrl + S to save
  if ((e.metaKey || e.ctrlKey) && e.key === 's') {
    e.preventDefault()
    save()
  }
  
  // Escape to close (when not in vim mode)
  if (e.key === 'Escape' && !vimMode.value) {
    close()
  }
}

// Watch for content changes
import { watch } from 'vue'

watch(() => props.content, (newContent) => {
  if (!editor && newContent) {
    // Initialize editor if not already initialized
    initEditor()
  } else if (editor && newContent !== editor.state.doc.toString()) {
    // Update editor content
    editor.dispatch({
      changes: {
        from: 0,
        to: editor.state.doc.length,
        insert: newContent
      }
    })
    originalContent = newContent
    hasChanges.value = false
  }
})

onMounted(() => {
  // Wait for content before initializing editor
  if (props.content) {
    initEditor()
  }
  document.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  editor?.destroy()
  document.removeEventListener('keydown', handleKeydown)
})
</script>

<style scoped>
:deep(.cm-editor) {
  height: 100%;
  font-size: 14px;
}

:deep(.cm-focused) {
  outline: none;
}

.hover-bg:hover {
  filter: brightness(1.2);
}
</style>