<template>
  <div class="peers-view">
    <div class="view-header">
      <div class="header-content">
        <h1 class="view-title gradient-text">Network Peers</h1>
        <p class="view-subtitle">{{ peers.length }} peers connected</p>
      </div>
      <button class="btn-primary" @click="showAddPeer = true">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
          <line x1="12" y1="5" x2="12" y2="19"/>
          <line x1="5" y1="12" x2="19" y2="12"/>
        </svg>
        <span>Add Peer</span>
      </button>
    </div>

    <div class="peers-grid">
      <PeerCard
        v-for="peer in peers"
        :key="peer.id"
        :peer="peer"
        @remove="removePeer"
      />
    </div>

    <!-- Add Peer Modal -->
    <div v-if="showAddPeer" class="modal-overlay" @click="showAddPeer = false">
      <div class="modal-card glass" @click.stop>
        <div class="modal-header">
          <h2>Add New Peer</h2>
          <button class="close-btn" @click="showAddPeer = false">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
              <line x1="18" y1="6" x2="6" y2="18"/>
              <line x1="6" y1="6" x2="18" y2="18"/>
            </svg>
          </button>
        </div>
        <div class="modal-body">
          <label>Peer Address</label>
          <input 
            v-model="newPeerAddress" 
            type="text" 
            placeholder="192.168.1.100:8080"
            @keyup.enter="addPeer"
          />
        </div>
        <div class="modal-actions">
          <button class="btn-secondary" @click="showAddPeer = false">Cancel</button>
          <button class="btn-primary" @click="addPeer">Add Peer</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import PeerCard from './PeerCard.vue'
import type { Peer } from './PeerCard.vue'

const peers = ref<Peer[]>([])
const loading = ref(false)
const showAddPeer = ref(false)
const newPeerAddress = ref('')

const loadPeers = async () => {
  loading.value = true
  try {
    const peerList = await invoke<Peer[]>('list_peers')
    peers.value = peerList
  } catch (error) {
    console.error('Failed to load peers:', error)
  } finally {
    loading.value = false
  }
}

const addPeer = async () => {
  if (newPeerAddress.value) {
    try {
      await invoke('add_peer', { address: newPeerAddress.value })
      newPeerAddress.value = ''
      showAddPeer.value = false
      await loadPeers()
    } catch (error) {
      console.error('Failed to add peer:', error)
      alert(`Failed to add peer: ${error}`)
    }
  }
}

const removePeer = (peer: Peer) => {
  // TODO: Implement remove peer command
  peers.value = peers.value.filter(p => p.id !== peer.id)
}

onMounted(() => {
  loadPeers()
  // Refresh every 10 seconds
  setInterval(loadPeers, 10000)
})
</script>

<style scoped>
.peers-view {
  padding: var(--spacing-xl);
  animation: fadeIn 0.4s ease-out;
}

.view-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: var(--spacing-xl);
  padding-bottom: var(--spacing-lg);
  border-bottom: var(--border-subtle);
}

.header-content {
  flex: 1;
}

.view-title {
  font-size: 36px;
  font-weight: 800;
  margin-bottom: var(--spacing-xs);
  letter-spacing: -0.5px;
}

.view-subtitle {
  font-size: 14px;
  color: var(--color-text-secondary);
  font-weight: 500;
}

.btn-primary {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  padding: 12px 24px;
  background: var(--gradient-primary);
  border: none;
  border-radius: var(--radius-md);
  color: var(--color-bg-primary);
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all var(--transition-base);
  box-shadow: var(--shadow-sm);
}

.btn-primary:hover {
  transform: translateY(-2px);
  box-shadow: var(--glow-primary), var(--shadow-md);
}

.peers-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: var(--spacing-md);
}

.peer-card {
  padding: var(--spacing-lg);
  border-radius: var(--radius-lg);
  transition: all var(--transition-base);
  animation: fadeIn 0.5s ease-out both;
}

.peer-card:hover {
  transform: translateY(-4px);
  box-shadow: var(--shadow-md);
}

.peer-header {
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
  margin-bottom: var(--spacing-md);
}

.peer-status {
  position: relative;
  width: 14px;
  height: 14px;
  border-radius: 50%;
  flex-shrink: 0;
}

.peer-status.online {
  background: var(--color-accent-primary);
  box-shadow: 0 0 16px rgba(0, 255, 136, 0.6);
}

.peer-status.offline {
  background: var(--color-text-tertiary);
}

.status-pulse {
  position: absolute;
  inset: -4px;
  border-radius: 50%;
  border: 2px solid var(--color-accent-primary);
  opacity: 0;
}

.peer-status.online .status-pulse {
  animation: pulse 2s ease-in-out infinite;
}

.peer-address {
  flex: 1;
  font-size: 15px;
  font-weight: 600;
  color: var(--color-text-primary);
}

.remove-btn {
  width: 32px;
  height: 32px;
  background: rgba(255, 255, 255, 0.03);
  border: var(--border-subtle);
  border-radius: var(--radius-sm);
  color: var(--color-text-tertiary);
  cursor: pointer;
  transition: all var(--transition-fast);
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
}

.peer-card:hover .remove-btn {
  opacity: 1;
}

.remove-btn:hover {
  background: rgba(255, 51, 102, 0.15);
  border-color: var(--color-accent-danger);
  color: var(--color-accent-danger);
  transform: rotate(90deg);
}

.peer-stats {
  display: flex;
  gap: var(--spacing-lg);
}

.peer-stat {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  font-size: 13px;
  color: var(--color-text-secondary);
  font-weight: 500;
}

.peer-stat svg {
  color: var(--color-accent-primary);
}

.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.8);
  backdrop-filter: blur(20px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  animation: fadeIn 0.2s ease-out;
}

.modal-card {
  border-radius: var(--radius-xl);
  min-width: 480px;
  box-shadow: var(--shadow-xl);
  animation: fadeIn 0.3s ease-out 0.1s both;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--spacing-xl);
  border-bottom: var(--border-subtle);
}

.modal-header h2 {
  font-size: 20px;
  font-weight: 700;
  color: var(--color-text-primary);
  margin: 0;
}

.close-btn {
  width: 32px;
  height: 32px;
  background: rgba(255, 255, 255, 0.05);
  border: var(--border-subtle);
  border-radius: var(--radius-sm);
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all var(--transition-fast);
  display: flex;
  align-items: center;
  justify-content: center;
}

.close-btn:hover {
  background: rgba(255, 51, 102, 0.15);
  border-color: var(--color-accent-danger);
  color: var(--color-accent-danger);
  transform: rotate(90deg);
}

.modal-body {
  padding: var(--spacing-xl);
}

.modal-body label {
  display: block;
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-secondary);
  text-transform: uppercase;
  letter-spacing: 1px;
  margin-bottom: var(--spacing-sm);
}

.modal-body input {
  width: 100%;
}

.modal-actions {
  display: flex;
  gap: var(--spacing-sm);
  justify-content: flex-end;
  padding: var(--spacing-xl);
  border-top: var(--border-subtle);
}

.btn-secondary {
  padding: 12px 24px;
  background: rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(10px);
  border: var(--border-subtle);
  border-radius: var(--radius-md);
  color: var(--color-text-primary);
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all var(--transition-base);
}

.btn-secondary:hover {
  background: rgba(255, 255, 255, 0.08);
  border-color: rgba(255, 255, 255, 0.15);
}
</style>
