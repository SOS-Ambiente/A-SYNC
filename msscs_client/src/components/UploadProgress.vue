<template>
  <Teleport to="body">
    <div v-if="hasActiveOperations" class="upload-overlay">
      <div class="upload-card glass">
        <div class="card-header">
          <h3>File Operations</h3>
          <button class="minimize-btn" @click="minimized = !minimized">
            <svg v-if="!minimized" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
              <line x1="5" y1="12" x2="19" y2="12"/>
            </svg>
            <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
              <polyline points="18 15 12 9 6 15"/>
            </svg>
          </button>
        </div>
        
        <div v-if="!minimized" class="operations-list">
          <!-- Upload Progress -->
          <div v-for="[path, data] in uploadProgress" :key="'up-' + path" class="operation-item">
            <div class="operation-header">
              <span class="operation-icon upload">📤</span>
              <span class="operation-name">{{ getFileName(path) }}</span>
              <span class="operation-percent">{{ data.progress }}%</span>
            </div>
            <div class="operation-bar">
              <div class="operation-fill upload" :style="{ width: data.progress + '%' }"></div>
            </div>
            <div class="operation-info">
              <span v-if="data.speed" class="operation-speed">{{ formatSpeed(data.speed) }}</span>
              <span v-if="data.eta" class="operation-eta">{{ formatTime(data.eta) }}</span>
            </div>
          </div>
          
          <!-- Download Progress -->
          <div v-for="[path, data] in downloadProgress" :key="'down-' + path" class="operation-item">
            <div class="operation-header">
              <span class="operation-icon download">📥</span>
              <span class="operation-name">{{ getFileName(path) }}</span>
              <span class="operation-percent">{{ data.progress }}%</span>
            </div>
            <div class="operation-bar">
              <div class="operation-fill download" :style="{ width: data.progress + '%' }"></div>
            </div>
            <div class="operation-info">
              <span v-if="data.speed" class="operation-speed">{{ formatSpeed(data.speed) }}</span>
              <span v-if="data.eta" class="operation-eta">{{ formatTime(data.eta) }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import type { ProgressData } from '../stores/filesStore'

const props = defineProps<{
  uploadProgress: Map<string, ProgressData>
  downloadProgress: Map<string, ProgressData>
}>()

const minimized = ref(false)

const hasActiveOperations = computed(() => {
  return props.uploadProgress.size > 0 || props.downloadProgress.size > 0
})

const getFileName = (path: string): string => {
  return path.split('/').pop() || path.split('\\').pop() || path
}

const formatSpeed = (blocksPerSec: number): string => {
  return `${blocksPerSec.toFixed(1)} blocks/s`
}

const formatTime = (seconds: number): string => {
  if (seconds < 60) return `${Math.round(seconds)}s`
  const mins = Math.floor(seconds / 60)
  const secs = Math.round(seconds % 60)
  return `${mins}m ${secs}s`
}
</script>

<style scoped>
.upload-overlay {
  position: fixed;
  bottom: var(--spacing-xl);
  right: var(--spacing-xl);
  z-index: 1000;
  animation: fadeIn 0.2s ease-out;
}

.upload-card {
  border-radius: var(--radius-xl);
  padding: var(--spacing-lg);
  min-width: 400px;
  max-width: 500px;
  box-shadow: var(--shadow-xl);
  animation: fadeIn 0.3s ease-out 0.1s both;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--spacing-md);
}

.card-header h3 {
  font-size: 16px;
  font-weight: 700;
  color: var(--color-text-primary);
  margin: 0;
}

.minimize-btn {
  width: 32px;
  height: 32px;
  background: rgba(255, 255, 255, 0.05);
  border: var(--border-subtle);
  border-radius: var(--radius-sm);
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all var(--transition-fast);
  display: flex;
  align-items: center;
  justify-content: center;
}

.minimize-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: var(--color-text-primary);
}

.operations-list {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
}

.operation-item {
  padding: var(--spacing-md);
  background: rgba(255, 255, 255, 0.02);
  border-radius: var(--radius-md);
  border: var(--border-subtle);
}

.operation-header {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  margin-bottom: var(--spacing-sm);
}

.operation-icon {
  font-size: 18px;
  flex-shrink: 0;
}

.operation-name {
  font-size: 14px;
  font-weight: 500;
  color: var(--color-text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  flex: 1;
}

.operation-percent {
  font-size: 13px;
  font-weight: 700;
  color: var(--color-accent-primary);
}

.operation-bar {
  position: relative;
  height: 6px;
  background: rgba(255, 255, 255, 0.05);
  border-radius: var(--radius-full);
  overflow: hidden;
  margin-bottom: var(--spacing-sm);
}

.operation-fill {
  height: 100%;
  border-radius: var(--radius-full);
  transition: width 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.operation-fill.upload {
  background: var(--gradient-primary);
  box-shadow: var(--glow-primary);
}

.operation-fill.download {
  background: linear-gradient(90deg, #00ccff, #0099ff);
  box-shadow: 0 0 12px rgba(0, 204, 255, 0.4);
}

.operation-info {
  display: flex;
  justify-content: space-between;
  font-size: 12px;
  color: var(--color-text-tertiary);
}

.operation-speed,
.operation-eta {
  color: var(--color-accent-primary);
}
</style>
