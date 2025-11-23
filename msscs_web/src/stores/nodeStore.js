import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useNodeStore = defineStore('node', () => {
  const status = ref('offline')
  const storageUsed = ref(0)
  const storageTotal = ref(1024 * 1024 * 1024) // 1GB default for web
  const peerCount = ref(0)
  const blockCount = ref(0)
  const peerId = ref(null)
  
  let p2p = null
  let storage = null
  let crypto = null
  let quantumCrypto = null
  let erasureCoding = null
  let bridge = null
  let files = new Map()
  let pollingInterval = null
  let consecutiveErrors = 0
  const maxConsecutiveErrors = 5

  const initialize = async () => {
    try {
      console.log('🚀 Initializing MSSCS Web node...')
      status.value = 'syncing'
      
      // Initialize P2P modules
      const P2PNetwork = window.P2PNetwork
      const StorageManager = window.StorageManager
      const CryptoManager = window.CryptoManager
      const QuantumCryptoManager = window.QuantumCryptoManager
      const ErasureCoding = window.ErasureCoding
      const WebRTCBridge = window.WebRTCBridge
      
      p2p = new P2PNetwork()
      storage = new StorageManager()
      crypto = new CryptoManager()
      quantumCrypto = new QuantumCryptoManager()
      erasureCoding = new ErasureCoding(10, 4)
      
      // Initialize crypto with stored or default passphrase
      console.log('🔐 Initializing quantum-resistant cryptography...')
      let passphrase = localStorage.getItem('msscs_passphrase')
      if (!passphrase) {
        // Generate a secure random passphrase for first-time users
        passphrase = Array.from(crypto.getRandomValues(new Uint8Array(32)))
          .map(b => b.toString(16).padStart(2, '0'))
          .join('')
        localStorage.setItem('msscs_passphrase', passphrase)
        console.log('✅ Generated new encryption key')
      }
      
      await quantumCrypto.init(passphrase)
      await crypto.init()
      console.log('✅ Encryption ready')
      
      // Initialize storage
      console.log('💾 Initializing local storage...')
      await storage.init()
      console.log('✅ Storage ready')
      
      // Load existing files
      const existingFiles = await storage.getAllFiles()
      for (const file of existingFiles) {
        files.set(file.id, file)
      }
      console.log(`✅ Loaded ${files.size} files`)
      
      // Initialize P2P with proper error handling
      console.log('🌐 Connecting to P2P network...')
      try {
        await Promise.race([
          p2p.init(),
          new Promise((_, reject) => 
            setTimeout(() => reject(new Error('P2P initialization timeout')), 10000)
          )
        ])
        console.log('✅ P2P network connected')
      } catch (p2pError) {
        console.warn('⚠️  P2P initialization issue:', p2pError.message)
        console.log('💡 Continuing with limited connectivity')
        // Don't fail completely - continue with limited functionality
      }
      
      // Initialize WebRTC bridge
      console.log('🌉 Initializing WebRTC bridge...')
      bridge = new WebRTCBridge(p2p)
      await bridge.init()
      console.log('✅ Bridge ready')
      
      // Setup P2P event listeners
      setupP2PListeners()
      
      // CRITICAL FIX: Transition to online immediately after successful P2P init
      const connStats = p2p.getConnectionStats()
      if (connStats.isConnected && connStats.peerId) {
        peerId.value = connStats.peerId
        status.value = 'online'
        console.log('✅ Node is now ONLINE')
        console.log('🆔 Peer ID:', connStats.peerId)
      } else {
        // Even without P2P, we can still use local storage
        status.value = 'online'
        console.log('✅ Node is online (local mode)')
      }
      
      // Start metrics polling
      startMetricsPolling()
      
      // Make modules globally available for legacy code
      window.msscsNode = {
        p2p,
        storage,
        crypto,
        quantumCrypto,
        erasureCoding,
        bridge,
        files
      }
      
      console.log('✅ MSSCS Web node initialized successfully')
      
    } catch (error) {
      console.error('❌ Failed to initialize node:', error)
      status.value = 'offline'
    }
  }

  const setupP2PListeners = () => {
    if (!p2p) return
    
    p2p.on('ready', (id) => {
      peerId.value = id
      status.value = 'online'
      console.log('✅ P2P ready with ID:', id)
    })
    
    p2p.on('peer-connected', (id) => {
      console.log('✅ Peer connected:', id)
      updateMetrics()
    })
    
    p2p.on('peer-disconnected', (id) => {
      console.log('❌ Peer disconnected:', id)
      updateMetrics()
    })
    
    p2p.on('block-request', async (data) => {
      const { blockId, peerId: requestPeerId } = data
      const block = await storage.getBlock(blockId)
      if (block) {
        p2p.sendBlock(requestPeerId, block)
      }
    })
    
    p2p.on('block-received', async (data) => {
      const { block } = data
      await storage.saveBlock(block)
      updateMetrics()
    })
  }

  const updateMetrics = async () => {
    if (!p2p || !storage) return
    
    try {
      // Get P2P stats
      const connStats = p2p.getConnectionStats()
      peerCount.value = connStats.connectedPeers
      
      // Reset error counter on success
      consecutiveErrors = 0
      
      // CRITICAL FIX: If we have a peer ID, we're online
      if (connStats.isConnected && connStats.peerId) {
        const wasOffline = status.value !== 'online'
        status.value = 'online'
        peerId.value = connStats.peerId
        
        if (wasOffline) {
          console.log('✅ Node transitioned to ONLINE')
          console.log(`📊 ${peerCount.value} peers connected`)
        }
      } else if (status.value === 'online') {
        consecutiveErrors++
        if (consecutiveErrors >= maxConsecutiveErrors) {
          status.value = 'syncing'
          console.warn('⚠️  Node connection unstable, attempting to reconnect...')
        }
      }
      
      // Get storage stats
      const allFiles = await storage.getAllFiles()
      files.clear()
      for (const file of allFiles) {
        files.set(file.id, file)
      }
      
      const totalSize = Array.from(files.values()).reduce((sum, f) => sum + f.size, 0)
      storageUsed.value = totalSize
      blockCount.value = Array.from(files.values()).reduce((sum, f) => sum + f.chunks.length, 0)
      
      // Update storage limit from localStorage
      const limitMB = parseInt(localStorage.getItem('msscs_storage_limit_mb') || '1024')
      storageTotal.value = limitMB * 1024 * 1024
      
    } catch (error) {
      consecutiveErrors++
      if (consecutiveErrors >= maxConsecutiveErrors) {
        console.error('Failed to update metrics:', error)
      }
    }
  }

  const startMetricsPolling = () => {
    if (pollingInterval) return
    
    updateMetrics()
    pollingInterval = setInterval(updateMetrics, 3000)
    console.log('📊 Started metrics polling (every 3 seconds)')
  }

  return {
    status,
    storageUsed,
    storageTotal,
    peerCount,
    blockCount,
    peerId,
    initialize,
    updateMetrics,
  }
})
