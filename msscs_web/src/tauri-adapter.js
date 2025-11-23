// Tauri API adapter for web version
// This provides mock implementations of Tauri APIs for browser compatibility

export const invoke = async (command, args = {}) => {
  console.warn(`[Tauri Adapter] Mock invoke: ${command}`, args)
  
  // Use the global msscsNode instance
  const node = window.msscsNode
  
  switch (command) {
    case 'is_node_running':
      return node?.p2p?.peer !== null
      
    case 'start_node':
      // Node is already started in nodeStore.initialize()
      return { success: true }
      
    case 'get_metrics':
      if (!node) throw new Error('Node not initialized')
      
      const connStats = node.p2p.getConnectionStats()
      const allFiles = await node.storage.getAllFiles()
      const totalSize = allFiles.reduce((sum, f) => sum + f.size, 0)
      const totalBlocks = allFiles.reduce((sum, f) => sum + f.chunks.length, 0)
      
      return {
        block_count: totalBlocks,
        storage_bytes: totalSize,
        peer_count: connStats.connectedPeers,
        uptime_seconds: Math.floor((Date.now() - (window.msscsStartTime || Date.now())) / 1000),
        requests_total: 0,
        requests_failed: 0,
        success_rate: 100
      }
      
    case 'upload_file':
      if (!node) throw new Error('Node not initialized')
      // Implement file upload logic
      throw new Error('Not implemented in web version - use UI')
      
    case 'download_file':
      if (!node) throw new Error('Node not initialized')
      // Implement file download logic
      throw new Error('Not implemented in web version - use UI')
      
    default:
      throw new Error(`Unknown command: ${command}`)
  }
}

export const appWindow = {
  minimize: () => console.log('[Tauri Adapter] Window minimize not available in web'),
  toggleMaximize: () => console.log('[Tauri Adapter] Window maximize not available in web'),
  close: () => window.close(),
}

// Make it available globally for compatibility
if (typeof window !== 'undefined') {
  window.__TAURI__ = {
    tauri: { invoke },
    window: { appWindow }
  }
}
