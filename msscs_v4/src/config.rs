// Configuration module
use crate::error::{MSSCSError, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// System configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub port: u16,
    pub data_dir: PathBuf,
    pub replication_factor: usize,
    pub chunk_size: usize,
    pub log_level: String,
    pub bootstrap_peers: Vec<String>,
    pub api_keys: Option<Vec<String>>,
}

impl Config {
    /// Load configuration from TOML file
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)
            .map_err(|e| MSSCSError::Config(format!("Failed to parse config: {}", e)))?;
        config.validate()?;
        Ok(config)
    }
    
    /// Save configuration to TOML file
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| MSSCSError::Config(format!("Failed to serialize config: {}", e)))?;
        std::fs::write(path, content)?;
        Ok(())
    }
    
    /// Create default configuration
    pub fn default() -> Self {
        Config {
            port: 8080,
            data_dir: PathBuf::from("./msscs_data"),
            replication_factor: 3,
            chunk_size: 1024,
            log_level: "info".to_string(),
            bootstrap_peers: Vec::new(),
            api_keys: None,
        }
    }
    
    /// Validate configuration values
    pub fn validate(&self) -> Result<()> {
        if self.port == 0 {
            return Err(MSSCSError::Config("Port cannot be 0".to_string()));
        }
        
        if self.replication_factor == 0 {
            return Err(MSSCSError::Config("Replication factor must be at least 1".to_string()));
        }
        
        if self.chunk_size == 0 {
            return Err(MSSCSError::Config("Chunk size must be greater than 0".to_string()));
        }
        
        let valid_log_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            return Err(MSSCSError::Config(format!(
                "Invalid log level '{}'. Must be one of: {:?}",
                self.log_level, valid_log_levels
            )));
        }
        
        Ok(())
    }
}
