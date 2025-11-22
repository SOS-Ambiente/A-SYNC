<template>
  <div class="mobile-app">
    <!-- Status Bar -->
    <div class="status-bar">
      <div class="status-left">
        <span class="status-icon" :class="statusClass">●</span>
        <span class="status-text">{{ statusText }}</span>
      </div>
      <div class="status-right">
        <span class="peer-count">{{ peerCount }} peers</span>
      </div>
    </div>

    <!-- Main Content -->
    <div class="content">
      <router-view />
    </div>

    <!-- Toast Notifications -->
    <ToastContainer />

    <!-- Bottom Navigation -->
    <nav class="bottom-nav">
      <router-link to="/files" class="nav-item">
        <span class="nav-icon">📁</span>
        <span class="nav-label">Files</span>
      </router-link>
      <router-link to="/nodes" class="nav-item">
        <span class="nav-icon">🌐</span>
        <span class="nav-label">Nodes</span>
      </router-link>
      <router-link to="/security" class="nav-item">
        <span class="nav-icon">🔐</span>
        <span class="nav-label">Security</span>
      </router-link>
      <router-link to="/settings" class="nav-item">
        <span class="nav-icon">⚙️</span>
        <span class="nav-label">Settings</span>
      </router-link>
    </nav>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useNodeStore } from './stores/nodeStore'
import { useNetwork } from './composables/useNetwork'
import { useToast } from './composables/useToast'
import ToastContainer from './components/ToastContainer.vue'

const nodeStore = useNodeStore()
const { isOnline } = useNetwork()
const toast = useToast()

const statusClass = computed(() => {
  return {
    'status-online': nodeStore.status === 'online',
    'status-syncing': nodeStore.status === 'syncing',
    'status-offline': nodeStore.status === 'offline',
  }
})

const statusText = computed(() => {
  return nodeStore.status.charAt(0).toUpperCase() + nodeStore.status.slice(1)
})

const peerCount = computed(() => nodeStore.peerCount)

onMounted(async () => {
  try {
    await nodeStore.initialize()
    toast.success('Node initialized successfully')
  } catch (error) {
    toast.error('Failed to initialize node')
    console.error('Initialization error:', error)
  }
  
  // Watch for network changes
  let wasOnline = isOnline.value
  setInterval(() => {
    if (isOnline.value !== wasOnline) {
      if (isOnline.value) {
        toast.success('Back online')
      } else {
        toast.warning('You are offline')
      }
      wasOnline = isOnline.value
    }
  }, 1000)
})
</script>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
  background: #000;
  color: #fff;
  overflow: hidden;
}

.mobile-app {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: #000;
}

.status-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background: #111;
  border-bottom: 1px solid #222;
}

.status-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.status-icon {
  font-size: 12px;
}

.status-online {
  color: #00ff88;
}

.status-syncing {
  color: #ffaa00;
  animation: pulse 1.5s ease-in-out infinite;
}

.status-offline {
  color: #ff4444;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

.status-text {
  font-size: 14px;
  font-weight: 500;
}

.status-right {
  font-size: 12px;
  color: #888;
}

.peer-count {
  padding: 4px 8px;
  background: #222;
  border-radius: 12px;
}

.content {
  flex: 1;
  overflow-y: auto;
  -webkit-overflow-scrolling: touch;
}

.bottom-nav {
  display: flex;
  justify-content: space-around;
  background: #111;
  border-top: 1px solid #222;
  padding: 8px 0;
  padding-bottom: max(8px, env(safe-area-inset-bottom));
}

.nav-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  padding: 8px 16px;
  text-decoration: none;
  color: #888;
  transition: color 0.2s;
}

.nav-item.router-link-active {
  color: #00ff88;
}

.nav-icon {
  font-size: 24px;
}

.nav-label {
  font-size: 11px;
  font-weight: 500;
}
</style>
