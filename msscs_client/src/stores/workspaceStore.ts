import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'

export interface Workspace {
  id: string
  name: string
  description: string
  owner_id: string
  created_at: number
  members: WorkspaceMember[]
  shared_folders: SharedFolder[]
}

export interface WorkspaceMember {
  user_id: string
  email: string
  permission: 'Viewer' | 'Editor' | 'Admin' | 'Owner'
  joined_at: number
  last_active: number
}

export interface SharedFolder {
  id: string
  name: string
  path: string
  owner_id: string
  created_at: number
  members: Record<string, 'Viewer' | 'Editor' | 'Admin' | 'Owner'>
  files: Record<string, string>
}

export interface WorkspaceInvite {
  id: string
  workspace_id: string
  email: string
  permission: 'Viewer' | 'Editor' | 'Admin' | 'Owner'
  invited_by: string
  created_at: number
  expires_at: number
  accepted: boolean
}

export const useWorkspaceStore = defineStore('workspace', () => {
  const workspaces = ref<Workspace[]>([])
  const currentWorkspace = ref<Workspace | null>(null)
  const loading = ref(false)

  const loadWorkspaces = async () => {
    loading.value = true
    try {
      const list = await invoke<Workspace[]>('list_workspaces')
      workspaces.value = list
      console.log(`✅ Carregados ${list.length} workspaces`)
    } catch (error) {
      console.error('❌ Erro ao carregar workspaces:', error)
      workspaces.value = []
    } finally {
      loading.value = false
    }
  }

  const createWorkspace = async (name: string, description: string) => {
    try {
      const workspaceId = await invoke<string>('create_workspace', {
        name,
        description
      })
      console.log('✅ Workspace criado:', workspaceId)
      await loadWorkspaces()
      return workspaceId
    } catch (error) {
      console.error('❌ Erro ao criar workspace:', error)
      throw error
    }
  }

  const inviteMember = async (workspaceId: string, email: string, permission: string) => {
    try {
      const inviteId = await invoke<string>('invite_workspace_member', {
        workspaceId,
        email,
        permission
      })
      console.log('📧 Convite enviado:', inviteId)
      return inviteId
    } catch (error) {
      console.error('❌ Erro ao enviar convite:', error)
      throw error
    }
  }

  const acceptInvite = async (workspaceId: string, inviteId: string) => {
    try {
      await invoke('accept_workspace_invite', {
        workspaceId,
        inviteId
      })
      console.log('✅ Convite aceito')
      await loadWorkspaces()
    } catch (error) {
      console.error('❌ Erro ao aceitar convite:', error)
      throw error
    }
  }

  const createSharedFolder = async (workspaceId: string, name: string, path: string) => {
    try {
      const folderId = await invoke<string>('create_shared_folder', {
        workspaceId,
        name,
        path
      })
      console.log('📁 Pasta compartilhada criada:', folderId)
      await loadWorkspaces()
      return folderId
    } catch (error) {
      console.error('❌ Erro ao criar pasta:', error)
      throw error
    }
  }

  const shareFolderWithMember = async (
    workspaceId: string,
    folderId: string,
    memberId: string,
    permission: string
  ) => {
    try {
      await invoke('share_folder_with_member', {
        workspaceId,
        folderId,
        memberId,
        permission
      })
      console.log('🤝 Pasta compartilhada com membro')
      await loadWorkspaces()
    } catch (error) {
      console.error('❌ Erro ao compartilhar pasta:', error)
      throw error
    }
  }

  const setCurrentWorkspace = (workspace: Workspace | null) => {
    currentWorkspace.value = workspace
  }

  return {
    workspaces,
    currentWorkspace,
    loading,
    loadWorkspaces,
    createWorkspace,
    inviteMember,
    acceptInvite,
    createSharedFolder,
    shareFolderWithMember,
    setCurrentWorkspace
  }
})
