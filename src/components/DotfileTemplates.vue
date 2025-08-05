<template>
  <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50" @click.self="close">
    <div 
      class="p-6 rounded-lg shadow-xl max-w-3xl w-full mx-4 max-h-[80vh] flex flex-col" 
      style="background: var(--bg-secondary); border: 1px solid var(--border-primary)"
    >
      <h3 class="text-lg font-semibold mb-4" style="color: var(--text-primary)">
        Configuration Templates
      </h3>

      <!-- Loading state -->
      <div v-if="isLoading" class="py-8 text-center">
        <div class="animate-pulse text-sm" style="color: var(--text-tertiary)">
          Loading templates...
        </div>
      </div>

      <!-- Templates grid -->
      <div v-else-if="templates.length > 0" class="flex-1 overflow-y-auto">
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div
            v-for="template in templates"
            :key="template.name"
            class="p-4 rounded border cursor-pointer hover-bg"
            style="background: var(--bg-tertiary); border-color: var(--border-primary)"
            @click="selectTemplate(template)"
          >
            <!-- Template header -->
            <div class="flex items-start justify-between mb-2">
              <div class="flex items-center space-x-2">
                <!-- Icon based on file type -->
                <svg class="w-5 h-5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path 
                    v-if="template.fileType === 'Shell'"
                    stroke-linecap="round" 
                    stroke-linejoin="round" 
                    stroke-width="2" 
                    d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" 
                  />
                  <path 
                    v-else-if="template.fileType === 'Git'"
                    stroke-linecap="round" 
                    stroke-linejoin="round" 
                    stroke-width="2" 
                    d="M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4" 
                  />
                  <path 
                    v-else-if="template.fileType === 'Vim'"
                    stroke-linecap="round" 
                    stroke-linejoin="round" 
                    stroke-width="2" 
                    d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" 
                  />
                  <path 
                    v-else
                    stroke-linecap="round" 
                    stroke-linejoin="round" 
                    stroke-width="2" 
                    d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" 
                  />
                </svg>
                
                <h4 class="font-medium" style="color: var(--text-primary)">
                  {{ template.name }}
                </h4>
              </div>
              
              <span class="text-xs px-2 py-1 rounded" style="background: var(--bg-secondary); color: var(--text-tertiary)">
                {{ template.fileType }}
              </span>
            </div>
            
            <!-- Description -->
            <p class="text-sm mb-3" style="color: var(--text-secondary)">
              {{ template.description }}
            </p>
            
            <!-- Preview button -->
            <button
              @click.stop="previewTemplate(template)"
              class="text-xs"
              style="color: var(--text-primary)"
            >
              Preview →
            </button>
          </div>
        </div>
      </div>

      <!-- Empty state -->
      <div v-else class="py-8 text-center">
        <p class="text-sm" style="color: var(--text-tertiary)">
          No templates available
        </p>
      </div>

      <!-- Close button -->
      <div class="mt-6 flex justify-end">
        <button
          @click="close"
          class="px-4 py-2 text-sm border rounded"
          style="background: var(--bg-secondary); border-color: var(--border-primary); color: var(--text-secondary)"
        >
          Close
        </button>
      </div>
    </div>

    <!-- Preview Modal -->
    <div v-if="previewingTemplate" class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-[60]" @click.self="closePreview">
      <div 
        class="p-6 rounded-lg shadow-xl max-w-4xl w-full mx-4 max-h-[80vh] flex flex-col" 
        style="background: var(--bg-secondary); border: 1px solid var(--border-primary)"
      >
        <h4 class="text-md font-semibold mb-3" style="color: var(--text-primary)">
          Template Preview - {{ previewingTemplate.name }}
        </h4>
        
        <div class="flex-1 overflow-auto p-4 rounded font-mono text-sm" style="background: var(--bg-primary)">
          <pre style="color: var(--text-primary)">{{ previewingTemplate.content }}</pre>
        </div>
        
        <div class="mt-4 flex justify-between">
          <div class="text-xs" style="color: var(--text-tertiary)">
            Target file: {{ getTargetPath(previewingTemplate) }}
          </div>
          
          <div class="flex space-x-2">
            <button
              @click="closePreview"
              class="px-4 py-2 text-sm border rounded"
              style="background: var(--bg-secondary); border-color: var(--border-primary); color: var(--text-secondary)"
            >
              Close
            </button>
            <button
              @click="applyTemplate(previewingTemplate)"
              class="px-4 py-2 text-sm border rounded"
              style="background: var(--bg-primary); border-color: var(--border-primary); color: var(--text-primary)"
            >
              Apply Template
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useWebSocket } from '@/composables/useWebSocket'
import type { DotFileTemplate, ServerMessage } from '@/types'

const emit = defineEmits<{
  select: [template: { path: string; content: string }]
  close: []
}>()

const ws = useWebSocket()

// State
const isLoading = ref(true)
const templates = ref<DotFileTemplate[]>([])
const previewingTemplate = ref<DotFileTemplate | null>(null)

// Load templates
const loadTemplates = () => {
  isLoading.value = true
  ws.send({ type: 'get-dotfile-templates' })
}

// Get target path for template
const getTargetPath = (template: DotFileTemplate): string => {
  const fileMap: Record<string, string> = {
    'Basic .bashrc': '~/.bashrc',
    'Basic .vimrc': '~/.vimrc',
    'Basic .gitconfig': '~/.gitconfig',
    'Basic .tmux.conf': '~/.tmux.conf',
  }
  
  return fileMap[template.name] || '~/.' + template.name.toLowerCase().replace(/^basic \./, '')
}

// Select template
const selectTemplate = (template: DotFileTemplate) => {
  const path = getTargetPath(template)
  emit('select', { path, content: template.content })
}

// Preview template
const previewTemplate = (template: DotFileTemplate) => {
  previewingTemplate.value = template
}

// Close preview
const closePreview = () => {
  previewingTemplate.value = null
}

// Apply template
const applyTemplate = (template: DotFileTemplate) => {
  selectTemplate(template)
  closePreview()
}

// Close modal
const close = () => {
  emit('close')
}

// WebSocket handler
const handleDotfileTemplates = (msg: Extract<ServerMessage, { type: 'dotfile-templates' }>) => {
  templates.value = msg.templates
  isLoading.value = false
}

// Setup
onMounted(() => {
  loadTemplates()
  const unsub = ws.onMessage('dotfile-templates', handleDotfileTemplates)
  
  // Cleanup
  return () => {
    unsub()
  }
})
</script>

<style scoped>
.hover-bg:hover {
  filter: brightness(1.1);
}

pre {
  white-space: pre-wrap;
  word-wrap: break-word;
}
</style>