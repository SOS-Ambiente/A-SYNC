import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'

export interface NodeMetrics {
  block_count: number
  storage_bytes: number
  peer_count: number
  uptime_seconds: number
  requests_total: number
  requests_failed: number
  success_rate: number
}

export const useNodeStore = defineStore('node', () => {
  const status = ref<'offline' | 'online' | 'syncing'>('offline')
  const storageUsed = ref(0)
  const storageTotal = ref(100 * 1024 * 1024 * 1024) // 100GB default
  const peerCount = ref(0)
  const blockCount = ref(0)
  const metrics = ref<NodeMetrics | null>(null)
  let pollingInterval: number | null = null
  let consecutiveErrors = 0
  const maxConsecutiveErrors = 5

  const initialize = async () => {
    // Check if running in Tauri context
    if (typeof window !== 'undefined' && !(window as any).__TAURI__) {
      console.warn('⚠️  Not running in Tauri context - node initialization skipped')
      console.log('💡 This is expected in development mode (browser)')
      status.value = 'offline'
      return
    }

    try {
      console.log('🚀 Initializing MSSCS Tauri node...')
      status.value = 'syncing'
      
      // Check if already running
      try {
        const isRunning = await invoke<boolean>('is_node_running')
        if (isRunning) {
          console.log('✅ Node already running, transitioning to online')
          status.value = 'online'
          startMetricsPolling()
          return
        }
      } catch (e) {
        console.log('Node not running yet, starting...')
      }
      
      // CRITICAL FIX: Start node with shorter timeout (5 seconds)
      console.log('⏳ Starting node...')
      
      try {
        // Start node with 5 second timeout (backend is non-blocking now)
        await Promise.race([
          invoke('start_node'),
          new Promise((_, reject) => 
            setTimeout(() => reject(new Error('Node start timeout')), 5000)
          )
        ])
        
        console.log('✅ Node start command completed')
        console.log('   P2P bootstrap continuing in background...')
        
        // Transition to syncing first, then online when metrics confirm
        status.value = 'syncing'
        
        // Start metrics polling - will transition to online when ready
        startMetricsPolling()
        
      } catch (error) {
        console.error('❌ Node initialization failed:', error)
        const errorMsg = error instanceof Error ? error.message : String(error)
        
        // If timeout, still try to go online (node might be starting in background)
        if (errorMsg.includes('timeout')) {
          console.warn('⚠️  Node start timeout - continuing with metrics polling')
          console.log('   Node may still be initializing in background')
          status.value = 'syncing'
          startMetricsPolling() // This will transition to online when metrics work
        } else {
          console.error('Detailed error:', errorMsg)
          status.value = 'offline'
          // Retry after 5 seconds
          setTimeout(() => {
            console.log('🔄 Retrying node initialization...')
            initialize()
          }, 5000)
        }
      }
      
    } catch (error) {
      console.error('❌ Failed to initialize node:', error)
      status.value = 'offline'
      const errorMsg = error instanceof Error ? error.message : String(error)
      console.error('Detailed error:', errorMsg)
    }
  }

  const updateMetrics = async () => {
    // Skip if not in Tauri context
    if (typeof window !== 'undefined' && !(window as any).__TAURI__) {
      return
    }

    try {
      const data = await invoke<NodeMetrics>('get_metrics')
      metrics.value = data
      storageUsed.value = data.storage_bytes
      peerCount.value = data.peer_count
      blockCount.value = data.block_count
      
      // Reset error counter on success
      consecutiveErrors = 0
      
      // CRITICAL FIX: If we can fetch metrics, the node is definitely online
      const wasOffline = status.value !== 'online'
      status.value = 'online'
      
      if (wasOffline) {
        console.log('✅ Node is now online!')
        console.log(`📊 Metrics: ${data.peer_count} peers, ${data.block_count} blocks, ${(data.storage_bytes / (1024 * 1024)).toFixed(2)} MB`)
      }
    } catch (error) {
      consecutiveErrors++
      
      // Handle errors based on current status
      if (status.value === 'syncing') {
        // Still initializing - this is expected (don't log)
      } else if (status.value === 'online') {
        // Was online, now having issues
        if (consecutiveErrors >= maxConsecutiveErrors) {
          console.error('Failed to fetch metrics:', error)
          status.value = 'syncing'
          console.warn('⚠️  Node connection lost, attempting to reconnect...')
        }
      }
      // If offline, stay offline (don't log repeatedly)
    }
  }

  const startMetricsPolling = () => {
    // Prevent multiple polling intervals
    if (pollingInterval) {
      console.log('Metrics polling already active')
      return
    }
    
    updateMetrics()
    pollingInterval = setInterval(updateMetrics, 3000) // Update every 3 seconds
    console.log('📊 Started metrics polling (every 3 seconds)')
  }

  return {
    status,
    storageUsed,
    storageTotal,
    peerCount,
    blockCount,
    metrics,
    initialize,
    updateMetrics,
  }
})
