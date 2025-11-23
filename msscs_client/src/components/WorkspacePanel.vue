<template>
  <div class="workspace-panel">
    <div class="workspace-header">
      <h2>🏢 Workspaces</h2>
      <button @click="showCreateDialog = true" class="btn-primary">
        + Novo Workspace
      </button>
    </div>

    <div v-if="loading" class="loading">Carregando...</div>

    <div v-else class="workspace-list">
      <div
        v-for="workspace in workspaces"
        :key="workspace.id"
        class="workspace-card"
        :class="{ active: currentWorkspace?.id === workspace.id }"
        @click="selectWorkspace(workspace)"
      >
        <div class="workspace-info">
          <h3>{{ workspace.name }}</h3>
          <p>{{ workspace.description }}</p>
          <div class="workspace-meta">
            <span>👥 {{ workspace.members.length }} membros</span>
            <span>📁 {{ workspace.shared_folders.length }} pastas</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Dialog: Criar Workspace -->
    <div v-if="showCreateDialog" class="dialog-overlay" @click="showCreateDialog = false">
      <div class="dialog" @click.stop>
        <h3>Criar Novo Workspace</h3>
        <input
          v-model="newWorkspace.name"
          placeholder="Nome do workspace"
          class="input"
        />
        <textarea
          v-model="newWorkspace.description"
          placeholder="Descrição"
          class="input"
          rows="3"
        ></textarea>
        <div class="dialog-actions">
          <button @click="showCreateDialog = false" class="btn-secondary">
            Cancelar
          </button>
          <button @click="createWorkspace" class="btn-primary">
            Criar
          </button>
        </div>
      </div>
    </div>

    <!-- Painel de Workspace Selecionado -->
    <div v-if="currentWorkspace" class="workspace-details">
      <div class="details-header">
        <h3>{{ currentWorkspace.name }}</h3>
        <button @click="showInviteDialog = true" class="btn-primary">
          📧 Convidar Membro
        </button>
      </div>

      <WorkspaceMemberList
        :members="currentWorkspace.members"
        :can-manage="true"
        @invite="showInviteDialog = true"
        @manage="handleManageMember"
      />

      <SharedFolderList
        :folders="currentWorkspace.shared_folders"
        @create="showFolderDialog = true"
        @select="handleSelectFolder"
        @manage="handleManageFolder"
      />
    </div>

    <!-- Dialog: Convidar Membro -->
    <div v-if="showInviteDialog" class="dialog-overlay" @click="showInviteDialog = false">
      <div class="dialog" @click.stop>
        <h3>Convidar Membro</h3>
        <input
          v-model="inviteData.email"
          placeholder="Email do membro"
          type="email"
          class="input"
        />
        <select v-model="inviteData.permission" class="input">
          <option value="Viewer">Viewer (apenas visualização)</option>
          <option value="Editor">Editor (pode editar)</option>
          <option value="Admin">Admin (controle total)</option>
        </select>
        <div class="dialog-actions">
          <button @click="showInviteDialog = false" class="btn-secondary">
            Cancelar
          </button>
          <button @click="sendInvite" class="btn-primary">
            Enviar Convite
          </button>
        </div>
      </div>
    </div>

    <!-- Dialog: Nova Pasta -->
    <div v-if="showFolderDialog" class="dialog-overlay" @click="showFolderDialog = false">
      <div class="dialog" @click.stop>
        <h3>Criar Pasta Compartilhada</h3>
        <input
          v-model="folderData.name"
          placeholder="Nome da pasta"
          class="input"
        />
        <input
          v-model="folderData.path"
          placeholder="Caminho (ex: /documentos)"
          class="input"
        />
        <div class="dialog-actions">
          <button @click="showFolderDialog = false" class="btn-secondary">
            Cancelar
          </button>
          <button @click="createFolder" class="btn-primary">
            Criar
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useWorkspaceStore } from '../stores/workspaceStore'
import type { WorkspaceMember, SharedFolder } from '../stores/workspaceStore'
import { storeToRefs } from 'pinia'
import WorkspaceMemberList from './WorkspaceMemberList.vue'
import SharedFolderList from './SharedFolderList.vue'

const workspaceStore = useWorkspaceStore()
const { workspaces, currentWorkspace, loading } = storeToRefs(workspaceStore)

const showCreateDialog = ref(false)
const showInviteDialog = ref(false)
const showFolderDialog = ref(false)

const newWorkspace = ref({
  name: '',
  description: ''
})

const inviteData = ref({
  email: '',
  permission: 'Editor'
})

const folderData = ref({
  name: '',
  path: ''
})

onMounted(() => {
  workspaceStore.loadWorkspaces()
})

const selectWorkspace = (workspace: any) => {
  workspaceStore.setCurrentWorkspace(workspace)
}

const createWorkspace = async () => {
  try {
    await workspaceStore.createWorkspace(
      newWorkspace.value.name,
      newWorkspace.value.description
    )
    showCreateDialog.value = false
    newWorkspace.value = { name: '', description: '' }
  } catch (error) {
    alert('Erro ao criar workspace: ' + error)
  }
}

const sendInvite = async () => {
  if (!currentWorkspace.value) return
  
  try {
    await workspaceStore.inviteMember(
      currentWorkspace.value.id,
      inviteData.value.email,
      inviteData.value.permission
    )
    showInviteDialog.value = false
    inviteData.value = { email: '', permission: 'Editor' }
    alert('Convite enviado com sucesso!')
  } catch (error) {
    alert('Erro ao enviar convite: ' + error)
  }
}

const createFolder = async () => {
  if (!currentWorkspace.value) return
  
  try {
    await workspaceStore.createSharedFolder(
      currentWorkspace.value.id,
      folderData.value.name,
      folderData.value.path
    )
    showFolderDialog.value = false
    folderData.value = { name: '', path: '' }
  } catch (error) {
    alert('Erro ao criar pasta: ' + error)
  }
}

const handleManageMember = (member: WorkspaceMember) => {
  console.log('Manage member:', member)
  // TODO: Implement member management
}

const handleSelectFolder = (folder: SharedFolder) => {
  console.log('Select folder:', folder)
  // TODO: Navigate to folder view
}

const handleManageFolder = (folder: SharedFolder) => {
  console.log('Manage folder:', folder)
  // TODO: Implement folder management
}
</script>

<style scoped>
.workspace-panel {
  padding: var(--spacing-xl);
  animation: fadeIn 0.4s ease-out;
}

.workspace-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--spacing-xl);
  padding-bottom: var(--spacing-lg);
  border-bottom: var(--border-subtle);
}

.workspace-header h2 {
  font-size: 36px;
  font-weight: 800;
  background: var(--gradient-primary);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  letter-spacing: -0.5px;
}

.workspace-list {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: var(--spacing-md);
  margin-bottom: var(--spacing-xl);
}

.workspace-card {
  background: rgba(255, 255, 255, 0.03);
  backdrop-filter: blur(20px);
  border: var(--border-subtle);
  border-radius: var(--radius-lg);
  padding: var(--spacing-lg);
  cursor: pointer;
  transition: all var(--transition-base);
  animation: fadeIn 0.5s ease-out both;
}

.workspace-card:hover {
  border-color: var(--color-accent-primary);
  transform: translateY(-4px);
  box-shadow: var(--shadow-lg);
}

.workspace-card.active {
  border-color: var(--color-accent-primary);
  background: rgba(0, 255, 136, 0.05);
  box-shadow: var(--glow-primary);
}

.workspace-info h3 {
  margin: 0 0 var(--spacing-sm) 0;
  color: var(--color-text-primary);
  font-size: 18px;
  font-weight: 700;
}

.workspace-info p {
  margin: 0 0 var(--spacing-md) 0;
  color: var(--color-text-secondary);
  font-size: 14px;
  line-height: 1.5;
}

.workspace-meta {
  display: flex;
  gap: var(--spacing-md);
  font-size: 13px;
  color: var(--color-text-tertiary);
  font-weight: 500;
}

.workspace-details {
  background: rgba(255, 255, 255, 0.03);
  backdrop-filter: blur(20px);
  border: var(--border-subtle);
  border-radius: var(--radius-lg);
  padding: var(--spacing-xl);
  margin-top: var(--spacing-xl);
  animation: fadeIn 0.5s ease-out;
}

.details-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--spacing-xl);
  padding-bottom: var(--spacing-lg);
  border-bottom: var(--border-subtle);
}

.details-header h3 {
  font-size: 24px;
  font-weight: 700;
  color: var(--color-text-primary);
}

.section {
  margin-bottom: var(--spacing-xl);
}

.section h4 {
  margin-bottom: var(--spacing-md);
  color: var(--color-text-primary);
  font-size: 16px;
  font-weight: 700;
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}

.members-list, .folders-list {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
  margin-top: var(--spacing-md);
}

.member-card, .folder-card {
  background: rgba(255, 255, 255, 0.02);
  border: var(--border-subtle);
  border-radius: var(--radius-md);
  padding: var(--spacing-md);
  transition: all var(--transition-fast);
}

.member-card:hover, .folder-card:hover {
  background: rgba(255, 255, 255, 0.05);
  border-color: var(--color-accent-primary);
  transform: translateX(4px);
}

.member-info, .folder-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: var(--spacing-md);
}

.member-info strong, .folder-info strong {
  color: var(--color-text-primary);
  font-weight: 600;
}

.folder-info span {
  color: var(--color-text-tertiary);
  font-size: 13px;
}

.permission-badge {
  padding: 4px 12px;
  border-radius: var(--radius-full);
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.permission-badge.owner {
  background: rgba(255, 51, 102, 0.15);
  color: var(--color-accent-danger);
  border: 1px solid rgba(255, 51, 102, 0.3);
}

.permission-badge.admin {
  background: rgba(255, 170, 0, 0.15);
  color: var(--color-accent-warning);
  border: 1px solid rgba(255, 170, 0, 0.3);
}

.permission-badge.editor {
  background: rgba(0, 204, 255, 0.15);
  color: var(--color-accent-secondary);
  border: 1px solid rgba(0, 204, 255, 0.3);
}

.permission-badge.viewer {
  background: rgba(255, 255, 255, 0.05);
  color: var(--color-text-tertiary);
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.dialog-overlay {
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

.dialog {
  background: rgba(255, 255, 255, 0.03);
  backdrop-filter: blur(40px);
  border: var(--border-medium);
  border-radius: var(--radius-xl);
  padding: var(--spacing-xl);
  min-width: 480px;
  max-width: 600px;
  box-shadow: var(--shadow-xl);
  animation: fadeIn 0.3s ease-out 0.1s both;
}

.dialog h3 {
  margin: 0 0 var(--spacing-lg) 0;
  color: var(--color-text-primary);
  font-size: 20px;
  font-weight: 700;
}

.input {
  width: 100%;
  padding: 12px 16px;
  margin-bottom: var(--spacing-md);
  background: rgba(255, 255, 255, 0.03);
  backdrop-filter: blur(10px);
  border: var(--border-subtle);
  border-radius: var(--radius-md);
  color: var(--color-text-primary);
  font-size: 14px;
  transition: all var(--transition-base);
}

.input:hover {
  background: rgba(255, 255, 255, 0.05);
  border-color: rgba(255, 255, 255, 0.1);
}

.input:focus {
  background: rgba(255, 255, 255, 0.05);
  border-color: var(--color-accent-primary);
  box-shadow: 0 0 0 3px rgba(0, 255, 136, 0.1);
  outline: none;
}

.dialog-actions {
  display: flex;
  gap: var(--spacing-sm);
  justify-content: flex-end;
  margin-top: var(--spacing-xl);
  padding-top: var(--spacing-lg);
  border-top: var(--border-subtle);
}

.btn-primary, .btn-secondary {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  padding: 12px 24px;
  border-radius: var(--radius-md);
  cursor: pointer;
  font-size: 14px;
  font-weight: 600;
  transition: all var(--transition-base);
}

.btn-primary {
  background: var(--gradient-primary);
  border: none;
  color: var(--color-bg-primary);
  box-shadow: var(--shadow-sm);
}

.btn-primary:hover {
  transform: translateY(-2px);
  box-shadow: var(--glow-primary), var(--shadow-md);
}

.btn-secondary {
  background: rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(10px);
  border: var(--border-subtle);
  color: var(--color-text-primary);
}

.btn-secondary:hover {
  background: rgba(255, 255, 255, 0.08);
  border-color: rgba(255, 255, 255, 0.15);
  transform: translateY(-2px);
}

.loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 80px;
  color: var(--color-text-tertiary);
}

.loading::before {
  content: '';
  width: 48px;
  height: 48px;
  border: 3px solid transparent;
  border-top-color: var(--color-accent-primary);
  border-right-color: var(--color-accent-secondary);
  border-radius: 50%;
  animation: spin 1s cubic-bezier(0.68, -0.55, 0.265, 1.55) infinite;
  margin-bottom: var(--spacing-lg);
}
</style>
