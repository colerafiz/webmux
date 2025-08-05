<template>
  <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50" @click.self="cancel">
    <div 
      class="p-6 rounded-lg shadow-xl max-w-2xl w-full mx-4 max-h-[90vh] overflow-y-auto" 
      style="background: var(--bg-secondary); border: 1px solid var(--border-primary)"
    >
      <h3 class="text-lg font-semibold mb-4" style="color: var(--text-primary)">
        {{ isEditing ? 'Edit Cron Job' : 'Create New Cron Job' }}
      </h3>
      
      <!-- Job Name -->
      <div class="mb-4">
        <label class="block text-sm font-medium mb-1" style="color: var(--text-secondary)">
          Job Name
        </label>
        <input 
          v-model="form.name"
          type="text" 
          placeholder="e.g., Backup Database"
          class="w-full px-3 py-2 border rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
          style="background: var(--bg-primary); border-color: var(--border-primary); color: var(--text-primary)"
          ref="nameInput"
        />
      </div>
      
      <!-- Schedule -->
      <div class="mb-4">
        <label class="block text-sm font-medium mb-1" style="color: var(--text-secondary)">
          Schedule
        </label>
        
        <!-- Quick presets -->
        <div class="flex flex-wrap gap-2 mb-2">
          <button
            v-for="preset in schedulePresets"
            :key="preset.value"
            @click="form.schedule = preset.value"
            class="px-2 py-1 text-xs rounded"
            :class="form.schedule === preset.value ? 'bg-blue-600 text-white' : ''"
            :style="form.schedule !== preset.value ? 'background: var(--bg-tertiary); color: var(--text-secondary)' : ''"
          >
            {{ preset.label }}
          </button>
        </div>
        
        <!-- Custom expression -->
        <input 
          v-model="form.schedule"
          type="text" 
          placeholder="* * * * * (minute hour day month weekday)"
          class="w-full px-3 py-2 border rounded focus:outline-none focus:ring-2 focus:ring-blue-500 font-mono"
          style="background: var(--bg-primary); border-color: var(--border-primary); color: var(--text-primary)"
        />
        
        <!-- Schedule preview -->
        <div class="mt-1 text-xs" style="color: var(--text-tertiary)">
          {{ scheduleDescription }}
        </div>
      </div>
      
      <!-- Command -->
      <div class="mb-4">
        <label class="block text-sm font-medium mb-1" style="color: var(--text-secondary)">
          Command
        </label>
        <textarea 
          v-model="form.command"
          placeholder="e.g., /home/user/scripts/backup.sh"
          rows="3"
          class="w-full px-3 py-2 border rounded focus:outline-none focus:ring-2 focus:ring-blue-500 font-mono text-sm"
          style="background: var(--bg-primary); border-color: var(--border-primary); color: var(--text-primary)"
        ></textarea>
        
        <!-- Test button -->
        <button
          @click="testCommand"
          :disabled="!form.command || isTesting"
          class="mt-2 px-3 py-1 text-xs rounded"
          :class="{ 'opacity-50 cursor-not-allowed': !form.command || isTesting }"
          style="background: var(--bg-tertiary); color: var(--text-primary)"
        >
          {{ isTesting ? 'Testing...' : 'Test Command' }}
        </button>
        
        <!-- Test output -->
        <div v-if="testOutput" class="mt-2 p-2 rounded text-xs font-mono" style="background: var(--bg-primary)">
          <pre style="color: var(--text-primary)">{{ testOutput }}</pre>
        </div>
      </div>
      
      <!-- Advanced Options (collapsible) -->
      <div class="mb-4">
        <button
          @click="showAdvanced = !showAdvanced"
          class="text-sm font-medium flex items-center space-x-1"
          style="color: var(--text-secondary)"
        >
          <svg 
            class="w-3 h-3 transition-transform" 
            :class="{ 'rotate-90': showAdvanced }"
            fill="none" 
            stroke="currentColor" 
            viewBox="0 0 24 24"
          >
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
          </svg>
          <span>Advanced Options</span>
        </button>
        
        <div v-show="showAdvanced" class="mt-3 space-y-3">
          <!-- Email output -->
          <div>
            <label class="block text-xs font-medium mb-1" style="color: var(--text-secondary)">
              Email Output To
            </label>
            <input 
              v-model="form.emailTo"
              type="email" 
              placeholder="user@example.com"
              class="w-full px-3 py-2 text-sm border rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
              style="background: var(--bg-primary); border-color: var(--border-primary); color: var(--text-primary)"
            />
          </div>
          
          <!-- Log output -->
          <div>
            <label class="flex items-center space-x-2">
              <input
                v-model="form.logOutput"
                type="checkbox"
                class="rounded"
              />
              <span class="text-xs" style="color: var(--text-secondary)">
                Log command output to file
              </span>
            </label>
          </div>
          
          <!-- TMUX session -->
          <div>
            <label class="block text-xs font-medium mb-1" style="color: var(--text-secondary)">
              Run in TMUX Session
            </label>
            <input 
              v-model="form.tmuxSession"
              type="text" 
              placeholder="session-name"
              class="w-full px-3 py-2 text-sm border rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
              style="background: var(--bg-primary); border-color: var(--border-primary); color: var(--text-primary)"
            />
          </div>
        </div>
      </div>
      
      <!-- Actions -->
      <div class="flex justify-end space-x-2">
        <button 
          @click="cancel"
          class="px-4 py-2 text-sm border rounded"
          style="background: var(--bg-secondary); border-color: var(--border-primary); color: var(--text-secondary)"
        >
          Cancel
        </button>
        <button 
          @click="save"
          :disabled="!isValid"
          class="px-4 py-2 text-sm border rounded"
          :class="{ 'opacity-50 cursor-not-allowed': !isValid }"
          style="background: var(--bg-primary); border-color: var(--border-primary); color: var(--text-primary)"
        >
          {{ isEditing ? 'Update' : 'Create' }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, nextTick } from 'vue'
import { useWebSocket } from '@/composables/useWebSocket'
import type { CronJob, TestCronCommandMessage, CronCommandOutputMessage } from '@/types'

interface Props {
  job?: CronJob | null
}

const props = defineProps<Props>()

const emit = defineEmits<{
  save: [job: CronJob]
  cancel: []
}>()

const ws = useWebSocket()

const nameInput = ref<HTMLInputElement>()
const showAdvanced = ref(false)
const isTesting = ref(false)
const testOutput = ref('')

const form = ref<Partial<CronJob>>({
  name: '',
  schedule: '0 * * * *',
  command: '',
  enabled: true,
  emailTo: '',
  logOutput: false,
  tmuxSession: ''
})

const schedulePresets = [
  { label: 'Every hour', value: '0 * * * *' },
  { label: 'Every day', value: '0 0 * * *' },
  { label: 'Every week', value: '0 0 * * 0' },
  { label: 'Every month', value: '0 0 1 * *' },
  { label: 'Every 5 min', value: '*/5 * * * *' },
  { label: 'Every 30 min', value: '*/30 * * * *' },
  { label: 'Weekdays 9am', value: '0 9 * * 1-5' },
]

const isEditing = computed(() => !!props.job)
const isValid = computed(() => {
  return form.value.name?.trim() && 
         form.value.schedule?.trim() && 
         form.value.command?.trim()
})

const scheduleDescription = computed(() => {
  const schedule = form.value.schedule || ''
  
  // Basic descriptions for common patterns
  const descriptions: Record<string, string> = {
    '* * * * *': 'Every minute',
    '0 * * * *': 'Every hour at minute 0',
    '*/5 * * * *': 'Every 5 minutes',
    '*/10 * * * *': 'Every 10 minutes',
    '*/15 * * * *': 'Every 15 minutes',
    '*/30 * * * *': 'Every 30 minutes',
    '0 0 * * *': 'Every day at midnight',
    '0 9 * * *': 'Every day at 9:00 AM',
    '0 0 * * 0': 'Every Sunday at midnight',
    '0 0 1 * *': 'Monthly on the 1st at midnight',
    '0 0 * * 1-5': 'Weekdays at midnight',
    '0 9 * * 1-5': 'Weekdays at 9:00 AM',
  }
  
  return descriptions[schedule] || 'Custom schedule'
})

const testCommand = async () => {
  if (!form.value.command || isTesting.value) return
  
  isTesting.value = true
  testOutput.value = ''
  
  const message: TestCronCommandMessage = {
    type: 'test-cron-command',
    command: form.value.command
  }
  ws.send(message)
  
  // Set up a timeout
  setTimeout(() => {
    if (isTesting.value) {
      isTesting.value = false
      testOutput.value = 'Test timed out'
    }
  }, 10000)
}

const save = () => {
  if (!isValid.value) return
  
  const job: CronJob = {
    id: props.job?.id || '',
    name: form.value.name!.trim(),
    schedule: form.value.schedule!.trim(),
    command: form.value.command!.trim(),
    enabled: form.value.enabled ?? true,
    createdAt: props.job?.createdAt || new Date().toISOString(),
    updatedAt: new Date().toISOString(),
    emailTo: form.value.emailTo?.trim() || undefined,
    logOutput: form.value.logOutput || false,
    tmuxSession: form.value.tmuxSession?.trim() || undefined,
  }
  
  emit('save', job)
}

const cancel = () => {
  emit('cancel')
}

onMounted(() => {
  // Populate form if editing
  if (props.job) {
    form.value = { ...props.job }
  }
  
  // Focus name input
  nextTick(() => {
    nameInput.value?.focus()
  })
  
  // Listen for test command output
  ws.onMessage<CronCommandOutputMessage>('cron-command-output', (msg) => {
    isTesting.value = false
    testOutput.value = msg.error || msg.output
  })
})
</script>

<style scoped>
.rotate-90 {
  transform: rotate(90deg);
}
</style>