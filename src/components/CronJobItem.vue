<template>
  <div 
    class="p-2 rounded hover-bg"
    style="background: var(--bg-tertiary)"
  >
    <div class="flex items-start justify-between">
      <!-- Job Info -->
      <div class="flex-1 min-w-0">
        <div class="flex items-center space-x-2">
          <!-- Status indicator -->
          <div 
            class="w-2 h-2 rounded-full flex-shrink-0"
            :class="job.enabled ? 'bg-green-500' : 'bg-gray-500'"
            :title="job.enabled ? 'Enabled' : 'Disabled'"
          ></div>
          
          <!-- Job name -->
          <span class="text-xs font-medium truncate" style="color: var(--text-primary)">
            {{ job.name }}
          </span>
        </div>
        
        <!-- Schedule -->
        <div class="mt-1 text-xs" style="color: var(--text-secondary)">
          {{ formatSchedule(job.schedule) }}
        </div>
        
        <!-- Command preview -->
        <div class="mt-1 text-xs font-mono truncate" style="color: var(--text-tertiary)">
          {{ job.command }}
        </div>
        
        <!-- Next run time -->
        <div v-if="job.nextRun" class="mt-1 text-xs" style="color: var(--text-tertiary)">
          Next: {{ formatNextRun(job.nextRun) }}
        </div>
      </div>
      
      <!-- Actions -->
      <div class="flex items-center space-x-1 ml-2">
        <!-- Toggle button -->
        <button
          @click="$emit('toggle', job)"
          class="p-1 rounded hover-bg"
          :title="job.enabled ? 'Disable' : 'Enable'"
        >
          <svg v-if="job.enabled" class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 9v6m4-6v6m7-3a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <svg v-else class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
        </button>
        
        <!-- Edit button -->
        <button
          @click="$emit('edit', job)"
          class="p-1 rounded hover-bg"
          title="Edit"
        >
          <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
          </svg>
        </button>
        
        <!-- Test button -->
        <button
          @click="$emit('test', job)"
          class="p-1 rounded hover-bg"
          title="Test Run"
        >
          <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
          </svg>
        </button>
        
        <!-- Delete button -->
        <button
          @click="$emit('delete', job)"
          class="p-1 rounded hover-bg text-red-500"
          title="Delete"
        >
          <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
          </svg>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { CronJob } from '@/types'

interface Props {
  job: CronJob
}

defineProps<Props>()

defineEmits<{
  edit: [job: CronJob]
  toggle: [job: CronJob]
  delete: [job: CronJob]
  test: [job: CronJob]
}>()

// Format cron schedule in human-readable form
const formatSchedule = (schedule: string): string => {
  // Basic mappings for common patterns
  const patterns: Record<string, string> = {
    '0 * * * *': 'Every hour',
    '*/5 * * * *': 'Every 5 minutes',
    '*/10 * * * *': 'Every 10 minutes',
    '*/15 * * * *': 'Every 15 minutes',
    '*/30 * * * *': 'Every 30 minutes',
    '0 0 * * *': 'Daily at midnight',
    '0 9 * * *': 'Daily at 9:00 AM',
    '0 0 * * 0': 'Weekly on Sunday',
    '0 0 1 * *': 'Monthly on the 1st',
    '0 0 * * 1-5': 'Weekdays at midnight',
  }
  
  return patterns[schedule] || schedule
}

// Format next run time
const formatNextRun = (nextRun: string): string => {
  const date = new Date(nextRun)
  const now = new Date()
  const diff = date.getTime() - now.getTime()
  
  // If in the past, show "Overdue"
  if (diff < 0) return 'Overdue'
  
  // Format relative time
  const minutes = Math.floor(diff / 60000)
  const hours = Math.floor(minutes / 60)
  const days = Math.floor(hours / 24)
  
  if (days > 0) return `in ${days} day${days > 1 ? 's' : ''}`
  if (hours > 0) return `in ${hours} hour${hours > 1 ? 's' : ''}`
  if (minutes > 0) return `in ${minutes} minute${minutes > 1 ? 's' : ''}`
  return 'soon'
}
</script>

<style scoped>
.hover-bg:hover {
  filter: brightness(1.2);
}
</style>