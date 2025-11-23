<template>
  <div class="empty-state">
    <div class="empty-icon">
      <svg width="80" height="80" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z"/>
        <polyline points="13 2 13 9 20 9"/>
      </svg>
    </div>
    <h3 class="empty-title">{{ title }}</h3>
    <p class="empty-subtitle">{{ subtitle }}</p>
    <button v-if="showUploadButton" class="btn-primary" @click="$emit('upload')">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
        <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M17 8l-5-5-5 5M12 3v12"/>
      </svg>
      <span>Upload File</span>
    </button>
  </div>
</template>

<script setup lang="ts">
withDefaults(defineProps<{
  title?: string
  subtitle?: string
  showUploadButton?: boolean
}>(), {
  title: 'No files yet',
  subtitle: 'Upload your first file to get started',
  showUploadButton: true
})

defineEmits<{
  upload: []
}>()
</script>

<style scoped>
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  min-height: 400px;
  color: var(--color-text-tertiary);
}

.empty-icon {
  margin-bottom: var(--spacing-lg);
  opacity: 0.3;
  color: var(--color-text-tertiary);
  animation: fadeIn 0.5s ease-out;
}

.empty-title {
  font-size: 24px;
  font-weight: 700;
  color: var(--color-text-primary);
  margin-bottom: var(--spacing-sm);
  animation: fadeIn 0.5s ease-out 0.1s both;
}

.empty-subtitle {
  font-size: 15px;
  color: var(--color-text-secondary);
  margin-bottom: var(--spacing-xl);
  animation: fadeIn 0.5s ease-out 0.2s both;
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
  animation: fadeIn 0.5s ease-out 0.3s both;
}

.btn-primary:hover {
  transform: translateY(-2px);
  box-shadow: var(--glow-primary), var(--shadow-md);
}
</style>
