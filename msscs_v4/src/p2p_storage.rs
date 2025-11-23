// P2P DISTRIBUTED STORAGE SYSTEM
// Cada host age como servidor de seus próprios arquivos e pastas compartilhadas

use crate::error::{MSSCSError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Informações de replicação de arquivo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileReplication {
    pub file_uuid: Uuid,
    pub file_path: String,
    pub owner_id: Uuid,
    pub workspace_id: Option<Uuid>,
    pub folder_id: Option<Uuid>,
    /// Hosts que possuem este arquivo
    pub hosts: Vec<HostInfo>,
    pub total_size: u64,
    pub created_at: u64,
    pub last_updated: u64,
}

/// Informações de um host na rede
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostInfo {
    pub peer_id: String,
    pub user_id: Uuid,
    pub online: bool,
    pub last_seen: u64,
    /// Espaço disponível para compartilhamento (bytes)
    pub available_space: u64,
    /// Espaço usado (bytes)
    pub used_space: u64,
    /// Latência média (ms)
    pub latency_ms: u32,
}

/// Estatísticas de armazenamento distribuído
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributedStorageStats {
    pub total_files: usize,
    pub total_size: u64,
    pub replicated_files: usize,
    pub online_hosts: usize,
    pub total_hosts: usize,
    pub network_capacity: u64,
    pub network_used: u64,
}

/// Gerenciador de armazenamento P2P distribuído
pub struct P2PStorageManager {
    /// Mapeamento de arquivos para hosts
    file_replications: Arc<RwLock<HashMap<Uuid, FileReplication>>>,
    /// Hosts conhecidos na rede
    hosts: Arc<RwLock<HashMap<String, HostInfo>>>,
    /// ID do host local
    local_peer_id: String,
    local_user_id: Uuid,
}

impl P2PStorageManager {
    pub fn new(local_peer_id: String, local_user_id: Uuid) -> Self {
        Self {
            file_replications: Arc::new(RwLock::new(HashMap::new())),
            hosts: Arc::new(RwLock::new(HashMap::new())),
            local_peer_id,
            local_user_id,
        }
    }
    
    /// Registrar arquivo no sistema distribuído
    pub async fn register_file(
        &self,
        file_uuid: Uuid,
        file_path: String,
        owner_id: Uuid,
        workspace_id: Option<Uuid>,
        folder_id: Option<Uuid>,
        size: u64,
    ) -> Result<()> {
        let now = current_timestamp();
        
        let hosts = vec![HostInfo {
            peer_id: self.local_peer_id.clone(),
            user_id: self.local_user_id,
            online: true,
            last_seen: now,
            available_space: 0, // Será atualizado
            used_space: 0,
            latency_ms: 0,
        }];
        
        let replication = FileReplication {
            file_uuid,
            file_path: file_path.clone(),
            owner_id,
            workspace_id,
            folder_id,
            hosts,
            total_size: size,
            created_at: now,
            last_updated: now,
        };
        
        self.file_replications.write().await.insert(file_uuid, replication);
        tracing::info!("📝 Arquivo {} registrado no sistema P2P", file_path);
        
        Ok(())
    }
    
    /// Adicionar host que possui o arquivo
    pub async fn add_file_host(&self, file_uuid: Uuid, host_info: HostInfo) -> Result<()> {
        let mut replications = self.file_replications.write().await;
        let replication = replications.get_mut(&file_uuid)
            .ok_or_else(|| MSSCSError::NotFound("Arquivo não encontrado".to_string()))?;
        
        // Verificar se host já existe
        if !replication.hosts.iter().any(|h| h.peer_id == host_info.peer_id) {
            replication.hosts.push(host_info.clone());
            replication.last_updated = current_timestamp();
            tracing::info!("🔗 Host {} adicionado para arquivo {}", host_info.peer_id, file_uuid);
        }
        
        Ok(())
    }
    
    /// Encontrar hosts que possuem o arquivo
    pub async fn find_file_hosts(&self, file_uuid: &Uuid) -> Result<Vec<HostInfo>> {
        let replications = self.file_replications.read().await;
        let replication = replications.get(file_uuid)
            .ok_or_else(|| MSSCSError::NotFound("Arquivo não encontrado".to_string()))?;
        
        // Filtrar apenas hosts online
        let online_hosts: Vec<HostInfo> = replication.hosts.iter()
            .filter(|h| h.online)
            .cloned()
            .collect();
        
        if online_hosts.is_empty() {
            return Err(MSSCSError::Network("Nenhum host online possui este arquivo".to_string()));
        }
        
        Ok(online_hosts)
    }
    
    /// Selecionar melhor host para download (menor latência)
    pub async fn select_best_host(&self, file_uuid: &Uuid) -> Result<HostInfo> {
        let hosts = self.find_file_hosts(file_uuid).await?;
        
        // Ordenar por latência
        let mut sorted_hosts = hosts;
        sorted_hosts.sort_by_key(|h| h.latency_ms);
        
        sorted_hosts.into_iter().next()
            .ok_or_else(|| MSSCSError::Network("Nenhum host disponível".to_string()))
    }
    
    /// Registrar host na rede
    pub async fn register_host(&self, host_info: HostInfo) -> Result<()> {
        self.hosts.write().await.insert(host_info.peer_id.clone(), host_info.clone());
        tracing::info!("🌐 Host {} registrado na rede", host_info.peer_id);
        Ok(())
    }
    
    /// Atualizar status de host
    pub async fn update_host_status(&self, peer_id: &str, online: bool) -> Result<()> {
        let mut hosts = self.hosts.write().await;
        if let Some(host) = hosts.get_mut(peer_id) {
            host.online = online;
            host.last_seen = current_timestamp();
            tracing::debug!("📡 Host {} status: {}", peer_id, if online { "online" } else { "offline" });
        }
        Ok(())
    }
    
    /// Obter estatísticas do armazenamento distribuído
    pub async fn get_stats(&self) -> DistributedStorageStats {
        let replications = self.file_replications.read().await;
        let hosts = self.hosts.read().await;
        
        let total_files = replications.len();
        let total_size: u64 = replications.values().map(|r| r.total_size).sum();
        let replicated_files = replications.values().filter(|r| r.hosts.len() > 1).count();
        
        let online_hosts = hosts.values().filter(|h| h.online).count();
        let total_hosts = hosts.len();
        
        let network_capacity: u64 = hosts.values().map(|h| h.available_space).sum();
        let network_used: u64 = hosts.values().map(|h| h.used_space).sum();
        
        DistributedStorageStats {
            total_files,
            total_size,
            replicated_files,
            online_hosts,
            total_hosts,
            network_capacity,
            network_used,
        }
    }
    
    /// Listar arquivos de um workspace
    pub async fn list_workspace_files(&self, workspace_id: &Uuid) -> Vec<FileReplication> {
        let replications = self.file_replications.read().await;
        replications.values()
            .filter(|r| r.workspace_id.as_ref() == Some(workspace_id))
            .cloned()
            .collect()
    }
    
    /// Listar arquivos de uma pasta compartilhada
    pub async fn list_folder_files(&self, folder_id: &Uuid) -> Vec<FileReplication> {
        let replications = self.file_replications.read().await;
        replications.values()
            .filter(|r| r.folder_id.as_ref() == Some(folder_id))
            .cloned()
            .collect()
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
