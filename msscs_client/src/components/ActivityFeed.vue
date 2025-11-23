<template>
  <div class="activity-feed glass">
    <div class="feed-header">
      <h2 class="feed-title">Recent Activity</h2>
      <button class="refresh-btn" @click="$emit('refresh')" title="Refresh">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
          <path d="M21.5 2v6h-6M2.5 22v-6h6M2 11.5a10 10 0 0 1 18.8-4.3M22 12.5a10 10 0 0 1-18.8 4.2"/>
        </svg>
      </button>
    </div>
    
    <div class="activity-list">
      <div v-for="activity in activities" :key="activity.id" class="activity-item">
        <div class="activity-icon" :class="activity.type">
          <svg v-if="activity.type === 'upload'" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M17 8l-5-5-5 5M12 3v12"/>
          </svg>
          <svg v-else-if="activity.type === 'download'" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3"/>
          </svg>
          <svg v-else-if="activity.type === 'sync'" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21.5 2v6h-6M2.5 22v-6h6M2 11.5a10 10 0 0 1 18.8-4.3M22 12.5a10 10 0 0 1-18.8 4.2"/>
          </svg>
          <svg v-else width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z"/>
            <polyline points="13 2 13 9 20 9"/>
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
</template>

<script setup lang="ts">
export interface Activity {
  id: number
  type: 'upload' | 'download' | 'sync' | 'file'
  title: string
  time: string
  status: 'completed' | 'pending' | 'failed'
}

defineProps<{
  activities: Activity[]
}>()

defineEmits<{
  refresh: []
}>()
</script>

<style scoped>
.activity-feed {
  border-radius: var(--radius-lg);
  padding: var(--spacing-xl);
  animation: fadeIn 0.5s ease-out 0.5s both;
}

.feed-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--spacing-lg);
  padding-bottom: var(--spacing-md);
  border-bottom: var(--border-subtle);
}

.feed-title {
  font-size: 20px;
  font-weight: 700;
  color: var(--color-text-primary);
  margin: 0;
}

.refresh-btn {
  width: 36px;
  height: 36px;
  background: rgba(255, 255, 255, 0.05);
  border: var(--border-subtle);
  border-radius: var(--radius-md);
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all var(--transition-fast);
  display: flex;
  align-items: center;
  justify-content: center;
}

.refresh-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: var(--color-accent-primary);
  transform: rotate(180deg);
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
  border: var(--border-subtle);
  transition: all var(--transition-base);
}

.activity-item:hover {
  background: rgba(255, 255, 255, 0.05);
  border-color: rgba(255, 255, 255, 0.1);
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

.activity-icon.file {
  background: rgba(255, 0, 255, 0.1);
  color: var(--color-accent-tertiary);
}

.activity-item:hover .activity-icon {
  transform: scale(1.1);
}

.activity-info {
  flex: 1;
  min-width: 0;
}

.activity-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text-primary);
  margin-bottom: 4px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
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
  flex-shrink: 0;
}

.activity-status.completed {
  background: rgba(0, 255, 136, 0.15);
  color: var(--color-accent-primary);
  border: 1px solid rgba(0, 255, 136, 0.3);
}

.activity-status.pending {
  background: rgba(255, 170, 0, 0.15);
  color: var(--color-accent-warning);
  border: 1px solid rgba(255, 170, 0, 0.3);
}

.activity-status.failed {
  background: rgba(255, 51, 102, 0.15);
  color: var(--color-accent-danger);
  border: 1px solid rgba(255, 51, 102, 0.3);
}
</style>
