<template>
  <div class="dashboard-view">
    <div class="view-header">
      <div class="header-content">
        <h1 class="view-title gradient-text">Dashboard</h1>
        <p class="view-subtitle">Overview of your decentralized storage network</p>
      </div>
    </div>

    <QuickStats
      :file-count="filesStore.files.length"
      :total-size="totalSize"
      :peer-count="nodeStore.peerCount"
      :sync-status="nodeStore.status"
    />

    <div class="dashboard-grid">
      <div class="dashboard-section glass">
        <div class="section-header">
          <h3>Recent Files</h3>
          <button class="view-all-btn" @click="$emit('navigate', 'files')">
            View All
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
              <polyline points="9 18 15 12 9 6"/>
            </svg>
          </button>
        </div>
        <div class="recent-files">
          <div v-for="file in recentFiles" :key="file.path" class="file-item">
            <div class="file-icon">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z"/>
                <polyline points="13 2 13 9 20 9"/>
              </svg>
            </div>
            <div class="file-info">
              <div class="file-name">{{ getFileName(file.path) }}</div>
              <div class="file-size">{{ formatBytes(file.size) }}</div>
            </div>
            <div v-if="file.synced" class="sync-badge">
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
                <polyline points="20 6 9 17 4 12"/>
              </svg>
            </div>
          </div>
        </div>
      </div>

      <div class="dashboard-section glass">
        <div class="section-header">
          <h3>Network Activity</h3>
        </div>
        <NetworkStats
          :block-count="nodeStore.blockCount"
          :peer-count="nodeStore.peerCount"
          :storage-used="nodeStore.storageUsed"
          :uptime="uptime"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useFilesStore } from '../stores/filesStore'
import { useNodeStore } from '../stores/nodeStore'
import QuickStats from './QuickStats.vue'
import NetworkStats from './NetworkStats.vue'

const filesStore = useFilesStore()
const nodeStore = useNodeStore()

defineEmits<{
  navigate: [view: string]
}>()

const totalSize = computed(() => {
  return filesStore.files.reduce((sum, file) => sum + file.size, 0)
})

const recentFiles = computed(() => {
  return filesStore.files
    .sort((a, b) => (b.modifiedAt || 0) - (a.modifiedAt || 0))
    .slice(0, 5)
})

const uptime = computed(() => {
  if (!nodeStore.metrics) return '0s'
  const seconds = nodeStore.metrics.uptime_seconds
  const hours = Math.floor(seconds / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)
  if (hours > 0) return `${hours}h ${minutes}m`
  if (minutes > 0) return `${minutes}m`
  return `${seconds}s`
})

const getFileName = (path: string): string => {
  return path.split('/').pop() || path.split('\\').pop() || path
}

const formatBytes = (bytes: number): string => {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i]
}
</script>

<style scoped>
.dashboard-view {
  padding: var(--spacing-xl);
  animation: fadeIn 0.4s ease-out;
}

.view-header {
  margin-bottom: var(--spacing-xl);
  padding-bottom: var(--spacing-lg);
  border-bottom: var(--border-subtle);
}

.header-content {
  max-width: 800px;
}

.view-title {
  font-size: 36px;
  font-weight: 800;
  margin-bottom: var(--spacing-xs);
  letter-spacing: -0.5px;
}

.view-subtitle {
  font-size: 14px;
  color: var(--color-text-secondary);
  font-weight: 500;
}

.dashboard-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
  gap: var(--spacing-lg);
}

.dashboard-section {
  padding: var(--spacing-xl);
  border-radius: var(--radius-lg);
  animation: fadeIn 0.5s ease-out both;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--spacing-lg);
  padding-bottom: var(--spacing-md);
  border-bottom: var(--border-subtle);
}

.section-header h3 {
  font-size: 18px;
  font-weight: 700;
  color: var(--color-text-primary);
  margin: 0;
}

.view-all-btn {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  padding: 8px 16px;
  background: rgba(255, 255, 255, 0.05);
  border: var(--border-subtle);
  border-radius: var(--radius-md);
  color: var(--color-text-secondary);
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.view-all-btn:hover {
  background: rgba(255, 255, 255, 0.08);
  color: var(--color-accent-primary);
  border-color: var(--color-accent-primary);
  transform: translateX(4px);
}

.recent-files {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
}

.file-item {
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
  padding: var(--spacing-md);
  background: rgba(255, 255, 255, 0.02);
  border: var(--border-subtle);
  border-radius: var(--radius-md);
  transition: all var(--transition-fast);
}

.file-item:hover {
  background: rgba(255, 255, 255, 0.05);
  border-color: rgba(255, 255, 255, 0.1);
  transform: translateX(4px);
}

.file-icon {
  width: 40px;
  height: 40px;
  background: rgba(0, 255, 136, 0.1);
  border-radius: var(--radius-md);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-accent-primary);
  flex-shrink: 0;
}

.file-info {
  flex: 1;
  min-width: 0;
}

.file-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text-primary);
  margin-bottom: 2px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.file-size {
  font-size: 12px;
  color: var(--color-text-tertiary);
}

.sync-badge {
  width: 24px;
  height: 24px;
  background: rgba(0, 255, 136, 0.15);
  border: 1px solid rgba(0, 255, 136, 0.3);
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-accent-primary);
  flex-shrink: 0;
}
</style>
