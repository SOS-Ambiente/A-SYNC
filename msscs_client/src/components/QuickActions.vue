<template>
  <div class="quick-actions">
    <Transition name="menu">
      <div v-if="isOpen" class="actions-menu">
        <button 
          v-for="action in actions" 
          :key="action.id"
          class="action-item glass"
          @click="handleAction(action.id)"
        >
          <div class="action-icon" v-html="action.icon"></div>
          <span class="action-label">{{ action.label }}</span>
        </button>
      </div>
    </Transition>
    
    <button 
      class="fab glass"
      :class="{ active: isOpen }"
      @click="isOpen = !isOpen"
      title="Quick actions"
    >
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
        <line x1="12" y1="5" x2="12" y2="19"/>
        <line x1="5" y1="12" x2="19" y2="12"/>
      </svg>
    </button>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'

const isOpen = ref(false)

const actions = [
  {
    id: 'upload',
    label: 'Upload File',
    icon: '<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M17 8l-5-5-5 5M12 3v12"/></svg>'
  },
  {
    id: 'folder',
    label: 'New Folder',
    icon: '<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/><line x1="12" y1="11" x2="12" y2="17"/><line x1="9" y1="14" x2="15" y2="14"/></svg>'
  },
  {
    id: 'workspace',
    label: 'New Workspace',
    icon: '<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M23 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/></svg>'
  },
  {
    id: 'share',
    label: 'Share File',
    icon: '<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="18" cy="5" r="3"/><circle cx="6" cy="12" r="3"/><circle cx="18" cy="19" r="3"/><line x1="8.59" y1="13.51" x2="15.42" y2="17.49"/><line x1="15.41" y1="6.51" x2="8.59" y2="10.49"/></svg>'
  }
]

const emit = defineEmits<{
  action: [id: string]
}>()

const handleAction = (id: string) => {
  emit('action', id)
  isOpen.value = false
}
</script>

<style scoped>
.quick-actions {
  position: fixed;
  bottom: var(--spacing-xl);
  right: var(--spacing-xl);
  z-index: 100;
}

.actions-menu {
  position: absolute;
  bottom: 80px;
  right: 0;
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
  min-width: 200px;
}

.action-item {
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
  padding: var(--spacing-md) var(--spacing-lg);
  border-radius: var(--radius-lg);
  cursor: pointer;
  transition: all var(--transition-fast);
  border: var(--border-subtle);
}

.action-item:hover {
  background: rgba(255, 255, 255, 0.08);
  border-color: var(--color-accent-primary);
  transform: translateX(-4px);
}

.action-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  background: rgba(0, 255, 136, 0.1);
  border-radius: var(--radius-md);
  color: var(--color-accent-primary);
}

.action-label {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text-primary);
}

.fab {
  width: 64px;
  height: 64px;
  border-radius: 50%;
  background: var(--gradient-primary);
  border: none;
  color: var(--color-bg-primary);
  cursor: pointer;
  transition: all var(--transition-base);
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: var(--shadow-lg), var(--glow-primary);
}

.fab:hover {
  transform: scale(1.1) rotate(90deg);
  box-shadow: var(--shadow-xl), var(--glow-primary);
}

.fab.active {
  transform: rotate(45deg);
  background: var(--gradient-danger);
}

.fab.active:hover {
  transform: scale(1.1) rotate(45deg);
}

.menu-enter-active,
.menu-leave-active {
  transition: all var(--transition-base);
}

.menu-enter-from {
  opacity: 0;
  transform: translateY(20px) scale(0.9);
}

.menu-leave-to {
  opacity: 0;
  transform: translateY(20px) scale(0.9);
}
</style>
