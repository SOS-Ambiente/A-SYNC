<template>
  <div class="peer-card glass">
    <div class="peer-header">
      <div class="peer-status" :class="peer.status">
        <div class="status-pulse"></div>
      </div>
      <div class="peer-address">{{ peer.address }}</div>
      <button class="remove-btn" @click.stop="$emit('remove', peer)" title="Remove">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
          <line x1="18" y1="6" x2="6" y2="18"/>
          <line x1="6" y1="6" x2="18" y2="18"/>
        </svg>
      </button>
    </div>
    
    <div class="peer-stats">
      <div class="peer-stat">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/>
        </svg>
        <span>{{ peer.blocks }} blocks</span>
      </div>
      <div class="peer-stat">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/>
        </svg>
        <span>{{ peer.latency }}ms</span>
      </div>
    </div>
    
    <div v-if="showDetails" class="peer-details">
      <div class="detail-row">
        <span class="detail-label">Connection:</span>
        <span class="detail-value">{{ peer.status === 'online' ? 'Active' : 'Inactive' }}</span>
      </div>
      <div class="detail-row">
        <span class="detail-label">Protocol:</span>
        <span class="detail-value">WebRTC</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
export interface Peer {
  id: string
  address: string
  status: 'online' | 'offline'
  blocks: number
  latency: number
}

withDefaults(defineProps<{
  peer: Peer
  showDetails?: boolean
}>(), {
  showDetails: false
})

defineEmits<{
  remove: [peer: Peer]
}>()
</script>

<style scoped>
.peer-card {
  padding: var(--spacing-lg);
  border-radius: var(--radius-lg);
  transition: all var(--transition-base);
  animation: fadeIn 0.5s ease-out both;
}

.peer-card:hover {
  transform: translateY(-4px);
  box-shadow: var(--shadow-md);
  border-color: var(--color-accent-primary);
}

.peer-header {
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
  margin-bottom: var(--spacing-md);
}

.peer-status {
  position: relative;
  width: 14px;
  height: 14px;
  border-radius: 50%;
  flex-shrink: 0;
}

.peer-status.online {
  background: var(--color-accent-primary);
  box-shadow: 0 0 16px rgba(0, 255, 136, 0.6);
}

.peer-status.offline {
  background: var(--color-text-tertiary);
}

.status-pulse {
  position: absolute;
  inset: -4px;
  border-radius: 50%;
  border: 2px solid var(--color-accent-primary);
  opacity: 0;
}

.peer-status.online .status-pulse {
  animation: pulse 2s ease-in-out infinite;
}

.peer-address {
  flex: 1;
  font-size: 15px;
  font-weight: 600;
  color: var(--color-text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.remove-btn {
  width: 32px;
  height: 32px;
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

.peer-card:hover .remove-btn {
  opacity: 1;
}

.remove-btn:hover {
  background: rgba(255, 51, 102, 0.15);
  border-color: var(--color-accent-danger);
  color: var(--color-accent-danger);
  transform: rotate(90deg);
}

.peer-stats {
  display: flex;
  gap: var(--spacing-lg);
  padding-bottom: var(--spacing-md);
}

.peer-stat {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  font-size: 13px;
  color: var(--color-text-secondary);
  font-weight: 500;
}

.peer-stat svg {
  color: var(--color-accent-primary);
}

.peer-details {
  padding-top: var(--spacing-md);
  border-top: var(--border-subtle);
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
}

.detail-row {
  display: flex;
  justify-content: space-between;
  font-size: 13px;
}

.detail-label {
  color: var(--color-text-tertiary);
}

.detail-value {
  color: var(--color-text-primary);
  font-weight: 600;
}
</style>
