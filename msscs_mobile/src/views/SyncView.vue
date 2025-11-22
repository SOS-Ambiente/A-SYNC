<template>
  <div class="sync-view">
    <div class="header">
      <h1>Sync Status</h1>
      <button class="btn-refresh" @click="refresh">🔄</button>
    </div>

    <div class="stats-grid">
      <div class="stat-card">
        <div class="stat-icon">📦</div>
        <div class="stat-value">{{ metrics?.block_count || 0 }}</div>
        <div class="stat-label">Blocks</div>
      </div>

      <div class="stat-card">
        <div class="stat-icon">🌐</div>
        <div class="stat-value">{{ metrics?.peer_count || 0 }}</div>
        <div class="stat-label">Peers</div>
      </div>

      <div class="stat-card">
        <div class="stat-icon">💾</div>
        <div class="stat-value">{{ formatSize(metrics?.storage_bytes || 0) }}</div>
        <div class="stat-label">Storage</div>
      </div>

      <div class="stat-card">
        <div class="stat-icon">⏱️</div>
        <div class="stat-value">{{ formatUptime(metrics?.uptime_seconds || 0) }}</div>
        <div class="stat-label">Uptime</div>
      </div>
    </div>

    <div class="section">
      <h2>Performance</h2>
      <div class="performance-card">
        <div class="performance-row">
          <span>Total Requests</span>
          <span class="value">{{ metrics?.requests_total || 0 }}</span>
        </div>
        <div class="performance-row">
          <span>Failed Requests</span>
          <span class="value error">{{ metrics?.requests_failed || 0 }}</span>
        </div>
        <div class="performance-row">
          <span>Success Rate</span>
          <span class="value success">{{ formatPercent(metrics?.success_rate || 0) }}</span>
        </div>
      </div>
    </div>

    <div class="section">
      <h2>Network Status</h2>
      <div class="status-card">
        <div class="status-indicator" :class="statusClass">
          <span class="status-dot">●</span>
          <span class="status-text">{{ statusText }}</span>
        </div>
        <p class="status-desc">{{ statusDescription }}</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useNodeStore } from '../stores/nodeStore'

const nodeStore = useNodeStore()

const metrics = computed(() => nodeStore.metrics)

const statusClass = computed(() => {
  const status = nodeStore.status
  return {
    'status-online': status === 'online',
    'status-syncing': status === 'syncing',
    'status-offline': status === 'offline',
  }
})

const statusText = computed(() => {
  const status = nodeStore.status
  return status.charAt(0).toUpperCase() + status.slice(1)
})

const statusDescription = computed(() => {
  const status = nodeStore.status
  const peerCount = nodeStore.peerCount
  
  if (status === 'online') {
    return `Connected to ${peerCount} peer${peerCount !== 1 ? 's' : ''}`
  } else if (status === 'syncing') {
    return 'Synchronizing with network...'
  } else {
    return 'Not connected to any peers'
  }
})

let refreshInterval: number | null = null

onMounted(() => {
  refresh()
  refreshInterval = window.setInterval(refresh, 5000)
})

onUnmounted(() => {
  if (refreshInterval) {
    clearInterval(refreshInterval)
  }
})

async function refresh() {
  await nodeStore.updateMetrics()
}

function formatSize(bytes: number): string {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return Math.round(bytes / Math.pow(k, i) * 10) / 10 + ' ' + sizes[i]
}

function formatUptime(seconds: number): string {
  if (seconds < 60) return `${seconds}s`
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m`
  if (seconds < 86400) return `${Math.floor(seconds / 3600)}h`
  return `${Math.floor(seconds / 86400)}d`
}

function formatPercent(value: number): string {
  return `${Math.round(value * 100)}%`
}
</script>

<style scoped>
.sync-view {
  padding: 16px;
  padding-bottom: 80px;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

h1 {
  font-size: 24px;
  font-weight: 600;
}

.btn-refresh {
  width: 48px;
  height: 48px;
  border: none;
  background: #222;
  border-radius: 50%;
  font-size: 24px;
  cursor: pointer;
  transition: transform 0.3s;
}

.btn-refresh:active {
  transform: rotate(180deg);
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
  margin-bottom: 32px;
}

.stat-card {
  padding: 20px;
  background: #111;
  border-radius: 16px;
  text-align: center;
}

.stat-icon {
  font-size: 32px;
  margin-bottom: 8px;
}

.stat-value {
  font-size: 24px;
  font-weight: 600;
  margin-bottom: 4px;
}

.stat-label {
  font-size: 12px;
  color: #888;
}

.section {
  margin-bottom: 32px;
}

.section h2 {
  font-size: 18px;
  font-weight: 600;
  margin-bottom: 12px;
}

.performance-card {
  background: #111;
  border-radius: 16px;
  padding: 16px;
}

.performance-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 0;
  border-bottom: 1px solid #222;
}

.performance-row:last-child {
  border-bottom: none;
}

.performance-row .value {
  font-weight: 600;
}

.performance-row .value.error {
  color: #ff4444;
}

.performance-row .value.success {
  color: #00ff88;
}

.status-card {
  background: #111;
  border-radius: 16px;
  padding: 20px;
}

.status-indicator {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.status-dot {
  font-size: 16px;
}

.status-indicator.status-online .status-dot {
  color: #00ff88;
}

.status-indicator.status-syncing .status-dot {
  color: #ffaa00;
  animation: pulse 1.5s ease-in-out infinite;
}

.status-indicator.status-offline .status-dot {
  color: #ff4444;
}

.status-text {
  font-size: 18px;
  font-weight: 600;
}

.status-desc {
  color: #888;
  font-size: 14px;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}
</style>
