// Persistence module
use crate::block::DataBlock;
use crate::error::{MSSCSError, Result};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

/// Manages disk I/O for blocks, manifest, and configuration
pub struct PersistenceManager {
    data_dir: PathBuf,
}

impl PersistenceManager {
    /// Create new persistence manager and initialize directory structure
    pub fn new(data_dir: PathBuf) -> Result<Self> {
        // Create data directory structure
        fs::create_dir_all(&data_dir)?;
        fs::create_dir_all(data_dir.join("blocks"))?;
        fs::create_dir_all(data_dir.join("logs"))?;
        
        Ok(PersistenceManager { data_dir })
    }

    /// Save block to disk
    pub fn save_block(&self, block: &DataBlock) -> Result<()> {
        let filename = format!("{}.block", block.uuid);
        let path = self.data_dir.join("blocks").join(filename);
        
        let serialized = bincode::serialize(block)?;
        fs::write(path, serialized)?;
        
        Ok(())
    }

    /// Load block from disk
    pub fn load_block(&self, uuid: &Uuid) -> Result<DataBlock> {
        let filename = format!("{}.block", uuid);
        let path = self.data_dir.join("blocks").join(filename);
        
        let data = fs::read(&path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                MSSCSError::NotFound(format!("Block {} not found on disk", uuid))
            } else {
                MSSCSError::Io(e)
            }
        })?;
        
        let block: DataBlock = bincode::deserialize(&data)?;
        Ok(block)
    }

    /// Load all blocks from disk
    pub fn load_all_blocks(&self) -> Result<HashMap<String, DataBlock>> {
        let mut blocks = HashMap::new();
        let blocks_dir = self.data_dir.join("blocks");
        
        if !blocks_dir.exists() {
            return Ok(blocks);
        }
        
        for entry in fs::read_dir(blocks_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("block") {
                let data = fs::read(&path)?;
                if let Ok(block) = bincode::deserialize::<DataBlock>(&data) {
                    blocks.insert(block.uuid.to_string(), block);
                }
            }
        }
        
        Ok(blocks)
    }

    /// Delete block from disk
    pub fn delete_block(&self, uuid: &Uuid) -> Result<()> {
        let filename = format!("{}.block", uuid);
        let path = self.data_dir.join("blocks").join(filename);
        
        if path.exists() {
            fs::remove_file(path)?;
        }
        
        Ok(())
    }

    /// Save manifest to disk
    pub fn save_manifest(&self, manifest: &HashMap<String, Uuid>) -> Result<()> {
        let path = self.data_dir.join("manifest.json");
        let json = serde_json::to_string_pretty(manifest)
            .map_err(|e| MSSCSError::Config(format!("Failed to serialize manifest: {}", e)))?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Load manifest from disk
    pub fn load_manifest(&self) -> Result<HashMap<String, Uuid>> {
        let path = self.data_dir.join("manifest.json");
        
        if !path.exists() {
            return Ok(HashMap::new());
        }
        
        let data = fs::read_to_string(&path)?;
        let manifest: HashMap<String, Uuid> = serde_json::from_str(&data)
            .map_err(|e| MSSCSError::Config(format!("Failed to parse manifest: {}", e)))?;
        
        Ok(manifest)
    }

    /// Clean up orphaned blocks not referenced in manifest
    pub fn cleanup_orphaned_blocks(&self, manifest: &HashMap<String, Uuid>) -> Result<usize> {
        let blocks_dir = self.data_dir.join("blocks");
        let mut deleted_count = 0;
        
        if !blocks_dir.exists() {
            return Ok(0);
        }
        
        // Collect all UUIDs referenced in manifest (including chain links)
        let mut referenced_uuids = std::collections::HashSet::new();
        for first_uuid in manifest.values() {
            referenced_uuids.insert(first_uuid.to_string());
            
            // Follow the chain to collect all linked blocks
            let mut current_uuid = *first_uuid;
            while let Ok(block) = self.load_block(&current_uuid) {
                if let Some(prev_uuid) = block.previous_uuid {
                    referenced_uuids.insert(prev_uuid.to_string());
                    current_uuid = prev_uuid;
                } else {
                    break;
                }
            }
        }
        
        // Delete blocks not in referenced set
        for entry in fs::read_dir(blocks_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("block") {
                if let Some(filename) = path.file_stem().and_then(|s| s.to_str()) {
                    if !referenced_uuids.contains(filename) {
                        fs::remove_file(&path)?;
                        deleted_count += 1;
                    }
                }
            }
        }
        
        Ok(deleted_count)
    }

    // FileMetadata methods removed - will be implemented when FileMetadata struct is added

    /// List all file metadata
    pub fn list_file_metadata(&self) -> Result<Vec<String>> {
        let metadata_dir = self.data_dir.join("metadata");

        if !metadata_dir.exists() {
            return Ok(Vec::new());
        }

        let mut file_ids = Vec::new();
        for entry in fs::read_dir(metadata_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("meta") {
                if let Some(filename) = path.file_stem().and_then(|s| s.to_str()) {
                    file_ids.push(filename.to_string());
                }
            }
        }

        Ok(file_ids)
    }
}
