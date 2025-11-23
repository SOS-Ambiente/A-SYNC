<template>
  <div class="network-stats">
    <div class="stats-grid">
      <div v-for="(stat, index) in stats" :key="stat.label" class="stat-card glass" :style="{ animationDelay: `${index * 0.1}s` }">
        <div class="stat-icon-wrapper" :class="stat.color">
          <div class="stat-icon" v-html="stat.icon"></div>
        </div>
        <div class="stat-info">
          <div class="stat-value">{{ stat.value }}</div>
          <div class="stat-label">{{ stat.label }}</div>
        </div>
        <div v-if="stat.trend" class="stat-trend" :class="stat.trend">
          <svg v-if="stat.trend === 'up'" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <polyline points="23 6 13.5 15.5 8.5 10.5 1 18"/>
            <polyline points="17 6 23 6 23 12"/>
          </svg>
          <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <polyline points="23 18 13.5 8.5 8.5 13.5 1 6"/>
            <polyline points="17 18 23 18 23 12"/>
          </svg>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

export interface NetworkStat {
  label: string
  value: string | number
  icon: string
  color: string
  trend?: 'up' | 'down'
}

const props = defineProps<{
  blockCount: number
  peerCount: number
  storageUsed: number
  uptime: string
}>()

const stats = computed<NetworkStat[]>(() => [
  {
    label: 'Total Blocks',
    value: props.blockCount,
    icon: '<svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/></svg>',
    color: 'blocks',
    trend: 'up'
  },
  {
    label: 'Connected Peers',
    value: props.peerCount,
    icon: '<svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><path d="M12 2a14.5 14.5 0 0 0 0 20 14.5 14.5 0 0 0 0-20M2 12h20"/></svg>',
    color: 'peers'
  },
  {
    label: 'Storage Used',
    value: formatBytes(props.storageUsed),
    icon: '<svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/><polyline points="3.27 6.96 12 12.01 20.73 6.96"/><line x1="12" y1="22.08" x2="12" y2="12"/></svg>',
    color: 'storage',
    trend: 'up'
  },
  {
    label: 'Uptime',
    value: props.uptime,
    icon: '<svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>',
    color: 'uptime'
  }
])

const formatBytes = (bytes: number): string => {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i]
}
</script>

<style scoped>
.network-stats {
  margin-bottom: var(--spacing-xl);
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
  gap: var(--spacing-md);
}

.stat-card {
  position: relative;
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
  padding: var(--spacing-lg);
  border-radius: var(--radius-lg);
  transition: all var(--transition-base);
  animation: fadeIn 0.5s ease-out both;
  overflow: hidden;
}

.stat-card::before {
  content: '';
  position: absolute;
  inset: 0;
  background: var(--gradient-primary);
  opacity: 0;
  transition: opacity var(--transition-base);
  pointer-events: none;
}

.stat-card:hover {
  transform: translateY(-4px);
  box-shadow: var(--shadow-md);
}

.stat-card:hover::before {
  opacity: 0.03;
}

.stat-icon-wrapper {
  position: relative;
  width: 56px;
  height: 56px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-md);
  flex-shrink: 0;
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

.stat-icon-wrapper.storage {
  background: rgba(255, 0, 255, 0.1);
  color: var(--color-accent-tertiary);
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
  position: relative;
  z-index: 1;
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

.stat-trend {
  position: absolute;
  top: var(--spacing-md);
  right: var(--spacing-md);
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  transition: all var(--transition-base);
}

.stat-trend.up {
  background: rgba(0, 255, 136, 0.15);
  color: var(--color-accent-primary);
}

.stat-trend.down {
  background: rgba(255, 51, 102, 0.15);
  color: var(--color-accent-danger);
}
</style>
