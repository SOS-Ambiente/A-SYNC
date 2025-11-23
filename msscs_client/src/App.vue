<template>
  <div class="app">
    <!-- Initialization Overlay -->
    <InitializationOverlay />
    
    <!-- Titlebar -->
    <div class="titlebar" data-tauri-drag-region>
      <div class="titlebar-left">
        <div class="app-icon">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none">
            <path d="M12 2L2 7L12 12L22 7L12 2Z" fill="url(#gradient1)" />
            <path d="M2 17L12 22L22 17V12L12 17L2 12V17Z" fill="url(#gradient2)" opacity="0.7" />
            <defs>
              <linearGradient id="gradient1" x1="2" y1="2" x2="22" y2="12">
                <stop offset="0%" stop-color="#00ff88" />
                <stop offset="100%" stop-color="#00ccff" />
              </linearGradient>
              <linearGradient id="gradient2" x1="2" y1="12" x2="22" y2="22">
                <stop offset="0%" stop-color="#00ccff" />
                <stop offset="100%" stop-color="#ff00ff" />
              </linearGradient>
            </defs>
          </svg>
        </div>
        <span class="app-title">MSSCS</span>
        <span class="app-version">v4.0</span>
      </div>
      <div class="titlebar-right">
        <button class="titlebar-btn" @click="minimizeWindow" title="Minimize">
          <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
            <line x1="0" y1="6" x2="12" y2="6" stroke="currentColor" stroke-width="1.5" />
          </svg>
        </button>
        <button class="titlebar-btn" @click="maximizeWindow" title="Maximize">
          <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
            <rect x="1" y="1" width="10" height="10" stroke="currentColor" stroke-width="1.5" fill="none" />
          </svg>
        </button>
        <button class="titlebar-btn close" @click="closeWindow" title="Close">
          <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
            <path d="M1 1L11 11M11 1L1 11" stroke="currentColor" stroke-width="1.5" />
          </svg>
        </button>
      </div>
    </div>

    <!-- Main Content -->
    <div class="main-container">
      <!-- Sidebar -->
      <aside class="sidebar">
        <nav class="nav">
          <button 
            v-for="item in navItems" 
            :key="item.id"
            :class="['nav-item', { active: activeView === item.id }]"
            @click="activeView = item.id"
          >
            <div class="nav-icon" v-html="item.icon"></div>
            <span class="nav-label">{{ item.label }}</span>
            <div v-if="activeView === item.id" class="nav-indicator"></div>
          </button>
        </nav>

        <!-- Node Status -->
        <div class="node-status glass">
          <div class="status-indicator" :class="nodeStore.status">
            <div class="status-pulse"></div>
          </div>
          <div class="status-info">
            <div class="status-label">Node Status</div>
            <div class="status-value">{{ nodeStore.status }}</div>
          </div>
        </div>

        <!-- Storage Info -->
        <div class="storage-info glass">
          <div class="storage-header">
            <div class="storage-label">Storage</div>
            <div class="storage-percent">{{ Math.round(storagePercent) }}%</div>
          </div>
          <div class="storage-bar">
            <div class="storage-fill" :style="{ width: storagePercent + '%' }"></div>
            <div class="storage-glow" :style="{ width: storagePercent + '%' }"></div>
          </div>
          <div class="storage-text">
            <span>{{ formatBytes(nodeStore.storageUsed) }}</span>
            <span class="storage-divider">/</span>
            <span>{{ formatBytes(nodeStore.storageTotal) }}</span>
          </div>
        </div>
      </aside>

      <!-- Content Area -->
      <main class="content">
        <transition name="fade" mode="out-in">
          <DashboardView v-if="activeView === 'dashboard'" @navigate="(view) => activeView = view" />
          <FilesView v-else-if="activeView === 'files'" />
          <WorkspacePanel v-else-if="activeView === 'workspaces'" />
          <SyncView v-else-if="activeView === 'sync'" />
          <PeersView v-else-if="activeView === 'peers'" />
          <SettingsView v-else-if="activeView === 'settings'" />
        </transition>
      </main>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { appWindow } from '@tauri-apps/api/window'
import { useNodeStore } from './stores/nodeStore'
import DashboardView from './components/DashboardView.vue'
import FilesView from './components/FilesView.vue'
import SyncView from './components/SyncView.vue'
import PeersView from './components/PeersView.vue'
import SettingsView from './components/SettingsView.vue'
import WorkspacePanel from './components/WorkspacePanel.vue'
import InitializationOverlay from './components/InitializationOverlay.vue'

const nodeStore = useNodeStore()
const activeView = ref('dashboard')

// Keyboard shortcuts
const handleKeyboard = (e: KeyboardEvent) => {
  if (e.ctrlKey || e.metaKey) {
    switch(e.key) {
      case 'd':
        e.preventDefault()
        activeView.value = 'dashboard'
        break
      case '1':
        e.preventDefault()
        activeView.value = 'files'
        break
      case '2':
        e.preventDefault()
        activeView.value = 'workspaces'
        break
      case '3':
        e.preventDefault()
        activeView.value = 'sync'
        break
      case '4':
        e.preventDefault()
        activeView.value = 'peers'
        break
      case ',':
        e.preventDefault()
        activeView.value = 'settings'
        break
    }
  }
}

const navItems = [
  { 
    id: 'dashboard', 
    icon: '<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/></svg>', 
    label: 'Dashboard',
    shortcut: 'Ctrl+D'
  },
  { 
    id: 'files', 
    icon: '<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z"/><polyline points="13 2 13 9 20 9"/></svg>', 
    label: 'Files',
    shortcut: 'Ctrl+1'
  },
  { 
    id: 'workspaces', 
    icon: '<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M23 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/></svg>', 
    label: 'Workspaces',
    shortcut: 'Ctrl+2'
  },
  { 
    id: 'sync', 
    icon: '<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21.5 2v6h-6M2.5 22v-6h6M2 11.5a10 10 0 0 1 18.8-4.3M22 12.5a10 10 0 0 1-18.8 4.2"/></svg>', 
    label: 'Sync',
    shortcut: 'Ctrl+3'
  },
  { 
    id: 'peers', 
    icon: '<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><path d="M12 2a14.5 14.5 0 0 0 0 20 14.5 14.5 0 0 0 0-20M2 12h20"/></svg>', 
    label: 'Peers',
    shortcut: 'Ctrl+4'
  },
  { 
    id: 'settings', 
    icon: '<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"/><path d="M12 1v6m0 6v6M5.64 5.64l4.24 4.24m4.24 4.24l4.24 4.24M1 12h6m6 0h6M5.64 18.36l4.24-4.24m4.24-4.24l4.24-4.24"/></svg>', 
    label: 'Settings',
    shortcut: 'Ctrl+,'
  },
]

const storagePercent = computed(() => {
  if (nodeStore.storageTotal === 0) return 0
  const percent = (nodeStore.storageUsed / nodeStore.storageTotal) * 100
  return Math.min(percent, 100) // Cap at 100%
})

const formatBytes = (bytes: number): string => {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i]
}

const minimizeWindow = () => appWindow.minimize()
const maximizeWindow = () => appWindow.toggleMaximize()
const closeWindow = () => appWindow.close()

// Initialize node on mount
onMounted(async () => {
  console.log('App mounted')
  // Node initialization is now handled by InitializationOverlay
  
  // Add keyboard shortcuts
  window.addEventListener('keydown', handleKeyboard)
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyboard)
})
</script>

<style scoped>
.app {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: var(--color-bg-primary);
  color: var(--color-text-primary);
}

/* Titlebar - Ultra Minimal */
.titlebar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  height: 40px;
  background: rgba(0, 0, 0, 0.8);
  backdrop-filter: blur(20px);
  border-bottom: var(--border-subtle);
  user-select: none;
  z-index: 100;
}

.titlebar-left {
  display: flex;
  align-items: center;
  gap: 12px;
  padding-left: 16px;
}

.app-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  animation: fadeIn 0.5s ease-out;
}

.app-icon svg {
  filter: drop-shadow(var(--glow-primary));
}

.app-title {
  font-size: 13px;
  font-weight: 700;
  letter-spacing: 1px;
  background: var(--gradient-primary);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.app-version {
  font-size: 10px;
  font-weight: 600;
  color: var(--color-text-tertiary);
  padding: 2px 8px;
  background: rgba(255, 255, 255, 0.05);
  border-radius: var(--radius-full);
}

.titlebar-right {
  display: flex;
}

.titlebar-btn {
  width: 48px;
  height: 40px;
  background: transparent;
  border: none;
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all var(--transition-fast);
  display: flex;
  align-items: center;
  justify-content: center;
}

.titlebar-btn:hover {
  background: rgba(255, 255, 255, 0.05);
  color: var(--color-text-primary);
}

.titlebar-btn.close:hover {
  background: var(--color-accent-danger);
  color: white;
}

/* Main Container */
.main-container {
  display: flex;
  flex: 1;
  overflow: hidden;
}

/* Sidebar - Glassmorphism */
.sidebar {
  width: 260px;
  background: rgba(10, 10, 10, 0.6);
  backdrop-filter: blur(20px);
  border-right: var(--border-subtle);
  display: flex;
  flex-direction: column;
  padding: var(--spacing-lg);
  gap: var(--spacing-lg);
  animation: slideIn 0.4s ease-out;
}

/* Navigation */
.nav {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.nav-item {
  position: relative;
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 14px 16px;
  background: transparent;
  border: none;
  border-radius: var(--radius-md);
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all var(--transition-base);
  text-align: left;
  font-weight: 500;
  overflow: hidden;
}

.nav-item::before {
  content: '';
  position: absolute;
  inset: 0;
  background: var(--gradient-primary);
  opacity: 0;
  transition: opacity var(--transition-base);
  border-radius: var(--radius-md);
}

.nav-item:hover {
  color: var(--color-text-primary);
  transform: translateX(4px);
}

.nav-item:hover::before {
  opacity: 0.05;
}

.nav-item.active {
  color: var(--color-accent-primary);
  background: rgba(0, 255, 136, 0.08);
}

.nav-item.active::before {
  opacity: 0.1;
}

.nav-item.active .nav-icon {
  filter: drop-shadow(0 0 8px rgba(0, 255, 136, 0.5));
}

.nav-icon {
  position: relative;
  z-index: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all var(--transition-base);
}

.nav-label {
  position: relative;
  z-index: 1;
  font-size: 14px;
  font-weight: 600;
  flex: 1;
}

.nav-indicator {
  position: absolute;
  right: 12px;
  width: 4px;
  height: 20px;
  background: var(--gradient-primary);
  border-radius: var(--radius-full);
  box-shadow: var(--glow-primary);
}

/* Node Status - Enhanced */
.node-status {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 16px;
  border-radius: var(--radius-lg);
  transition: all var(--transition-base);
  animation: fadeIn 0.6s ease-out 0.2s both;
}

.node-status:hover {
  transform: translateY(-2px);
  box-shadow: var(--shadow-md);
}

.status-indicator {
  position: relative;
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: var(--color-text-tertiary);
  transition: all var(--transition-base);
}

.status-pulse {
  position: absolute;
  inset: -4px;
  border-radius: 50%;
  border: 2px solid currentColor;
  opacity: 0;
}

.status-indicator.online {
  background: var(--color-accent-primary);
  box-shadow: 0 0 16px rgba(0, 255, 136, 0.6);
}

.status-indicator.online .status-pulse {
  animation: pulse 2s ease-in-out infinite;
  border-color: var(--color-accent-primary);
}

.status-indicator.syncing {
  background: var(--color-accent-warning);
  box-shadow: 0 0 16px rgba(255, 170, 0, 0.6);
  animation: pulse 1s ease-in-out infinite;
}

.status-info {
  flex: 1;
}

.status-label {
  font-size: 10px;
  color: var(--color-text-tertiary);
  text-transform: uppercase;
  letter-spacing: 1px;
  font-weight: 600;
  margin-bottom: 4px;
}

.status-value {
  font-size: 14px;
  color: var(--color-text-primary);
  font-weight: 700;
  text-transform: capitalize;
}

/* Storage Info - Modern Design */
.storage-info {
  padding: 16px;
  border-radius: var(--radius-lg);
  margin-top: auto;
  transition: all var(--transition-base);
  animation: fadeIn 0.6s ease-out 0.3s both;
}

.storage-info:hover {
  transform: translateY(-2px);
  box-shadow: var(--shadow-md);
}

.storage-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.storage-label {
  font-size: 10px;
  color: var(--color-text-tertiary);
  text-transform: uppercase;
  letter-spacing: 1px;
  font-weight: 600;
}

.storage-percent {
  font-size: 13px;
  font-weight: 700;
  background: var(--gradient-primary);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.storage-bar {
  position: relative;
  height: 6px;
  background: rgba(255, 255, 255, 0.05);
  border-radius: var(--radius-full);
  overflow: visible;
  margin-bottom: 10px;
}

.storage-fill {
  position: absolute;
  height: 100%;
  background: var(--gradient-primary);
  border-radius: var(--radius-full);
  transition: width 0.5s cubic-bezier(0.4, 0, 0.2, 1);
}

.storage-glow {
  position: absolute;
  height: 100%;
  background: var(--gradient-primary);
  border-radius: var(--radius-full);
  filter: blur(8px);
  opacity: 0.5;
  transition: width 0.5s cubic-bezier(0.4, 0, 0.2, 1);
}

.storage-text {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 11px;
  color: var(--color-text-secondary);
  font-weight: 500;
}

.storage-divider {
  color: var(--color-text-tertiary);
  margin: 0 4px;
}

/* Content Area */
.content {
  flex: 1;
  overflow: auto;
  background: var(--color-bg-primary);
  position: relative;
}

/* Fade Transition */
.fade-enter-active,
.fade-leave-active {
  transition: all var(--transition-base);
}

.fade-enter-from {
  opacity: 0;
  transform: translateY(10px);
}

.fade-leave-to {
  opacity: 0;
  transform: translateY(-10px);
}
</style>
