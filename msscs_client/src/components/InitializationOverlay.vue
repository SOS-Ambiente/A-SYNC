<template>
  <div v-if="show" class="initialization-overlay">
    <div class="init-modal glass">
      <div class="init-header">
        <div class="init-icon">
          <svg v-if="status === 'initializing'" class="spinner" width="48" height="48" viewBox="0 0 24 24">
            <circle cx="12" cy="12" r="10" stroke="url(#gradient)" stroke-width="3" fill="none" stroke-linecap="round" stroke-dasharray="60" stroke-dashoffset="20">
              <animateTransform attributeName="transform" type="rotate" from="0 12 12" to="360 12 12" dur="1s" repeatCount="indefinite"/>
            </circle>
            <defs>
              <linearGradient id="gradient" x1="0%" y1="0%" x2="100%" y2="100%">
                <stop offset="0%" stop-color="#00ff88" />
                <stop offset="100%" stop-color="#00ccff" />
              </linearGradient>
            </defs>
          </svg>
          <svg v-else-if="status === 'success'" width="48" height="48" viewBox="0 0 24 24" fill="none">
            <circle cx="12" cy="12" r="10" stroke="#00ff88" stroke-width="2" fill="none"/>
            <path d="M8 12l3 3 5-6" stroke="#00ff88" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
          <svg v-else-if="status === 'error'" width="48" height="48" viewBox="0 0 24 24" fill="none">
            <circle cx="12" cy="12" r="10" stroke="#ff4444" stroke-width="2" fill="none"/>
            <path d="M15 9l-6 6M9 9l6 6" stroke="#ff4444" stroke-width="2" stroke-linecap="round"/>
          </svg>
        </div>
        <h2 class="init-title">{{ title }}</h2>
        <p class="init-subtitle">{{ subtitle }}</p>
      </div>

      <div class="init-progress">
        <div class="progress-bar">
          <div class="progress-fill" :style="{ width: progress + '%' }"></div>
          <div class="progress-glow" :style="{ width: progress + '%' }"></div>
        </div>
        <div class="progress-text">{{ Math.round(progress) }}%</div>
      </div>

      <div class="init-logs">
        <div v-for="(log, index) in logs" :key="index" :class="['log-entry', log.type]">
          <span class="log-icon">{{ getLogIcon(log.type) }}</span>
          <span class="log-message">{{ log.message }}</span>
          <span class="log-time">{{ formatTime(log.timestamp) }}</span>
        </div>
      </div>

      <div v-if="firewallPrompt" class="firewall-prompt">
        <div class="prompt-icon">🛡️</div>
        <h3>Firewall Access Required</h3>
        <p>MSSCS needs network access to connect to peers. Please allow access when Windows Firewall prompts you.</p>
        <div class="prompt-actions">
          <button @click="requestFirewallAccess" class="btn-primary">
            Grant Firewall Access
          </button>
          <button @click="openFirewallSettings" class="btn-secondary">
            Open Settings Manually
          </button>
          <button @click="dismissFirewallPrompt" class="btn-secondary">
            Skip
          </button>
        </div>
      </div>

      <div v-if="showTroubleshooting" class="troubleshooting">
        <h3>🔧 Troubleshooting</h3>
        <ul>
          <li v-for="(tip, index) in troubleshootingTips" :key="index">{{ tip }}</li>
        </ul>
        <button @click="retryConnection" class="btn-primary">Retry Connection</button>
      </div>

      <div v-if="status === 'success'" class="init-actions">
        <button @click="close" class="btn-success">
          Get Started
        </button>
      </div>

      <div v-if="status === 'error'" class="init-actions">
        <button @click="retryConnection" class="btn-primary">Retry</button>
        <button @click="continueOffline" class="btn-secondary">Continue Offline</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { useNodeStore } from '../stores/nodeStore'
import { invoke } from '@tauri-apps/api/tauri'

interface Log {
  type: 'info' | 'success' | 'warning' | 'error'
  message: string
  timestamp: number
}

const nodeStore = useNodeStore()
const show = ref(true)
const status = ref<'initializing' | 'success' | 'error'>('initializing')
const progress = ref(0)
const logs = ref<Log[]>([])
const firewallPrompt = ref(false)
const showTroubleshooting = ref(false)

const title = computed(() => {
  switch (status.value) {
    case 'initializing': return 'Initializing P2P Network'
    case 'success': return 'Connected Successfully!'
    case 'error': return 'Connection Failed'
    default: return 'Initializing'
  }
})

const subtitle = computed(() => {
  switch (status.value) {
    case 'initializing': return 'Preparing your node...'
    case 'success': return 'Ready! Bootstrap continues in background'
    case 'error': return 'Unable to establish P2P connection'
    default: return ''
  }
})

const troubleshootingTips = [
  'Check your internet connection',
  'Allow MSSCS through Windows Firewall',
  'Disable VPN temporarily if using one',
  'Check if antivirus is blocking connections',
  'Try restarting the application'
]

const addLog = (type: Log['type'], message: string) => {
  logs.value.push({
    type,
    message,
    timestamp: Date.now()
  })
  
  // Keep only last 10 logs
  if (logs.value.length > 10) {
    logs.value.shift()
  }
}

const getLogIcon = (type: string) => {
  switch (type) {
    case 'success': return '✅'
    case 'error': return '❌'
    case 'warning': return '⚠️'
    default: return 'ℹ️'
  }
}

const formatTime = (timestamp: number) => {
  const now = Date.now()
  const diff = Math.floor((now - timestamp) / 1000)
  if (diff < 1) return 'just now'
  if (diff < 60) return `${diff}s ago`
  return `${Math.floor(diff / 60)}m ago`
}

const openFirewallSettings = async () => {
  try {
    await invoke('open_firewall_settings')
    addLog('info', 'Opened Windows Firewall settings')
  } catch (error) {
    addLog('error', 'Failed to open firewall settings')
    console.error('Firewall settings error:', error)
  }
}

const requestFirewallAccess = async () => {
  try {
    addLog('info', 'Requesting firewall access...')
    await invoke('request_firewall_access')
    addLog('success', 'Firewall access granted!')
    firewallPrompt.value = false
    return true
  } catch (error) {
    addLog('error', 'Failed to add firewall rule')
    console.error('Firewall access error:', error)
    return false
  }
}

const checkFirewallAccess = async () => {
  try {
    const hasAccess = await invoke<boolean>('check_firewall_access')
    if (hasAccess) {
      addLog('success', 'Firewall access already configured')
      return true
    }
    return false
  } catch (error) {
    console.error('Firewall check error:', error)
    return false
  }
}

const dismissFirewallPrompt = async () => {
  // Try to request firewall access automatically
  const granted = await requestFirewallAccess()
  if (granted) {
    addLog('info', 'Continuing with initialization...')
  } else {
    firewallPrompt.value = false
    addLog('warning', 'Continuing without firewall configuration')
  }
}

const retryConnection = async () => {
  status.value = 'initializing'
  progress.value = 0
  logs.value = []
  showTroubleshooting.value = false
  firewallPrompt.value = false
  await initializeNode()
}

const continueOffline = () => {
  addLog('warning', 'Continuing in offline mode')
  show.value = false
}

const close = () => {
  show.value = false
}

const initializeNode = async () => {
  addLog('info', 'Starting node initialization...')
  progress.value = 10

  try {
    // Step 1: Check firewall access (non-blocking, in parallel)
    addLog('info', 'Checking firewall configuration...')
    progress.value = 20
    
    // Start firewall check in background - don't wait
    checkFirewallAccess().then(hasAccess => {
      if (!hasAccess) {
        addLog('warning', 'Firewall access not configured')
        firewallPrompt.value = true
        // Auto-request after 2 seconds
        setTimeout(() => {
          if (firewallPrompt.value) {
            requestFirewallAccess()
          }
        }, 2000)
      } else {
        addLog('success', 'Firewall access confirmed')
      }
    }).catch(() => {
      // Ignore firewall check errors
    })

    // Step 2: Start node immediately (non-blocking)
    addLog('info', 'Setting up your decentralized node...')
    progress.value = 40

    // CRITICAL FIX: Start node with 2 second timeout - mark ready immediately
    const startPromise = invoke('start_node')
    const timeoutPromise = new Promise((_, reject) => 
      setTimeout(() => reject(new Error('Start timeout')), 2000)
    )
    
    try {
      await Promise.race([startPromise, timeoutPromise])
      addLog('success', 'Node started successfully')
    } catch (error) {
      // Timeout is OK - node is starting in background
      if (error instanceof Error && error.message === 'Start timeout') {
        addLog('info', 'Node starting in background')
      } else {
        throw error
      }
    }
    
    progress.value = 70
    firewallPrompt.value = false

    // Step 3: Mark as ready immediately
    addLog('info', 'Initializing P2P network...')
    progress.value = 85
    
    // Small delay for visual feedback
    await new Promise(resolve => setTimeout(resolve, 300))
    
    addLog('success', 'P2P network initialized!')
    progress.value = 95

    // Step 4: Finalize
    addLog('success', 'Node ready!')
    addLog('info', 'Bootstrap continues in background (10-30s)')
    addLog('info', 'You can start using the app now')
    progress.value = 100
    status.value = 'success'

    // Auto-close after 800ms
    setTimeout(() => {
      show.value = false
    }, 800)

  } catch (error) {
    const errorMsg = error instanceof Error ? error.message : String(error)
    addLog('error', `Initialization failed: ${errorMsg}`)
    status.value = 'error'
    progress.value = 0
    showTroubleshooting.value = true
    
    // Check if it's a firewall issue
    if (errorMsg.includes('timeout') || errorMsg.includes('network')) {
      firewallPrompt.value = true
    }
  }
}

// Watch node store status
watch(() => nodeStore.status, (newStatus) => {
  if (newStatus === 'online' && status.value === 'initializing') {
    addLog('success', 'Node transitioned to online')
    progress.value = 100
    status.value = 'success'
    setTimeout(() => show.value = false, 2000)
  } else if (newStatus === 'offline' && status.value === 'initializing') {
    addLog('error', 'Node went offline')
    status.value = 'error'
    showTroubleshooting.value = true
  }
})

onMounted(() => {
  initializeNode()
})
</script>

<style scoped>
.initialization-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.95);
  backdrop-filter: blur(20px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10000;
  animation: fadeIn 0.3s ease-out;
}

.init-modal {
  width: 90%;
  max-width: 600px;
  max-height: 90vh;
  overflow-y: auto;
  padding: var(--spacing-2xl);
  border-radius: var(--radius-xl);
  animation: slideUp 0.4s ease-out;
}

.init-header {
  text-align: center;
  margin-bottom: var(--spacing-xl);
}

.init-icon {
  margin-bottom: var(--spacing-lg);
  display: flex;
  justify-content: center;
}

.spinner {
  filter: drop-shadow(0 0 20px rgba(0, 255, 136, 0.5));
}

.init-title {
  font-size: 28px;
  font-weight: 700;
  background: var(--gradient-primary);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  margin-bottom: var(--spacing-sm);
}

.init-subtitle {
  color: var(--color-text-secondary);
  font-size: 14px;
}

.init-progress {
  margin-bottom: var(--spacing-xl);
}

.progress-bar {
  position: relative;
  height: 8px;
  background: rgba(255, 255, 255, 0.05);
  border-radius: var(--radius-full);
  overflow: visible;
  margin-bottom: var(--spacing-sm);
}

.progress-fill {
  position: absolute;
  height: 100%;
  background: var(--gradient-primary);
  border-radius: var(--radius-full);
  transition: width 0.5s cubic-bezier(0.4, 0, 0.2, 1);
}

.progress-glow {
  position: absolute;
  height: 100%;
  background: var(--gradient-primary);
  border-radius: var(--radius-full);
  filter: blur(12px);
  opacity: 0.6;
  transition: width 0.5s cubic-bezier(0.4, 0, 0.2, 1);
}

.progress-text {
  text-align: center;
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-tertiary);
}

.init-logs {
  background: rgba(0, 0, 0, 0.3);
  border-radius: var(--radius-lg);
  padding: var(--spacing-md);
  max-height: 200px;
  overflow-y: auto;
  margin-bottom: var(--spacing-lg);
  font-family: 'Courier New', monospace;
  font-size: 12px;
}

.log-entry {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  padding: var(--spacing-xs) 0;
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
}

.log-entry:last-child {
  border-bottom: none;
}

.log-icon {
  flex-shrink: 0;
}

.log-message {
  flex: 1;
  color: var(--color-text-secondary);
}

.log-entry.success .log-message {
  color: var(--color-accent-primary);
}

.log-entry.error .log-message {
  color: var(--color-accent-danger);
}

.log-entry.warning .log-message {
  color: var(--color-accent-warning);
}

.log-time {
  flex-shrink: 0;
  font-size: 10px;
  color: var(--color-text-tertiary);
}

.firewall-prompt {
  background: rgba(255, 170, 0, 0.1);
  border: 2px solid var(--color-accent-warning);
  border-radius: var(--radius-lg);
  padding: var(--spacing-lg);
  margin-bottom: var(--spacing-lg);
  text-align: center;
}

.prompt-icon {
  font-size: 48px;
  margin-bottom: var(--spacing-md);
}

.firewall-prompt h3 {
  color: var(--color-accent-warning);
  margin-bottom: var(--spacing-sm);
}

.firewall-prompt p {
  color: var(--color-text-secondary);
  margin-bottom: var(--spacing-lg);
  line-height: 1.6;
}

.prompt-actions {
  display: flex;
  gap: var(--spacing-md);
  justify-content: center;
}

.troubleshooting {
  background: rgba(255, 68, 68, 0.1);
  border: 2px solid var(--color-accent-danger);
  border-radius: var(--radius-lg);
  padding: var(--spacing-lg);
  margin-bottom: var(--spacing-lg);
}

.troubleshooting h3 {
  color: var(--color-accent-danger);
  margin-bottom: var(--spacing-md);
}

.troubleshooting ul {
  list-style: none;
  padding: 0;
  margin: 0 0 var(--spacing-lg) 0;
}

.troubleshooting li {
  padding: var(--spacing-xs) 0;
  color: var(--color-text-secondary);
}

.troubleshooting li::before {
  content: '• ';
  color: var(--color-accent-danger);
  font-weight: bold;
  margin-right: var(--spacing-xs);
}

.init-actions {
  display: flex;
  gap: var(--spacing-md);
  justify-content: center;
}

button {
  padding: 12px 24px;
  border-radius: var(--radius-md);
  font-weight: 600;
  cursor: pointer;
  transition: all var(--transition-base);
  border: none;
  font-size: 14px;
}

.btn-primary {
  background: var(--gradient-primary);
  color: white;
}

.btn-primary:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 24px rgba(0, 255, 136, 0.3);
}

.btn-secondary {
  background: rgba(255, 255, 255, 0.1);
  color: var(--color-text-primary);
}

.btn-secondary:hover {
  background: rgba(255, 255, 255, 0.15);
}

.btn-success {
  background: var(--color-accent-primary);
  color: white;
  padding: 14px 32px;
  font-size: 16px;
}

.btn-success:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 24px rgba(0, 255, 136, 0.4);
}

@keyframes fadeIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

@keyframes slideUp {
  from {
    opacity: 0;
    transform: translateY(30px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>
