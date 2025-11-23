// WORKSPACE COLLABORATION SYSTEM
// Sistema de workspaces colaborativos com compartilhamento P2P

use crate::error::{MSSCSError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Níveis de permissão em um workspace
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Permission {
    /// Apenas visualização
    Viewer,
    /// Pode editar arquivos
    Editor,
    /// Controle total (adicionar/remover membros)
    Admin,
    /// Dono do workspace
    Owner,
}

/// Convite para workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceInvite {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub email: String,
    pub permission: Permission,
    pub invited_by: Uuid,
    pub created_at: u64,
    pub expires_at: u64,
    pub accepted: bool,
}

/// Membro de um workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceMember {
    pub user_id: Uuid,
    pub email: String,
    pub permission: Permission,
    pub joined_at: u64,
    pub last_active: u64,
}

/// Pasta compartilhada dentro de um workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedFolder {
    pub id: Uuid,
    pub name: String,
    pub path: String,
    pub owner_id: Uuid,
    pub created_at: u64,
    /// Membros com acesso específico a esta pasta
    pub members: HashMap<Uuid, Permission>,
    /// Arquivos nesta pasta (path -> file_uuid)
    pub files: HashMap<String, Uuid>,
}

/// Workspace colaborativo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub owner_id: Uuid,
    pub created_at: u64,
    pub members: HashMap<Uuid, WorkspaceMember>,
    pub shared_folders: HashMap<Uuid, SharedFolder>,
    pub invites: HashMap<Uuid, WorkspaceInvite>,
}

impl Workspace {
    /// Criar novo workspace
    pub fn new(name: String, description: String, owner_id: Uuid, owner_email: String) -> Self {
        let id = Uuid::new_v4();
        let now = current_timestamp();
        
        let mut members = HashMap::new();
        members.insert(owner_id, WorkspaceMember {
            user_id: owner_id,
            email: owner_email,
            permission: Permission::Owner,
            joined_at: now,
            last_active: now,
        });
        
        Self {
            id,
            name,
            description,
            owner_id,
            created_at: now,
            members,
            shared_folders: HashMap::new(),
            invites: HashMap::new(),
        }
    }
    
    /// Convidar usuário por email
    pub fn invite_member(&mut self, email: String, permission: Permission, invited_by: Uuid) -> Result<Uuid> {
        // Verificar se quem convida tem permissão
        if !self.can_invite(&invited_by) {
            return Err(MSSCSError::PermissionDenied(
                "Você não tem permissão para convidar membros".to_string()
            ));
        }
        
        let invite_id = Uuid::new_v4();
        let now = current_timestamp();
        
        let invite = WorkspaceInvite {
            id: invite_id,
            workspace_id: self.id,
            email: email.clone(),
            permission,
            invited_by,
            created_at: now,
            expires_at: now + (7 * 24 * 60 * 60), // 7 dias
            accepted: false,
        };
        
        self.invites.insert(invite_id, invite);
        tracing::info!("📧 Convite enviado para {} no workspace {}", email, self.name);
        
        Ok(invite_id)
    }
    
    /// Aceitar convite
    pub fn accept_invite(&mut self, invite_id: Uuid, user_id: Uuid, email: String) -> Result<()> {
        let invite = self.invites.get_mut(&invite_id)
            .ok_or_else(|| MSSCSError::NotFound("Convite não encontrado".to_string()))?;
        
        // Verificar se o email corresponde
        if invite.email != email {
            return Err(MSSCSError::PermissionDenied("Email não corresponde ao convite".to_string()));
        }
        
        // Verificar se não expirou
        if current_timestamp() > invite.expires_at {
            return Err(MSSCSError::InvalidData("Convite expirado".to_string()));
        }
        
        // Adicionar como membro
        let now = current_timestamp();
        self.members.insert(user_id, WorkspaceMember {
            user_id,
            email: email.clone(),
            permission: invite.permission,
            joined_at: now,
            last_active: now,
        });
        
        invite.accepted = true;
        tracing::info!("✅ {} entrou no workspace {}", email, self.name);
        
        Ok(())
    }
    
    /// Criar pasta compartilhada
    pub fn create_shared_folder(&mut self, name: String, path: String, owner_id: Uuid) -> Result<Uuid> {
        // Verificar permissão
        if !self.can_create_folder(&owner_id) {
            return Err(MSSCSError::PermissionDenied(
                "Você não tem permissão para criar pastas".to_string()
            ));
        }
        
        let folder_id = Uuid::new_v4();
        let folder = SharedFolder {
            id: folder_id,
            name: name.clone(),
            path: path.clone(),
            owner_id,
            created_at: current_timestamp(),
            members: HashMap::new(),
            files: HashMap::new(),
        };
        
        self.shared_folders.insert(folder_id, folder);
        tracing::info!("📁 Pasta compartilhada criada: {} em {}", name, self.name);
        
        Ok(folder_id)
    }
    
    /// Adicionar arquivo a pasta compartilhada
    pub fn add_file_to_folder(&mut self, folder_id: Uuid, file_path: String, file_uuid: Uuid, user_id: Uuid) -> Result<()> {
        // Verificar permissão primeiro (antes de pegar referência mutável)
        {
            let folder = self.shared_folders.get(&folder_id)
                .ok_or_else(|| MSSCSError::NotFound("Pasta não encontrada".to_string()))?;
            
            if !self.can_write_to_folder(folder, &user_id) {
                return Err(MSSCSError::PermissionDenied(
                    "Você não tem permissão para adicionar arquivos nesta pasta".to_string()
                ));
            }
        }
        
        // Agora podemos pegar referência mutável
        let folder = self.shared_folders.get_mut(&folder_id).unwrap();
        folder.files.insert(file_path.clone(), file_uuid);
        tracing::info!("📄 Arquivo {} adicionado à pasta {}", file_path, folder.name);
        
        Ok(())
    }
    
    /// Compartilhar pasta com membro
    pub fn share_folder_with_member(&mut self, folder_id: Uuid, member_id: Uuid, permission: Permission, requester_id: Uuid) -> Result<()> {
        // Verificar se o membro existe no workspace
        if !self.members.contains_key(&member_id) {
            return Err(MSSCSError::NotFound("Membro não encontrado no workspace".to_string()));
        }
        
        // Verificar permissões primeiro (antes de pegar referência mutável)
        {
            let folder = self.shared_folders.get(&folder_id)
                .ok_or_else(|| MSSCSError::NotFound("Pasta não encontrada".to_string()))?;
            
            if folder.owner_id != requester_id && !self.is_admin(&requester_id) {
                return Err(MSSCSError::PermissionDenied(
                    "Apenas o dono da pasta ou admins podem compartilhar".to_string()
                ));
            }
        }
        
        // Agora podemos pegar referência mutável
        let folder = self.shared_folders.get_mut(&folder_id).unwrap();
        folder.members.insert(member_id, permission);
        tracing::info!("🤝 Pasta {} compartilhada com membro {}", folder.name, member_id);
        
        Ok(())
    }
    
    /// Verificar se usuário pode convidar
    fn can_invite(&self, user_id: &Uuid) -> bool {
        self.members.get(user_id)
            .map(|m| matches!(m.permission, Permission::Admin | Permission::Owner))
            .unwrap_or(false)
    }
    
    /// Verificar se usuário pode criar pastas
    fn can_create_folder(&self, user_id: &Uuid) -> bool {
        self.members.get(user_id)
            .map(|m| matches!(m.permission, Permission::Editor | Permission::Admin | Permission::Owner))
            .unwrap_or(false)
    }
    
    /// Verificar se usuário pode escrever em pasta
    fn can_write_to_folder(&self, folder: &SharedFolder, user_id: &Uuid) -> bool {
        // Dono da pasta sempre pode
        if folder.owner_id == *user_id {
            return true;
        }
        
        // Verificar permissão específica na pasta
        if let Some(perm) = folder.members.get(user_id) {
            return matches!(perm, Permission::Editor | Permission::Admin | Permission::Owner);
        }
        
        // Verificar permissão geral no workspace
        self.members.get(user_id)
            .map(|m| matches!(m.permission, Permission::Editor | Permission::Admin | Permission::Owner))
            .unwrap_or(false)
    }
    
    /// Verificar se é admin
    fn is_admin(&self, user_id: &Uuid) -> bool {
        self.members.get(user_id)
            .map(|m| matches!(m.permission, Permission::Admin | Permission::Owner))
            .unwrap_or(false)
    }
}

/// Gerenciador de workspaces
pub struct WorkspaceManager {
    workspaces: Arc<RwLock<HashMap<Uuid, Workspace>>>,
    user_workspaces: Arc<RwLock<HashMap<Uuid, Vec<Uuid>>>>, // user_id -> workspace_ids
}

impl WorkspaceManager {
    pub fn new() -> Self {
        Self {
            workspaces: Arc::new(RwLock::new(HashMap::new())),
            user_workspaces: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Criar workspace
    pub async fn create_workspace(&self, name: String, description: String, owner_id: Uuid, owner_email: String) -> Result<Uuid> {
        let workspace = Workspace::new(name, description, owner_id, owner_email);
        let workspace_id = workspace.id;
        
        self.workspaces.write().await.insert(workspace_id, workspace);
        
        // Adicionar aos workspaces do usuário
        self.user_workspaces.write().await
            .entry(owner_id)
            .or_insert_with(Vec::new)
            .push(workspace_id);
        
        tracing::info!("🏢 Workspace criado: {}", workspace_id);
        Ok(workspace_id)
    }
    
    /// Listar workspaces do usuário
    pub async fn list_user_workspaces(&self, user_id: &Uuid) -> Vec<Workspace> {
        let user_ws = self.user_workspaces.read().await;
        let workspace_ids = user_ws.get(user_id).cloned().unwrap_or_default();
        
        let workspaces = self.workspaces.read().await;
        workspace_ids.iter()
            .filter_map(|id| workspaces.get(id).cloned())
            .collect()
    }
    
    /// Obter workspace
    pub async fn get_workspace(&self, workspace_id: &Uuid) -> Option<Workspace> {
        self.workspaces.read().await.get(workspace_id).cloned()
    }
    
    /// Convidar membro
    pub async fn invite_member(&self, workspace_id: Uuid, email: String, permission: Permission, invited_by: Uuid) -> Result<Uuid> {
        let mut workspaces = self.workspaces.write().await;
        let workspace = workspaces.get_mut(&workspace_id)
            .ok_or_else(|| MSSCSError::NotFound("Workspace não encontrado".to_string()))?;
        
        workspace.invite_member(email, permission, invited_by)
    }
    
    /// Aceitar convite
    pub async fn accept_invite(&self, workspace_id: Uuid, invite_id: Uuid, user_id: Uuid, email: String) -> Result<()> {
        let mut workspaces = self.workspaces.write().await;
        let workspace = workspaces.get_mut(&workspace_id)
            .ok_or_else(|| MSSCSError::NotFound("Workspace não encontrado".to_string()))?;
        
        workspace.accept_invite(invite_id, user_id, email)?;
        
        // Adicionar workspace à lista do usuário
        drop(workspaces);
        self.user_workspaces.write().await
            .entry(user_id)
            .or_insert_with(Vec::new)
            .push(workspace_id);
        
        Ok(())
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
