<template>
  <div class="file-toolbar glass">
    <div class="toolbar-left">
      <button class="toolbar-btn" @click="$emit('back')" :disabled="!canGoBack" title="Back">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
          <path d="M19 12H5M12 19l-7-7 7-7"/>
        </svg>
      </button>
      
      <div class="breadcrumb">
        <button 
          v-for="(crumb, index) in breadcrumbs" 
          :key="index"
          class="breadcrumb-item"
          @click="$emit('navigate', crumb.path)"
        >
          <svg v-if="index === 0" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>
          </svg>
          <span v-else>{{ crumb.name }}</span>
          <svg v-if="index < breadcrumbs.length - 1" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <polyline points="9 18 15 12 9 6"/>
          </svg>
        </button>
      </div>
    </div>

    <div class="toolbar-center">
      <div class="search-box">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
          <circle cx="11" cy="11" r="8"/>
          <path d="m21 21-4.35-4.35"/>
        </svg>
        <input 
          type="text" 
          :value="searchQuery"
          @input="$emit('search', ($event.target as HTMLInputElement).value)"
          placeholder="Search files..."
        />
      </div>
    </div>

    <div class="toolbar-right">
      <div class="view-toggle">
        <button 
          :class="['toggle-btn', { active: viewMode === 'grid' }]"
          @click="$emit('view-mode', 'grid')"
          title="Grid view"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <rect x="3" y="3" width="7" height="7"/>
            <rect x="14" y="3" width="7" height="7"/>
            <rect x="14" y="14" width="7" height="7"/>
            <rect x="3" y="14" width="7" height="7"/>
          </svg>
        </button>
        <button 
          :class="['toggle-btn', { active: viewMode === 'list' }]"
          @click="$emit('view-mode', 'list')"
          title="List view"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <line x1="8" y1="6" x2="21" y2="6"/>
            <line x1="8" y1="12" x2="21" y2="12"/>
            <line x1="8" y1="18" x2="21" y2="18"/>
            <line x1="3" y1="6" x2="3.01" y2="6"/>
            <line x1="3" y1="12" x2="3.01" y2="12"/>
            <line x1="3" y1="18" x2="3.01" y2="18"/>
          </svg>
        </button>
      </div>

      <button class="toolbar-btn" @click="$emit('new-folder')" title="New folder">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
          <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
          <line x1="12" y1="11" x2="12" y2="17"/>
          <line x1="9" y1="14" x2="15" y2="14"/>
        </svg>
      </button>

      <button class="toolbar-btn primary" @click="$emit('upload')" title="Upload">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M17 8l-5-5-5 5M12 3v12"/>
        </svg>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  currentPath: string
  searchQuery: string
  viewMode: 'grid' | 'list'
}>()

defineEmits<{
  back: []
  navigate: [path: string]
  search: [query: string]
  'view-mode': [mode: 'grid' | 'list']
  'new-folder': []
  upload: []
}>()

const breadcrumbs = computed(() => {
  const parts = props.currentPath.split('/').filter(Boolean)
  const crumbs = [{ name: 'Home', path: '/' }]
  
  let currentPath = ''
  for (const part of parts) {
    currentPath += '/' + part
    crumbs.push({ name: part, path: currentPath })
  }
  
  return crumbs
})

const canGoBack = computed(() => props.currentPath !== '/')
</script>

<style scoped>
.file-toolbar {
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
  padding: var(--spacing-md) var(--spacing-lg);
  border-radius: var(--radius-lg);
  margin-bottom: var(--spacing-lg);
}

.toolbar-left, .toolbar-right {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}

.toolbar-center {
  flex: 1;
  display: flex;
  justify-content: center;
}

.toolbar-btn {
  width: 40px;
  height: 40px;
  background: rgba(255, 255, 255, 0.03);
  border: var(--border-subtle);
  border-radius: var(--radius-md);
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all var(--transition-fast);
  display: flex;
  align-items: center;
  justify-content: center;
}

.toolbar-btn:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.08);
  border-color: var(--color-accent-primary);
  color: var(--color-accent-primary);
  transform: translateY(-2px);
}

.toolbar-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.toolbar-btn.primary {
  background: var(--gradient-primary);
  border: none;
  color: var(--color-bg-primary);
}

.toolbar-btn.primary:hover {
  box-shadow: var(--glow-primary);
}

.breadcrumb {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
}

.breadcrumb-item {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  padding: 8px 12px;
  background: transparent;
  border: none;
  border-radius: var(--radius-sm);
  color: var(--color-text-secondary);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.breadcrumb-item:hover {
  background: rgba(255, 255, 255, 0.05);
  color: var(--color-text-primary);
}

.breadcrumb-item:last-child {
  color: var(--color-accent-primary);
  font-weight: 600;
}

.search-box {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  padding: 10px 16px;
  background: rgba(255, 255, 255, 0.03);
  border: var(--border-subtle);
  border-radius: var(--radius-md);
  width: 100%;
  max-width: 400px;
  transition: all var(--transition-base);
}

.search-box:focus-within {
  background: rgba(255, 255, 255, 0.05);
  border-color: var(--color-accent-primary);
  box-shadow: 0 0 0 3px rgba(0, 255, 136, 0.1);
}

.search-box svg {
  color: var(--color-text-tertiary);
  flex-shrink: 0;
}

.search-box input {
  flex: 1;
  background: transparent;
  border: none;
  outline: none;
  color: var(--color-text-primary);
  font-size: 14px;
  padding: 0;
}

.search-box input::placeholder {
  color: var(--color-text-tertiary);
}

.view-toggle {
  display: flex;
  gap: 2px;
  background: rgba(255, 255, 255, 0.03);
  border-radius: var(--radius-md);
  padding: 4px;
}

.toggle-btn {
  width: 36px;
  height: 36px;
  background: transparent;
  border: none;
  border-radius: var(--radius-sm);
  color: var(--color-text-tertiary);
  cursor: pointer;
  transition: all var(--transition-fast);
  display: flex;
  align-items: center;
  justify-content: center;
}

.toggle-btn:hover {
  background: rgba(255, 255, 255, 0.05);
  color: var(--color-text-primary);
}

.toggle-btn.active {
  background: var(--gradient-primary);
  color: var(--color-bg-primary);
  box-shadow: var(--shadow-sm);
}
</style>
