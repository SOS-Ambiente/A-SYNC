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

  const initialize = async () => {
    // Check if running in Tauri context
    if (typeof window !== 'undefined' && !(window as any).__TAURI_IPC__) {
      console.warn('Not running in Tauri context - node initialization skipped')
      status.value = 'offline'
      return
    }

    try {
      console.log('🚀 Initializing MSSCS node...')
      
      // Check if already running
      const isRunning = await invoke<boolean>('is_node_running')
      if (isRunning) {
        console.log('✅ Node already running')
        status.value = 'online'
        startMetricsPolling()
        return
      }
      
      // Start the node
      console.log('⏳ Starting node...')
      await invoke('start_node')
      console.log('✅ Node started successfully!')
      status.value = 'online'
      startMetricsPolling()
    } catch (error) {
      console.error('❌ Failed to start node:', error)
      status.value = 'offline'
      const errorMsg = error instanceof Error ? error.message : String(error)
      console.error('Detailed error:', errorMsg)
      
      // Show error in console but don't block UI with alert
      console.error('⚠️  Node initialization failed. Some features may not work.')
    }
  }

  const updateMetrics = async () => {
    try {
      const data = await invoke<NodeMetrics>('get_metrics')
      metrics.value = data
      storageUsed.value = data.storage_bytes
      peerCount.value = data.peer_count
      blockCount.value = data.block_count
      
      // Node is online if metrics are available (node is running)
      // Status will show peer count separately
      if (status.value !== 'online') {
        status.value = 'online'
      }
    } catch (error) {
      console.error('Failed to fetch metrics:', error)
      status.value = 'offline'
    }
  }

  const startMetricsPolling = () => {
    updateMetrics()
    setInterval(updateMetrics, 5000) // Update every 5 seconds
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
