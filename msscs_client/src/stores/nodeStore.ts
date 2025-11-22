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
    try {
      console.log('Initializing node...')
      await invoke('start_node')
      console.log('Node started successfully')
      status.value = 'online'
      startMetricsPolling()
    } catch (error) {
      console.error('Failed to start node:', error)
      status.value = 'offline'
      // Show error to user
      alert(`Failed to start node: ${error}`)
    }
  }

  const updateMetrics = async () => {
    try {
      const data = await invoke<NodeMetrics>('get_metrics')
      metrics.value = data
      storageUsed.value = data.storage_bytes
      peerCount.value = data.peer_count
      blockCount.value = data.block_count
      
      // Update status based on metrics
      if (data.peer_count > 0) {
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
