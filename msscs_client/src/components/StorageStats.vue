<template>
  <div class="storage-stats">
    <h3>📊 Armazenamento Distribuído</h3>
    
    <div class="stats-grid">
      <div class="stat-card">
        <div class="stat-icon">📁</div>
        <div class="stat-info">
          <div class="stat-value">{{ stats.total_files }}</div>
          <div class="stat-label">Arquivos Totais</div>
        </div>
      </div>
      
      <div class="stat-card">
        <div class="stat-icon">🔄</div>
        <div class="stat-info">
          <div class="stat-value">{{ stats.replicated_files }}</div>
          <div class="stat-label">Arquivos Replicados</div>
        </div>
      </div>
      
      <div class="stat-card">
        <div class="stat-icon">🌐</div>
        <div class="stat-info">
          <div class="stat-value">{{ stats.online_hosts }} / {{ stats.total_hosts }}</div>
          <div class="stat-label">Hosts Online</div>
        </div>
      </div>
      
      <div class="stat-card">
        <div class="stat-icon">💾</div>
        <div class="stat-info">
          <div class="stat-value">{{ formatBytes(stats.total_size) }}</div>
          <div class="stat-label">Tamanho Total</div>
        </div>
      </div>
      
      <div class="stat-card full-width">
        <div class="stat-label">Capacidade da Rede</div>
        <div class="progress-bar">
          <div 
            class="progress-fill" 
            :style="{ width: networkUsagePercent + '%' }"
          ></div>
        </div>
        <div class="progress-text">
          {{ formatBytes(stats.network_used) }} / {{ formatBytes(stats.network_capacity) }}
          ({{ networkUsagePercent.toFixed(1) }}%)
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

interface StorageStats {
  total_files: number
  total_size: number
  replicated_files: number
  online_hosts: number
  total_hosts: number
  network_capacity: number
  network_used: number
}

const props = defineProps<{
  stats: StorageStats
}>()

const networkUsagePercent = computed(() => {
  if (props.stats.network_capacity === 0) return 0
  return (props.stats.network_used / props.stats.network_capacity) * 100
})

const formatBytes = (bytes: number): string => {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i]
}
</script>

<style scoped>
.storage-stats {
  background: #2a2a2a;
  border-radius: 8px;
  padding: 20px;
  margin-bottom: 20px;
}

.storage-stats h3 {
  margin: 0 0 20px 0;
  color: #fff;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 15px;
}

.stat-card {
  background: #1a1a1a;
  border: 1px solid #3a3a3a;
  border-radius: 6px;
  padding: 15px;
  display: flex;
  align-items: center;
  gap: 12px;
}

.stat-card.full-width {
  grid-column: 1 / -1;
  flex-direction: column;
  align-items: stretch;
}

.stat-icon {
  font-size: 32px;
}

.stat-info {
  flex: 1;
}

.stat-value {
  font-size: 24px;
  font-weight: bold;
  color: #4a9eff;
  margin-bottom: 4px;
}

.stat-label {
  font-size: 13px;
  color: #888;
  margin-bottom: 8px;
}

.progress-bar {
  width: 100%;
  height: 20px;
  background: #0a0a0a;
  border-radius: 10px;
  overflow: hidden;
  margin: 10px 0;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, #4a9eff, #6ab0ff);
  transition: width 0.3s ease;
}

.progress-text {
  text-align: center;
  font-size: 13px;
  color: #aaa;
}
</style>
