<template>
  <div 
    :class="['file-card', 'glass', { selected: isSelected, folder: file.isFolder }]"
    @click="handleClick"
    @dblclick="handleDoubleClick"
    @contextmenu.prevent="$emit('contextmenu', $event)"
  >
    <div class="file-icon-wrapper">
      <div class="file-icon" v-html="getFileIconSVG()"></div>
      <div v-if="file.synced && !file.isFolder" class="sync-badge">
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
          <polyline points="20 6 9 17 4 12"/>
        </svg>
      </div>
      <div v-if="file.sharedWith?.length" class="shared-badge" :title="`Shared with ${file.sharedWith.length} users`">
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
          <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/>
          <circle cx="9" cy="7" r="4"/>
          <path d="M23 21v-2a4 4 0 0 0-3-3.87"/>
          <path d="M16 3.13a4 4 0 0 1 0 7.75"/>
        </svg>
      </div>
    </div>
    <div class="file-info">
      <div class="file-name" :title="file.path">{{ fileName }}</div>
      <div class="file-meta">
        <span v-if="!file.isFolder" class="file-size">{{ formatBytes(file.size) }}</span>
        <span v-if="!file.isFolder" class="meta-dot">•</span>
        <span v-if="!file.isFolder" class="file-blocks">{{ file.blocks }} blocks</span>
        <span class="file-type-badge">{{ fileExtension }}</span>
      </div>
    </div>
    <div class="file-actions" @click.stop>
      <slot name="actions"></slot>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { FileItem } from '../stores/filesStore'

const props = defineProps<{
  file: FileItem
  isSelected: boolean
}>()

const emit = defineEmits<{
  click: [event: MouseEvent]
  dblclick: [event: MouseEvent]
  contextmenu: [event: MouseEvent]
}>()

const fileName = computed(() => {
  return props.file.path.split('/').pop() || props.file.path.split('\\').pop() || props.file.path
})

const fileExtension = computed(() => {
  if (props.file.isFolder) return 'FOLDER'
  const ext = props.file.path.split('.').pop()?.toUpperCase()
  return ext || 'FILE'
})

const handleClick = (e: MouseEvent) => emit('click', e)
const handleDoubleClick = (e: MouseEvent) => emit('dblclick', e)

const formatBytes = (bytes: number): string => {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i]
}

const getFileIconSVG = (): string => {
  if (props.file.isFolder) {
    return '<svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>'
  }
  
  const ext = props.file.path.split('.').pop()?.toLowerCase()
  
  if (['jpg', 'jpeg', 'png', 'gif', 'webp', 'svg'].includes(ext || '')) {
    return '<svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"/><circle cx="8.5" cy="8.5" r="1.5"/><polyline points="21 15 16 10 5 21"/></svg>'
  }
  if (['mp4', 'avi', 'mkv', 'mov', 'webm'].includes(ext || '')) {
    return '<svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="23 7 16 12 23 17 23 7"/><rect x="1" y="5" width="15" height="14" rx="2" ry="2"/></svg>'
  }
  if (['mp3', 'wav', 'flac', 'ogg'].includes(ext || '')) {
    return '<svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 18V5l12-2v13"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="16" r="3"/></svg>'
  }
  if (ext === 'pdf') {
    return '<svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/></svg>'
  }
  if (['js', 'ts', 'py', 'rs', 'java', 'cpp', 'c', 'go', 'rb'].includes(ext || '')) {
    return '<svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="16 18 22 12 16 6"/><polyline points="8 6 2 12 8 18"/></svg>'
  }
  if (['zip', 'rar', '7z', 'tar', 'gz'].includes(ext || '')) {
    return '<svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/></svg>'
  }
  return '<svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z"/><polyline points="13 2 13 9 20 9"/></svg>'
}
</script>

<style scoped>
.file-card {
  position: relative;
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
  padding: var(--spacing-lg);
  border-radius: var(--radius-lg);
  transition: all var(--transition-fast);
  cursor: pointer;
  will-change: transform;
  transform: translateZ(0);
  backface-visibility: hidden;
}

.file-card::before {
  content: '';
  position: absolute;
  inset: 0;
  background: var(--gradient-primary);
  opacity: 0;
  transition: opacity var(--transition-fast);
  pointer-events: none;
  border-radius: var(--radius-lg);
}

.file-card:hover {
  transform: translateY(-4px) translateZ(0);
  box-shadow: var(--shadow-lg);
}

.file-card:hover::before {
  opacity: 0.03;
}

.file-card.selected {
  border-color: var(--color-accent-primary);
  background: rgba(0, 255, 136, 0.05);
}

.file-card.selected::before {
  opacity: 0.08;
}

.file-card.folder .file-icon {
  color: var(--color-accent-secondary);
}

.file-icon-wrapper {
  position: relative;
  flex-shrink: 0;
}

.file-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 56px;
  height: 56px;
  background: rgba(255, 255, 255, 0.03);
  border-radius: var(--radius-md);
  color: var(--color-accent-primary);
  transition: all var(--transition-base);
}

.file-card:hover .file-icon {
  background: rgba(0, 255, 136, 0.1);
  transform: scale(1.05);
}

.file-info {
  flex: 1;
  min-width: 0;
}

.file-name {
  font-size: 15px;
  font-weight: 600;
  color: var(--color-text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-bottom: 6px;
  transition: color var(--transition-fast);
}

.file-card:hover .file-name {
  color: var(--color-accent-primary);
}

.file-meta {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  font-size: 12px;
  color: var(--color-text-tertiary);
  font-weight: 500;
}

.meta-dot {
  opacity: 0.5;
}

.file-type-badge {
  padding: 2px 8px;
  background: rgba(255, 255, 255, 0.05);
  border-radius: var(--radius-sm);
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--color-accent-secondary);
}

.file-actions {
  display: flex;
  gap: var(--spacing-xs);
  opacity: 0;
  transform: translateX(-10px);
  transition: all var(--transition-base);
}

.file-card:hover .file-actions {
  opacity: 1;
  transform: translateX(0);
}

.sync-badge, .shared-badge {
  position: absolute;
  width: 24px;
  height: 24px;
  backdrop-filter: blur(10px);
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  animation: fadeIn 0.3s ease-out;
}

.sync-badge {
  top: -4px;
  right: -4px;
  background: rgba(0, 255, 136, 0.15);
  border: 1px solid rgba(0, 255, 136, 0.3);
  color: var(--color-accent-primary);
  box-shadow: 0 0 12px rgba(0, 255, 136, 0.3);
}

.shared-badge {
  bottom: -4px;
  right: -4px;
  background: rgba(0, 204, 255, 0.15);
  border: 1px solid rgba(0, 204, 255, 0.3);
  color: var(--color-accent-secondary);
  box-shadow: 0 0 12px rgba(0, 204, 255, 0.3);
}
</style>
