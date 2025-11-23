<template>
  <Teleport to="body">
    <div v-if="visible" class="preview-overlay" @click="$emit('close')">
      <div class="preview-modal glass" @click.stop>
        <div class="preview-header">
          <h3>{{ fileName }}</h3>
          <button class="close-btn" @click="$emit('close')">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
              <line x1="18" y1="6" x2="6" y2="18"/>
              <line x1="6" y1="6" x2="18" y2="18"/>
            </svg>
          </button>
        </div>
        <div class="preview-content">
          <img v-if="type === 'image'" :src="data" alt="Preview" />
          <video v-else-if="type === 'video'" :src="data" controls />
          <pre v-else-if="type === 'text'">{{ data }}</pre>
          <div v-else class="no-preview">
            <span>📄</span>
            <p>No preview available</p>
            <button class="btn-primary" @click="$emit('open', file)">Open with default app</button>
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { FileItem } from '../stores/filesStore'

const props = defineProps<{
  visible: boolean
  file: FileItem | null
  data: string | null
  type: string
}>()

defineEmits<{
  close: []
  open: [file: FileItem]
}>()

const fileName = computed(() => {
  if (!props.file) return ''
  return props.file.path.split('/').pop() || props.file.path.split('\\').pop() || props.file.path
})
</script>

<style scoped>
.preview-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.95);
  backdrop-filter: blur(30px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2000;
  animation: fadeIn 0.2s ease-out;
}

.preview-modal {
  border-radius: var(--radius-xl);
  max-width: 90vw;
  max-height: 90vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  box-shadow: var(--shadow-xl);
  animation: fadeIn 0.3s ease-out 0.1s both;
}

.preview-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--spacing-lg) var(--spacing-xl);
  border-bottom: var(--border-subtle);
  background: rgba(0, 0, 0, 0.3);
}

.preview-header h3 {
  font-size: 16px;
  font-weight: 600;
  color: var(--color-text-primary);
  margin: 0;
}

.close-btn {
  width: 40px;
  height: 40px;
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

.close-btn:hover {
  background: rgba(255, 51, 102, 0.15);
  border-color: var(--color-accent-danger);
  color: var(--color-accent-danger);
  transform: rotate(90deg);
}

.preview-content {
  padding: var(--spacing-xl);
  overflow: auto;
  max-height: calc(90vh - 80px);
  display: flex;
  align-items: center;
  justify-content: center;
}

.preview-content img {
  max-width: 100%;
  max-height: 70vh;
  object-fit: contain;
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-lg);
}

.preview-content video {
  max-width: 100%;
  max-height: 70vh;
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-lg);
}

.preview-content pre {
  background: rgba(0, 0, 0, 0.5);
  padding: var(--spacing-lg);
  border-radius: var(--radius-lg);
  color: var(--color-accent-primary);
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', 'Monaco', monospace;
  font-size: 13px;
  line-height: 1.8;
  overflow-x: auto;
  max-height: 70vh;
  border: var(--border-subtle);
  width: 100%;
}

.no-preview {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 80px;
  color: var(--color-text-tertiary);
}

.no-preview span {
  font-size: 64px;
  margin-bottom: var(--spacing-lg);
  opacity: 0.3;
}

.no-preview p {
  margin-bottom: var(--spacing-xl);
  font-size: 16px;
  color: var(--color-text-secondary);
}

.btn-primary {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  padding: 12px 24px;
  background: var(--gradient-primary);
  border: none;
  border-radius: var(--radius-md);
  color: var(--color-bg-primary);
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all var(--transition-base);
  box-shadow: var(--shadow-sm);
}

.btn-primary:hover {
  transform: translateY(-2px);
  box-shadow: var(--glow-primary), var(--shadow-md);
}
</style>
