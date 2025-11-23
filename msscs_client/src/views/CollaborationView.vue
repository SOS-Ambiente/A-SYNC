<template>
  <div class="collaboration-view">
    <div class="header">
      <h1>🤝 Colaboração</h1>
      <p>Gerencie workspaces, membros e pastas compartilhadas</p>
    </div>

    <div class="content-grid">
      <!-- Painel de Workspaces -->
      <div class="panel">
        <WorkspacePanel />
      </div>

      <!-- Estatísticas de Armazenamento -->
      <div class="panel">
        <StorageStats :stats="storageStats" />
      </div>

      <!-- Arquivos do Workspace Atual -->
      <div class="panel full-width" v-if="currentWorkspace">
        <h3>📁 Arquivos de {{ currentWorkspace.name }}</h3>
        <div class="files-grid">
          <div
            v-for="file in workspaceFiles"
            :key="file.uuid"
            class="file-card"
          >
            <div class="file-icon">📄</div>
            <div class="file-info">
              <strong>{{ file.path }}</strong>
              <span>{{ formatBytes(file.size) }}</span>
              <span>{{ file.blocks }} blocos</span>
            </div>
            <div class="file-actions">
              <button @click="downloadFile(file)" class="btn-icon" title="Download">
                ⬇️
              </button>
              <button @click="shareFile(file)" class="btn-icon" title="Compartilhar">
                🔗
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- Hosts Online -->
      <div class="panel">
        <h3>🌐 Hosts Online</h3>
        <div class="hosts-list">
          <div
            v-for="host in onlineHosts"
            :key="host.peer_id"
            class="host-card"
          >
            <div class="host-status online">●</div>
            <div class="host-info">
              <strong>{{ host.peer_id.substring(0, 12) }}...</strong>
              <span>{{ formatBytes(host.available_space) }} disponível</span>
              <span>{{ host.latency_ms }}ms latência</span>
            </div>
          </div>
        </div>
      </div>

      <!-- Atividade Recente -->
      <div class="panel">
        <h3>📊 Atividade Recente</h3>
        <div class="activity-list">
          <div
            v-for="activity in recentActivity"
            :key="activity.id"
            class="activity-item"
          >
            <div class="activity-icon">{{ activity.icon }}</div>
            <div class="activity-info">
              <strong>{{ activity.title }}</strong>
              <span>{{ activity.description }}</span>
              <span class="activity-time">{{ formatTime(activity.timestamp) }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useWorkspaceStore } from '../stores/workspaceStore'
import { useFilesStore } from '../stores/filesStore'
import { storeToRefs } from 'pinia'
import WorkspacePanel from '../components/WorkspacePanel.vue'
import StorageStats from '../components/StorageStats.vue'

const workspaceStore = useWorkspaceStore()
const filesStore = useFilesStore()
const { currentWorkspace } = storeToRefs(workspaceStore)

const storageStats = ref({
  total_files: 0,
  total_size: 0,
  replicated_files: 0,
  online_hosts: 0,
  total_hosts: 0,
  network_capacity: 0,
  network_used: 0
})

const onlineHosts = ref<any[]>([])
const recentActivity = ref<any[]>([])

const workspaceFiles = computed(() => {
  if (!currentWorkspace.value) return []
  // Filtrar arquivos do workspace atual
  return filesStore.files.filter(file => {
    // Implementar lógica de filtro por workspace
    return true
  })
})

onMounted(async () => {
  await loadData()
  // Atualizar a cada 10 segundos
  setInterval(loadData, 10000)
})

const loadData = async () => {
  // Carregar estatísticas (implementar comando Tauri)
  // storageStats.value = await invoke('get_distributed_storage_stats')
  
  // Simular dados para demonstração
  storageStats.value = {
    total_files: 42,
    total_size: 1024 * 1024 * 150, // 150 MB
    replicated_files: 38,
    online_hosts: 5,
    total_hosts: 8,
    network_capacity: 1024 * 1024 * 1024 * 10, // 10 GB
    network_used: 1024 * 1024 * 500 // 500 MB
  }
  
  onlineHosts.value = [
    {
      peer_id: 'peer-abc123def456',
      available_space: 1024 * 1024 * 1024 * 2,
      latency_ms: 45
    },
    {
      peer_id: 'peer-xyz789ghi012',
      available_space: 1024 * 1024 * 1024 * 5,
      latency_ms: 78
    }
  ]
  
  recentActivity.value = [
    {
      id: 1,
      icon: '📤',
      title: 'Arquivo enviado',
      description: 'documento.pdf adicionado ao workspace',
      timestamp: Date.now() - 300000
    },
    {
      id: 2,
      icon: '👥',
      title: 'Novo membro',
      description: 'user@email.com entrou no workspace',
      timestamp: Date.now() - 600000
    }
  ]
}

const downloadFile = async (file: any) => {
  try {
    await filesStore.downloadFile(file.path, `./downloads/${file.path}`)
    alert('Download iniciado!')
  } catch (error) {
    alert('Erro ao baixar arquivo: ' + error)
  }
}

const shareFile = (file: any) => {
  // Implementar compartilhamento
  alert('Compartilhar: ' + file.path)
}

const formatBytes = (bytes: number): string => {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i]
}

const formatTime = (timestamp: number): string => {
  const diff = Date.now() - timestamp
  const minutes = Math.floor(diff / 60000)
  const hours = Math.floor(minutes / 60)
  const days = Math.floor(hours / 24)
  
  if (days > 0) return `${days}d atrás`
  if (hours > 0) return `${hours}h atrás`
  if (minutes > 0) return `${minutes}m atrás`
  return 'agora'
}
</script>

<style scoped>
.collaboration-view {
  padding: 20px;
  max-width: 1400px;
  margin: 0 auto;
}

.header {
  margin-bottom: 30px;
}

.header h1 {
  margin: 0 0 8px 0;
  color: #fff;
}

.header p {
  margin: 0;
  color: #888;
}

.content-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
  gap: 20px;
}

.panel {
  background: #2a2a2a;
  border-radius: 8px;
  padding: 20px;
}

.panel.full-width {
  grid-column: 1 / -1;
}

.panel h3 {
  margin: 0 0 15px 0;
  color: #fff;
}

.files-grid {
  display: grid;
  gap: 10px;
}

.file-card {
  background: #1a1a1a;
  border: 1px solid #3a3a3a;
  border-radius: 6px;
  padding: 12px;
  display: flex;
  align-items: center;
  gap: 12px;
}

.file-icon {
  font-size: 32px;
}

.file-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.file-info strong {
  color: #fff;
}

.file-info span {
  font-size: 13px;
  color: #888;
}

.file-actions {
  display: flex;
  gap: 8px;
}

.btn-icon {
  background: transparent;
  border: 1px solid #3a3a3a;
  border-radius: 4px;
  padding: 8px;
  cursor: pointer;
  font-size: 16px;
  transition: all 0.2s;
}

.btn-icon:hover {
  background: #3a3a3a;
  border-color: #4a9eff;
}

.hosts-list, .activity-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.host-card {
  background: #1a1a1a;
  border: 1px solid #3a3a3a;
  border-radius: 6px;
  padding: 12px;
  display: flex;
  align-items: center;
  gap: 12px;
}

.host-status {
  width: 12px;
  height: 12px;
  border-radius: 50%;
}

.host-status.online {
  background: #4ade80;
  box-shadow: 0 0 8px #4ade80;
}

.host-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.host-info strong {
  color: #fff;
}

.host-info span {
  font-size: 13px;
  color: #888;
}

.activity-item {
  background: #1a1a1a;
  border: 1px solid #3a3a3a;
  border-radius: 6px;
  padding: 12px;
  display: flex;
  align-items: flex-start;
  gap: 12px;
}

.activity-icon {
  font-size: 24px;
}

.activity-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.activity-info strong {
  color: #fff;
}

.activity-info span {
  font-size: 13px;
  color: #888;
}

.activity-time {
  color: #666 !important;
  font-size: 12px !important;
}
</style>
