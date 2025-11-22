<template>
  <div class="sync-view">
    <div class="view-header">
      <div class="header-content">
        <h1 class="view-title gradient-text">Sync Status</h1>
        <p class="view-subtitle">Real-time synchronization monitoring</p>
      </div>
      <button class="btn-secondary" @click="refresh">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
          <path d="M21.5 2v6h-6M2.5 22v-6h6M2 11.5a10 10 0 0 1 18.8-4.3M22 12.5a10 10 0 0 1-18.8 4.2"/>
        </svg>
        <span>Refresh</span>
      </button>
    </div>

    <div class="sync-stats">
      <div class="stat-card glass">
        <div class="stat-icon-wrapper blocks">
          <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/>
          </svg>
        </div>
        <div class="stat-info">
          <div class="stat-value">{{ nodeStore.blockCount }}</div>
          <div class="stat-label">Total Blocks</div>
        </div>
      </div>

      <div class="stat-card glass">
        <div class="stat-icon-wrapper peers">
          <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10"/>
            <path d="M12 2a14.5 14.5 0 0 0 0 20 14.5 14.5 0 0 0 0-20M2 12h20"/>
          </svg>
        </div>
        <div class="stat-info">
          <div class="stat-value">{{ nodeStore.peerCount }}</div>
          <div class="stat-label">Connected Peers</div>
        </div>
      </div>

      <div class="stat-card glass">
        <div class="stat-icon-wrapper synced">
          <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="20 6 9 17 4 12"/>
          </svg>
        </div>
        <div class="stat-info">
          <div class="stat-value">{{ syncedFiles }}</div>
          <div class="stat-label">Synced Files</div>
        </div>
      </div>

      <div class="stat-card glass">
        <div class="stat-icon-wrapper uptime">
          <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10"/>
            <polyline points="12 6 12 12 16 14"/>
          </svg>
        </div>
        <div class="stat-info">
          <div class="stat-value">{{ uptime }}</div>
          <div class="stat-label">Uptime</div>
        </div>
      </div>
    </div>

    <div class="sync-activity glass">
      <h2 class="section-title">Recent Activity</h2>
      <div class="activity-list">
        <div v-for="activity in activities" :key="activity.id" class="activity-item">
          <div class="activity-icon" :class="activity.type">
            <svg v-if="activity.type === 'upload'" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M17 8l-5-5-5 5M12 3v12"/>
            </svg>
            <svg v-else-if="activity.type === 'download'" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3"/>
            </svg>
            <svg v-else width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M21.5 2v6h-6M2.5 22v-6h6M2 11.5a10 10 0 0 1 18.8-4.3M22 12.5a10 10 0 0 1-18.8 4.2"/>
            </svg>
          </div>
          <div class="activity-info">
            <div class="activity-title">{{ activity.title }}</div>
            <div class="activity-time">{{ activity.time }}</div>
          </div>
          <div class="activity-status" :class="activity.status">
            {{ activity.status }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useNodeStore } from '../stores/nodeStore'
import { useFilesStore } from '../stores/filesStore'

const nodeStore = useNodeStore()
const filesStore = useFilesStore()

const activities = ref([
  { id: 1, type: 'upload', title: 'document.pdf uploaded', time: '2 minutes ago', status: 'completed' },
  { id: 2, type: 'sync', title: 'Synced with peer 192.168.1.100', time: '5 minutes ago', status: 'completed' },
  { id: 3, type: 'download', title: 'image.png downloaded', time: '10 minutes ago', status: 'completed' },
])

const syncedFiles = computed(() => {
  return filesStore.files.filter(f => f.synced).length
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

const refresh = () => {
  nodeStore.updateMetrics()
  filesStore.loadFiles()
}
</script>

<style scoped>
.sync-view {
  padding: var(--spacing-xl);
  animation: fadeIn 0.4s ease-out;
}

.view-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: var(--spacing-xl);
  padding-bottom: var(--spacing-lg);
  border-bottom: var(--border-subtle);
}

.header-content {
  flex: 1;
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

.btn-secondary {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  padding: 12px 24px;
  background: rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(10px);
  border: var(--border-subtle);
  border-radius: var(--radius-md);
  color: var(--color-text-primary);
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all var(--transition-base);
}

.btn-secondary:hover {
  background: rgba(255, 255, 255, 0.08);
  border-color: rgba(255, 255, 255, 0.15);
  transform: translateY(-2px);
}

.sync-stats {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  gap: var(--spacing-md);
  margin-bottom: var(--spacing-xl);
}

.stat-card {
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
  padding: var(--spacing-lg);
  border-radius: var(--radius-lg);
  transition: all var(--transition-base);
  animation: fadeIn 0.5s ease-out both;
}

.stat-card:nth-child(1) { animation-delay: 0.1s; }
.stat-card:nth-child(2) { animation-delay: 0.2s; }
.stat-card:nth-child(3) { animation-delay: 0.3s; }
.stat-card:nth-child(4) { animation-delay: 0.4s; }

.stat-card:hover {
  transform: translateY(-4px);
  box-shadow: var(--shadow-md);
}

.stat-icon-wrapper {
  width: 56px;
  height: 56px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-md);
  transition: all var(--transition-base);
}

.stat-icon-wrapper.blocks {
  background: rgba(0, 255, 136, 0.1);
  color: var(--color-accent-primary);
}

.stat-icon-wrapper.peers {
  background: rgba(0, 204, 255, 0.1);
  color: var(--color-accent-secondary);
}

.stat-icon-wrapper.synced {
  background: rgba(0, 255, 136, 0.1);
  color: var(--color-accent-primary);
}

.stat-icon-wrapper.uptime {
  background: rgba(255, 170, 0, 0.1);
  color: var(--color-accent-warning);
}

.stat-card:hover .stat-icon-wrapper {
  transform: scale(1.1) rotate(5deg);
}

.stat-info {
  flex: 1;
}

.stat-value {
  font-size: 28px;
  font-weight: 800;
  color: var(--color-text-primary);
  margin-bottom: 4px;
  line-height: 1;
}

.stat-label {
  font-size: 12px;
  color: var(--color-text-tertiary);
  text-transform: uppercase;
  letter-spacing: 1px;
  font-weight: 600;
}

.sync-activity {
  border-radius: var(--radius-lg);
  padding: var(--spacing-xl);
  animation: fadeIn 0.5s ease-out 0.5s both;
}

.section-title {
  font-size: 20px;
  font-weight: 700;
  color: var(--color-text-primary);
  margin-bottom: var(--spacing-lg);
}

.activity-list {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
}

.activity-item {
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
  padding: var(--spacing-md);
  background: rgba(255, 255, 255, 0.02);
  border-radius: var(--radius-md);
  transition: all var(--transition-base);
}

.activity-item:hover {
  background: rgba(255, 255, 255, 0.05);
  transform: translateX(4px);
}

.activity-icon {
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-md);
  flex-shrink: 0;
  transition: all var(--transition-base);
}

.activity-icon.upload {
  background: rgba(0, 255, 136, 0.1);
  color: var(--color-accent-primary);
}

.activity-icon.download {
  background: rgba(0, 204, 255, 0.1);
  color: var(--color-accent-secondary);
}

.activity-icon.sync {
  background: rgba(255, 170, 0, 0.1);
  color: var(--color-accent-warning);
}

.activity-item:hover .activity-icon {
  transform: scale(1.1);
}

.activity-info {
  flex: 1;
}

.activity-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text-primary);
  margin-bottom: 4px;
}

.activity-time {
  font-size: 12px;
  color: var(--color-text-tertiary);
}

.activity-status {
  padding: 6px 16px;
  border-radius: var(--radius-full);
  font-size: 12px;
  font-weight: 700;
  text-transform: capitalize;
  letter-spacing: 0.5px;
}

.activity-status.completed {
  background: rgba(0, 255, 136, 0.15);
  color: var(--color-accent-primary);
}

.activity-status.pending {
  background: rgba(255, 170, 0, 0.15);
  color: var(--color-accent-warning);
}

.activity-status.failed {
  background: rgba(255, 51, 102, 0.15);
  color: var(--color-accent-danger);
}
</style>
