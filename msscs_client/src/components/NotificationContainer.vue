<template>
  <Teleport to="body">
    <div class="notification-container">
      <TransitionGroup name="notification">
        <div
          v-for="notification in notifications"
          :key="notification.id"
          :class="['notification', notification.type]"
          @click="removeNotification(notification.id)"
        >
          <div class="notification-icon">
            <svg v-if="notification.type === 'success'" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
              <polyline points="20 6 9 17 4 12"/>
            </svg>
            <svg v-else-if="notification.type === 'error'" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
              <circle cx="12" cy="12" r="10"/>
              <line x1="15" y1="9" x2="9" y2="15"/>
              <line x1="9" y1="9" x2="15" y2="15"/>
            </svg>
            <svg v-else-if="notification.type === 'warning'" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
              <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/>
              <line x1="12" y1="9" x2="12" y2="13"/>
              <line x1="12" y1="17" x2="12.01" y2="17"/>
            </svg>
            <svg v-else width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
              <circle cx="12" cy="12" r="10"/>
              <line x1="12" y1="16" x2="12" y2="12"/>
              <line x1="12" y1="8" x2="12.01" y2="8"/>
            </svg>
          </div>
          <div class="notification-content">
            <div class="notification-title">{{ notification.title }}</div>
            <div class="notification-message">{{ notification.message }}</div>
          </div>
          <button class="notification-close" @click.stop="removeNotification(notification.id)">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
              <line x1="18" y1="6" x2="6" y2="18"/>
              <line x1="6" y1="6" x2="18" y2="18"/>
            </svg>
          </button>
        </div>
      </TransitionGroup>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { useNotifications } from '../composables/useNotifications'

const { notifications, removeNotification } = useNotifications()
</script>

<style scoped>
.notification-container {
  position: fixed;
  top: var(--spacing-xl);
  right: var(--spacing-xl);
  z-index: 9999;
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
  max-width: 400px;
}

.notification {
  display: flex;
  align-items: flex-start;
  gap: var(--spacing-md);
  padding: var(--spacing-md) var(--spacing-lg);
  background: rgba(0, 0, 0, 0.95);
  backdrop-filter: blur(20px);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-xl);
  cursor: pointer;
  transition: all var(--transition-base);
  border: var(--border-medium);
}

.notification:hover {
  transform: translateX(-4px);
  box-shadow: var(--shadow-xl), 0 0 20px rgba(0, 0, 0, 0.5);
}

.notification.success {
  border-color: var(--color-accent-primary);
  box-shadow: var(--shadow-xl), var(--glow-primary);
}

.notification.error {
  border-color: var(--color-accent-danger);
  box-shadow: var(--shadow-xl), 0 0 20px rgba(255, 51, 102, 0.3);
}

.notification.warning {
  border-color: var(--color-accent-warning);
  box-shadow: var(--shadow-xl), 0 0 20px rgba(255, 170, 0, 0.3);
}

.notification.info {
  border-color: var(--color-accent-secondary);
  box-shadow: var(--shadow-xl), var(--glow-secondary);
}

.notification-icon {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-md);
  flex-shrink: 0;
}

.notification.success .notification-icon {
  background: rgba(0, 255, 136, 0.15);
  color: var(--color-accent-primary);
}

.notification.error .notification-icon {
  background: rgba(255, 51, 102, 0.15);
  color: var(--color-accent-danger);
}

.notification.warning .notification-icon {
  background: rgba(255, 170, 0, 0.15);
  color: var(--color-accent-warning);
}

.notification.info .notification-icon {
  background: rgba(0, 204, 255, 0.15);
  color: var(--color-accent-secondary);
}

.notification-content {
  flex: 1;
  min-width: 0;
}

.notification-title {
  font-size: 14px;
  font-weight: 700;
  color: var(--color-text-primary);
  margin-bottom: 4px;
}

.notification-message {
  font-size: 13px;
  color: var(--color-text-secondary);
  line-height: 1.5;
}

.notification-close {
  width: 28px;
  height: 28px;
  background: rgba(255, 255, 255, 0.05);
  border: var(--border-subtle);
  border-radius: var(--radius-sm);
  color: var(--color-text-tertiary);
  cursor: pointer;
  transition: all var(--transition-fast);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.notification-close:hover {
  background: rgba(255, 255, 255, 0.1);
  color: var(--color-text-primary);
  transform: rotate(90deg);
}

/* Transition animations */
.notification-enter-active,
.notification-leave-active {
  transition: all var(--transition-base);
}

.notification-enter-from {
  opacity: 0;
  transform: translateX(100px);
}

.notification-leave-to {
  opacity: 0;
  transform: translateX(100px) scale(0.8);
}

.notification-move {
  transition: transform var(--transition-base);
}
</style>
