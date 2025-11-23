<template>
  <div class="loading-state">
    <div class="spinner-container">
      <div class="spinner"></div>
      <div class="spinner-glow"></div>
    </div>
    <p class="loading-text">{{ message }}</p>
  </div>
</template>

<script setup lang="ts">
withDefaults(defineProps<{
  message?: string
}>(), {
  message: 'Loading...'
})
</script>

<style scoped>
.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  min-height: 400px;
  color: var(--color-text-tertiary);
}

.spinner-container {
  position: relative;
  width: 64px;
  height: 64px;
  margin-bottom: var(--spacing-lg);
}

.spinner {
  position: absolute;
  inset: 0;
  border: 3px solid transparent;
  border-top-color: var(--color-accent-primary);
  border-right-color: var(--color-accent-secondary);
  border-radius: 50%;
  animation: spin 1s cubic-bezier(0.68, -0.55, 0.265, 1.55) infinite;
}

.spinner-glow {
  position: absolute;
  inset: -4px;
  border: 3px solid transparent;
  border-top-color: var(--color-accent-primary);
  border-right-color: var(--color-accent-secondary);
  border-radius: 50%;
  filter: blur(8px);
  opacity: 0.5;
  animation: spin 1s cubic-bezier(0.68, -0.55, 0.265, 1.55) infinite;
}

.loading-text {
  font-size: 15px;
  font-weight: 500;
  color: var(--color-text-secondary);
}
</style>
