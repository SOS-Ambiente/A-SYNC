<template>
  <div class="sync-status" :class="{ 'sync-active': isSyncing }">
    <div class="sync-icon">
      <span :class="{ spinning: isSyncing }">{{ syncIcon }}</span>
    </div>
    <div class="sync-info">
      <div class="sync-title">{{ syncTitle }}</div>
      <div class="sync-details">{{ syncDetails }}</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

interface Props {
  status: 'idle' | 'syncing' | 'error' | 'success'
  filesCount?: number
  bytesTransferred?: number
  totalBytes?: number
}

const props = withDefaults(defineProps<Props>(), {
  status: 'idle',
  filesCount: 0,
  bytesTransferred: 0,
  totalBytes: 0,
})

const syncIcon = computed(() => {
  switch (props.status) {
    case 'syncing': return '🔄'
    case 'success': return '✓'
    case 'error': return '✕'
    default: return '⏸'
  }
})

const syncTitle = computed(() => {
  switch (props.status) {
    case 'syncing': return 'Syncing...'
    case 'success': return 'Synced'
    case 'error': return 'Sync Failed'
    default: return 'Ready'
  }
})

const syncDetails = computed(() => {
  if (props.status === 'syncing' && props.totalBytes > 0) {
    const percent = Math.round((props.bytesTransferred / props.totalBytes) * 100)
    return `${percent}% • ${formatBytes(props.bytesTransferred)} / ${formatBytes(props.totalBytes)}`
  }
  if (props.filesCount > 0) {
    return `${props.filesCount} file${props.filesCount !== 1 ? 's' : ''}`
  }
  return 'No active transfers'
})

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i]
}
</script>

<style scoped>
.sync-status {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  background: #111;
  border-radius: 12px;
  transition: all 0.3s ease;
}

.sync-active {
  background: linear-gradient(90deg, #111 0%, #1a1a1a 100%);
  box-shadow: 0 2px 8px rgba(0, 255, 136, 0.1);
}

.sync-icon {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #222;
  border-radius: 50%;
  font-size: 20px;
}

.spinning {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.sync-info {
  flex: 1;
  min-width: 0;
}

.sync-title {
  font-size: 14px;
  font-weight: 600;
  color: #fff;
}

.sync-details {
  font-size: 12px;
  color: #888;
  margin-top: 2px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
