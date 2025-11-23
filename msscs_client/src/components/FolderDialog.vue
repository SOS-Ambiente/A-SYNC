<template>
  <Teleport to="body">
    <div v-if="visible" class="modal-overlay" @click="$emit('close')">
      <div class="modal-card glass" @click.stop>
        <div class="modal-header">
          <h2>{{ title }}</h2>
          <button class="close-btn" @click="$emit('close')">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
              <line x1="18" y1="6" x2="6" y2="18"/>
              <line x1="6" y1="6" x2="18" y2="18"/>
            </svg>
          </button>
        </div>
        <div class="modal-body">
          <label>Folder Name</label>
          <input 
            ref="inputRef"
            v-model="folderName" 
            type="text" 
            placeholder="Enter folder name"
            @keyup.enter="handleSubmit"
            @keyup.esc="$emit('close')"
          />
          <p v-if="error" class="error-message">{{ error }}</p>
        </div>
        <div class="modal-actions">
          <button class="btn-secondary" @click="$emit('close')">Cancel</button>
          <button class="btn-primary" @click="handleSubmit" :disabled="!folderName.trim()">
            Create Folder
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'

const props = defineProps<{
  visible: boolean
  title?: string
}>()

const emit = defineEmits<{
  close: []
  submit: [name: string]
}>()

const folderName = ref('')
const error = ref('')
const inputRef = ref<HTMLInputElement>()

watch(() => props.visible, async (visible) => {
  if (visible) {
    folderName.value = ''
    error.value = ''
    await nextTick()
    inputRef.value?.focus()
  }
})

const handleSubmit = () => {
  const name = folderName.value.trim()
  if (!name) {
    error.value = 'Folder name cannot be empty'
    return
  }
  if (name.includes('/') || name.includes('\\')) {
    error.value = 'Folder name cannot contain / or \\'
    return
  }
  emit('submit', name)
  emit('close')
}
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.8);
  backdrop-filter: blur(20px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  animation: fadeIn 0.2s ease-out;
}

.modal-card {
  border-radius: var(--radius-xl);
  min-width: 480px;
  box-shadow: var(--shadow-xl);
  animation: fadeIn 0.3s ease-out 0.1s both;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--spacing-xl);
  border-bottom: var(--border-subtle);
}

.modal-header h2 {
  font-size: 20px;
  font-weight: 700;
  color: var(--color-text-primary);
  margin: 0;
}

.close-btn {
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

.close-btn:hover {
  background: rgba(255, 51, 102, 0.15);
  border-color: var(--color-accent-danger);
  color: var(--color-accent-danger);
  transform: rotate(90deg);
}

.modal-body {
  padding: var(--spacing-xl);
}

.modal-body label {
  display: block;
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-secondary);
  text-transform: uppercase;
  letter-spacing: 1px;
  margin-bottom: var(--spacing-sm);
}

.modal-body input {
  width: 100%;
}

.error-message {
  margin-top: var(--spacing-sm);
  font-size: 13px;
  color: var(--color-accent-danger);
}

.modal-actions {
  display: flex;
  gap: var(--spacing-sm);
  justify-content: flex-end;
  padding: var(--spacing-xl);
  border-top: var(--border-subtle);
}

.btn-primary, .btn-secondary {
  padding: 12px 24px;
  border-radius: var(--radius-md);
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all var(--transition-base);
}

.btn-primary {
  background: var(--gradient-primary);
  border: none;
  color: var(--color-bg-primary);
  box-shadow: var(--shadow-sm);
}

.btn-primary:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: var(--glow-primary), var(--shadow-md);
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-secondary {
  background: rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(10px);
  border: var(--border-subtle);
  color: var(--color-text-primary);
}

.btn-secondary:hover {
  background: rgba(255, 255, 255, 0.08);
  border-color: rgba(255, 255, 255, 0.15);
}
</style>
