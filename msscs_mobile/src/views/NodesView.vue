<template>
  <div class="nodes-view">
    <div class="header">
      <h1>Network Nodes</h1>
      <button class="btn-scan" @click="scanNodes" :disabled="scanning">
        <span v-if="!scanning">🔍</span>
        <div v-else class="spinner-small"></div>
      </button>
    </div>

    <div class="section">
      <h2>Discovered Nodes</h2>
      <p class="section-desc">Automatically found on local network</p>
      
      <div v-if="discoveredNodes.length === 0" class="empty">
        <p>No nodes discovered yet</p>
        <button @click="scanNodes">Scan Network</button>
      </div>

      <div v-else class="node-list">
        <div
          v-for="node in discoveredNodes"
          :key="node.address"
          class="node-item"
        >
          <div class="node-icon">🖥️</div>
          <div class="node-info">
            <div class="node-name">{{ node.name }}</div>
            <div class="node-address">{{ node.address }}:{{ node.port }}</div>
          </div>
          <button
            v-if="!isConnected(node.address)"
            class="btn-connect"
            @click="connectNode(node)"
          >
            Connect
          </button>
          <span v-else class="connected">✓</span>
        </div>
      </div>
    </div>

    <div class="section">
      <h2>Connected Peers</h2>
      <p class="section-desc">Active connections</p>
      
      <div v-if="connectedPeers.length === 0" class="empty">
        <p>No connected peers</p>
      </div>

      <div v-else class="node-list">
        <div
          v-for="peer in connectedPeers"
          :key="peer"
          class="node-item"
        >
          <div class="node-icon online">🟢</div>
          <div class="node-info">
            <div class="node-name">Peer</div>
            <div class="node-address">{{ peer }}</div>
          </div>
          <button class="btn-disconnect" @click="disconnectPeer(peer)">
            Disconnect
          </button>
        </div>
      </div>
    </div>

    <div class="section">
      <h2>Add Manual Node</h2>
      <div class="manual-form">
        <input
          v-model="manualAddress"
          type="text"
          placeholder="192.168.1.100:8080"
          class="input"
        />
        <button @click="addManualNode" class="btn-add">Add</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useNodeStore } from '../stores/nodeStore'

interface DiscoveredNode {
  name: string
  address: string
  port: number
  node_id: string
}

const nodeStore = useNodeStore()

const scanning = ref(false)
const discoveredNodes = ref<DiscoveredNode[]>([])
const connectedPeers = ref<string[]>([])
const manualAddress = ref('')

const isConnected = (address: string) => {
  return connectedPeers.value.some(peer => peer.includes(address))
}

onMounted(async () => {
  await scanNodes()
})

async function scanNodes() {
  scanning.value = true
  try {
    const nodes = await invoke<DiscoveredNode[]>('discover_nodes')
    discoveredNodes.value = nodes
  } catch (error) {
    console.error('Failed to discover nodes:', error)
  } finally {
    scanning.value = false
  }
}

async function connectNode(node: DiscoveredNode) {
  try {
    const address = `${node.address}:${node.port}`
    await invoke('connect_to_node', { address })
    connectedPeers.value.push(address)
    alert(`Connected to ${node.name}`)
  } catch (error) {
    alert('Failed to connect to node')
  }
}

function disconnectPeer(peer: string) {
  connectedPeers.value = connectedPeers.value.filter(p => p !== peer)
}

async function addManualNode() {
  if (!manualAddress.value) return
  
  try {
    await invoke('connect_to_node', { address: manualAddress.value })
    connectedPeers.value.push(manualAddress.value)
    manualAddress.value = ''
    alert('Node added successfully')
  } catch (error) {
    alert('Failed to add node')
  }
}
</script>

<style scoped>
.nodes-view {
  padding: 16px;
  padding-bottom: 80px;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

h1 {
  font-size: 24px;
  font-weight: 600;
}

.btn-scan {
  width: 48px;
  height: 48px;
  border: none;
  background: #222;
  border-radius: 50%;
  font-size: 24px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
}

.btn-scan:disabled {
  opacity: 0.5;
}

.spinner-small {
  width: 24px;
  height: 24px;
  border: 3px solid #333;
  border-top-color: #00ff88;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

.section {
  margin-bottom: 32px;
}

.section h2 {
  font-size: 18px;
  font-weight: 600;
  margin-bottom: 4px;
}

.section-desc {
  font-size: 14px;
  color: #888;
  margin-bottom: 16px;
}

.empty {
  padding: 32px;
  text-align: center;
  background: #111;
  border-radius: 12px;
}

.empty p {
  color: #888;
  margin-bottom: 16px;
}

.empty button {
  padding: 12px 24px;
  border: none;
  background: #222;
  color: #fff;
  border-radius: 8px;
  cursor: pointer;
}

.node-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.node-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px;
  background: #111;
  border-radius: 12px;
  transition: transform 0.15s cubic-bezier(0.4, 0, 0.2, 1);
  /* Performance */
  will-change: transform;
  transform: translateZ(0);
  backface-visibility: hidden;
  contain: layout style paint;
}

.node-item:active {
  transform: scale(0.98) translateZ(0);
}

.node-icon {
  font-size: 32px;
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #222;
  border-radius: 12px;
}

.node-icon.online {
  background: transparent;
}

.node-info {
  flex: 1;
  min-width: 0;
}

.node-name {
  font-size: 16px;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.node-address {
  font-size: 12px;
  color: #888;
  margin-top: 2px;
}

.btn-connect,
.btn-disconnect {
  padding: 8px 16px;
  border: none;
  background: #00ff88;
  color: #000;
  font-weight: 600;
  border-radius: 8px;
  cursor: pointer;
  white-space: nowrap;
  transition: all 0.15s cubic-bezier(0.4, 0, 0.2, 1);
  /* Performance */
  will-change: transform;
  transform: translateZ(0);
  backface-visibility: hidden;
  -webkit-tap-highlight-color: transparent;
  box-shadow: 0 2px 8px rgba(0, 255, 136, 0.3);
}

.btn-connect:active {
  transform: scale(0.95) translateZ(0);
  box-shadow: 0 1px 4px rgba(0, 255, 136, 0.4);
}

.btn-disconnect {
  background: #ff4444;
  color: #fff;
  box-shadow: 0 2px 8px rgba(255, 68, 68, 0.3);
}

.btn-disconnect:active {
  transform: scale(0.95) translateZ(0);
  box-shadow: 0 1px 4px rgba(255, 68, 68, 0.4);
}

.connected {
  color: #00ff88;
  font-size: 24px;
}

.manual-form {
  display: flex;
  gap: 8px;
}

.input {
  flex: 1;
  padding: 16px;
  border: 1px solid #222;
  background: #111;
  color: #fff;
  border-radius: 12px;
  font-size: 16px;
}

.input::placeholder {
  color: #666;
}

.btn-add {
  padding: 16px 24px;
  border: none;
  background: #00ff88;
  color: #000;
  font-weight: 600;
  border-radius: 12px;
  cursor: pointer;
  white-space: nowrap;
  transition: all 0.15s cubic-bezier(0.4, 0, 0.2, 1);
  /* Performance */
  will-change: transform;
  transform: translateZ(0);
  backface-visibility: hidden;
  -webkit-tap-highlight-color: transparent;
  box-shadow: 0 2px 8px rgba(0, 255, 136, 0.3);
}

.btn-add:active {
  transform: scale(0.95) translateZ(0);
  box-shadow: 0 1px 4px rgba(0, 255, 136, 0.4);
}
</style>
