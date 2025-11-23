<template>
  <Teleport to="body">
    <div 
      v-if="visible"
      class="context-menu glass"
      :style="{ top: position.y + 'px', left: position.x + 'px' }"
      @click="$emit('close')"
    >
      <button 
        v-for="item in items" 
        :key="item.id"
        :class="['menu-item', { danger: item.danger, disabled: item.disabled }]"
        @click="!item.disabled && $emit('select', item.id)"
        :disabled="item.disabled"
      >
        <div class="menu-icon" v-html="item.icon"></div>
        <span>{{ item.label }}</span>
        <span v-if="item.shortcut" class="menu-shortcut">{{ item.shortcut }}</span>
      </button>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
export interface MenuItem {
  id: string
  label: string
  icon: string
  shortcut?: string
  danger?: boolean
  disabled?: boolean
}

defineProps<{
  visible: boolean
  position: { x: number; y: number }
  items: MenuItem[]
}>()

defineEmits<{
  close: []
  select: [id: string]
}>()
</script>

<style scoped>
.context-menu {
  position: fixed;
  min-width: 220px;
  padding: var(--spacing-sm);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-xl);
  z-index: 9999;
  animation: fadeIn 0.15s ease-out;
}

.menu-item {
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
  width: 100%;
  padding: 10px 14px;
  background: transparent;
  border: none;
  border-radius: var(--radius-md);
  color: var(--color-text-primary);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition-fast);
  text-align: left;
}

.menu-item:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.08);
  transform: translateX(4px);
}

.menu-item.danger:hover:not(:disabled) {
  background: rgba(255, 51, 102, 0.15);
  color: var(--color-accent-danger);
}

.menu-item:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.menu-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  color: var(--color-text-secondary);
}

.menu-item:hover:not(:disabled) .menu-icon {
  color: var(--color-accent-primary);
}

.menu-item.danger:hover:not(:disabled) .menu-icon {
  color: var(--color-accent-danger);
}

.menu-shortcut {
  margin-left: auto;
  font-size: 12px;
  color: var(--color-text-tertiary);
  font-weight: 600;
}
</style>
