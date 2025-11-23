<template>
  <div class="file-actions">
    <button v-if="canPreview" class="action-btn" @click.stop="$emit('preview', file)" title="Preview">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/>
        <circle cx="12" cy="12" r="3"/>
      </svg>
    </button>
    <button class="action-btn" @click.stop="$emit('open', file)" title="Open">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6M15 3h6v6M10 14L21 3"/>
      </svg>
    </button>
    <button class="action-btn" @click.stop="$emit('download', file)" title="Download">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3"/>
      </svg>
    </button>
    <button class="action-btn danger" @click.stop="$emit('delete', file)" title="Delete">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <polyline points="3 6 5 6 21 6"/>
        <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
      </svg>
    </button>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { FileItem } from '../stores/filesStore'

const props = defineProps<{
  file: FileItem
}>()

defineEmits<{
  preview: [file: FileItem]
  open: [file: FileItem]
  download: [file: FileItem]
  delete: [file: FileItem]
}>()

const canPreview = computed(() => {
  const ext = props.file.path.split('.').pop()?.toLowerCase()
  const previewable = ['jpg', 'jpeg', 'png', 'gif', 'webp', 'txt', 'md', 'json', 'mp4', 'webm']
  return previewable.includes(ext || '')
})
</script>

<style scoped>
.file-actions {
  display: flex;
  gap: var(--spacing-xs);
}

.action-btn {
  width: 36px;
  height: 36px;
  background: rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(10px);
  border: var(--border-subtle);
  border-radius: var(--radius-sm);
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all var(--transition-fast);
  display: flex;
  align-items: center;
  justify-content: center;
  will-change: transform;
  transform: translateZ(0);
  backface-visibility: hidden;
}

.action-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  border-color: var(--color-accent-primary);
  color: var(--color-accent-primary);
  transform: translateY(-2px) translateZ(0) scale(1.05);
}

.action-btn:active {
  transform: translateY(0) translateZ(0) scale(0.95);
  transition: transform 50ms;
}

.action-btn.danger:hover {
  background: rgba(255, 51, 102, 0.1);
  border-color: var(--color-accent-danger);
  color: var(--color-accent-danger);
}
</style>
