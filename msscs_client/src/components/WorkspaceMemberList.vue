<template>
  <div class="member-list">
    <div class="list-header">
      <h4>👥 Members ({{ members.length }})</h4>
      <button class="btn-secondary" @click="$emit('invite')">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
          <line x1="12" y1="5" x2="12" y2="19"/>
          <line x1="5" y1="12" x2="19" y2="12"/>
        </svg>
        <span>Invite</span>
      </button>
    </div>
    
    <div class="members-grid">
      <div v-for="member in members" :key="member.user_id" class="member-card glass">
        <div class="member-avatar">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/>
            <circle cx="12" cy="7" r="4"/>
          </svg>
        </div>
        <div class="member-info">
          <div class="member-email">{{ member.email }}</div>
          <div class="member-meta">
            <span class="permission-badge" :class="member.permission.toLowerCase()">
              {{ member.permission }}
            </span>
            <span class="member-status" :class="{ active: isActive(member) }">
              {{ isActive(member) ? 'Active' : 'Offline' }}
            </span>
          </div>
        </div>
        <button v-if="canManage" class="member-action" @click="$emit('manage', member)">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="1"/>
            <circle cx="12" cy="5" r="1"/>
            <circle cx="12" cy="19" r="1"/>
          </svg>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { WorkspaceMember } from '../stores/workspaceStore'

defineProps<{
  members: WorkspaceMember[]
  canManage?: boolean
}>()

defineEmits<{
  invite: []
  manage: [member: WorkspaceMember]
}>()

const isActive = (member: WorkspaceMember): boolean => {
  const fiveMinutesAgo = Date.now() - 5 * 60 * 1000
  return member.last_active > fiveMinutesAgo
}
</script>

<style scoped>
.member-list {
  margin-bottom: var(--spacing-xl);
}

.list-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--spacing-md);
}

.list-header h4 {
  font-size: 16px;
  font-weight: 700;
  color: var(--color-text-primary);
}

.btn-secondary {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  padding: 8px 16px;
  background: rgba(255, 255, 255, 0.05);
  border: var(--border-subtle);
  border-radius: var(--radius-md);
  color: var(--color-text-primary);
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-secondary:hover {
  background: rgba(255, 255, 255, 0.08);
  border-color: var(--color-accent-primary);
  transform: translateY(-2px);
}

.members-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: var(--spacing-md);
}

.member-card {
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
  padding: var(--spacing-md);
  border-radius: var(--radius-lg);
  transition: all var(--transition-base);
}

.member-card:hover {
  transform: translateY(-2px);
  box-shadow: var(--shadow-md);
}

.member-avatar {
  width: 48px;
  height: 48px;
  background: rgba(0, 255, 136, 0.1);
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-accent-primary);
  flex-shrink: 0;
}

.member-info {
  flex: 1;
  min-width: 0;
}

.member-email {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-bottom: 4px;
}

.member-meta {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}

.permission-badge {
  padding: 2px 8px;
  border-radius: var(--radius-full);
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.permission-badge.owner {
  background: rgba(255, 51, 102, 0.15);
  color: var(--color-accent-danger);
  border: 1px solid rgba(255, 51, 102, 0.3);
}

.permission-badge.admin {
  background: rgba(255, 170, 0, 0.15);
  color: var(--color-accent-warning);
  border: 1px solid rgba(255, 170, 0, 0.3);
}

.permission-badge.editor {
  background: rgba(0, 204, 255, 0.15);
  color: var(--color-accent-secondary);
  border: 1px solid rgba(0, 204, 255, 0.3);
}

.permission-badge.viewer {
  background: rgba(255, 255, 255, 0.05);
  color: var(--color-text-tertiary);
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.member-status {
  font-size: 11px;
  color: var(--color-text-tertiary);
}

.member-status.active {
  color: var(--color-accent-primary);
}

.member-action {
  width: 32px;
  height: 32px;
  background: rgba(255, 255, 255, 0.03);
  border: var(--border-subtle);
  border-radius: var(--radius-sm);
  color: var(--color-text-tertiary);
  cursor: pointer;
  transition: all var(--transition-fast);
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
}

.member-card:hover .member-action {
  opacity: 1;
}

.member-action:hover {
  background: rgba(255, 255, 255, 0.08);
  color: var(--color-text-primary);
}
</style>
