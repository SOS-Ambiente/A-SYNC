<template>
  <div class="folder-list">
    <div class="list-header">
      <h4>📁 Shared Folders ({{ folders.length }})</h4>
      <button class="btn-secondary" @click="$emit('create')">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
          <line x1="12" y1="5" x2="12" y2="19"/>
          <line x1="5" y1="12" x2="19" y2="12"/>
        </svg>
        <span>New Folder</span>
      </button>
    </div>
    
    <div class="folders-grid">
      <div v-for="folder in folders" :key="folder.id" class="folder-card glass" @click="$emit('select', folder)">
        <div class="folder-icon">
          <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
          </svg>
        </div>
        <div class="folder-info">
          <div class="folder-name">{{ folder.name }}</div>
          <div class="folder-path">{{ folder.path }}</div>
          <div class="folder-meta">
            <span>{{ Object.keys(folder.files).length }} files</span>
            <span>•</span>
            <span>{{ Object.keys(folder.members).length }} members</span>
          </div>
        </div>
        <button class="folder-action" @click.stop="$emit('manage', folder)">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="3"/>
            <path d="M12 1v6m0 6v6M5.64 5.64l4.24 4.24m4.24 4.24l4.24 4.24M1 12h6m6 0h6M5.64 18.36l4.24-4.24m4.24-4.24l4.24-4.24"/>
          </svg>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { SharedFolder } from '../stores/workspaceStore'

defineProps<{
  folders: SharedFolder[]
}>()

defineEmits<{
  create: []
  select: [folder: SharedFolder]
  manage: [folder: SharedFolder]
}>()
</script>

<style scoped>
.folder-list {
  margin-bottom: var(--spacing-xl);
}

.list-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--spacing-md);
}

.list-header h4 {
  font-size: 16px;
  font-weight: 700;
  color: var(--color-text-primary);
}

.btn-secondary {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  padding: 8px 16px;
  background: rgba(255, 255, 255, 0.05);
  border: var(--border-subtle);
  border-radius: var(--radius-md);
  color: var(--color-text-primary);
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-secondary:hover {
  background: rgba(255, 255, 255, 0.08);
  border-color: var(--color-accent-primary);
  transform: translateY(-2px);
}

.folders-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: var(--spacing-md);
}

.folder-card {
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
  padding: var(--spacing-lg);
  border-radius: var(--radius-lg);
  cursor: pointer;
  transition: all var(--transition-base);
}

.folder-card:hover {
  transform: translateY(-4px);
  box-shadow: var(--shadow-md);
  border-color: var(--color-accent-secondary);
}

.folder-icon {
  width: 56px;
  height: 56px;
  background: rgba(0, 204, 255, 0.1);
  border-radius: var(--radius-md);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-accent-secondary);
  flex-shrink: 0;
  transition: all var(--transition-base);
}

.folder-card:hover .folder-icon {
  transform: scale(1.1);
}

.folder-info {
  flex: 1;
  min-width: 0;
}

.folder-name {
  font-size: 15px;
  font-weight: 600;
  color: var(--color-text-primary);
  margin-bottom: 4px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.folder-path {
  font-size: 12px;
  color: var(--color-text-tertiary);
  margin-bottom: 6px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.folder-meta {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  font-size: 12px;
  color: var(--color-text-secondary);
}

.folder-action {
  width: 36px;
  height: 36px;
  background: rgba(255, 255, 255, 0.03);
  border: var(--border-subtle);
  border-radius: var(--radius-sm);
  color: var(--color-text-tertiary);
  cursor: pointer;
  transition: all var(--transition-fast);
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
}

.folder-card:hover .folder-action {
  opacity: 1;
}

.folder-action:hover {
  background: rgba(255, 255, 255, 0.08);
  color: var(--color-accent-primary);
  transform: rotate(90deg);
}
</style>
