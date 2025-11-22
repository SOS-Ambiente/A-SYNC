// Unit tests for Config module
use msscs_v4::Config;
use tempfile::TempDir;

#[test]
fn test_default_config() {
    let config = Config::default();
    
    assert_eq!(config.port, 8080);
    assert_eq!(config.replication_factor, 3);
    assert_eq!(config.chunk_size, 1024);
    assert_eq!(config.log_level, "info");
    assert!(config.bootstrap_peers.is_empty());
    assert!(config.api_keys.is_none());
}

#[test]
fn test_config_validation() {
    let mut config = Config::default();
    
    // Valid config should pass
    assert!(config.validate().is_ok());
    
    // Invalid port
    config.port = 0;
    assert!(config.validate().is_err());
    config.port = 8080;
    
    // Invalid replication factor
    config.replication_factor = 0;
    assert!(config.validate().is_err());
    config.replication_factor = 3;
    
    // Invalid chunk size
    config.chunk_size = 0;
    assert!(config.validate().is_err());
    config.chunk_size = 1024;
    
    // Invalid log level
    config.log_level = "invalid".to_string();
    assert!(config.validate().is_err());
}

#[test]
fn test_config_save_and_load() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("config.toml");
    
    // Create and save config
    let mut config = Config::default();
    config.port = 9090;
    config.bootstrap_peers = vec!["127.0.0.1:8080".to_string()];
    config.api_keys = Some(vec!["test-key".to_string()]);
    
    config.save(&config_path).expect("Failed to save config");
    
    // Load config
    let loaded_config = Config::load(&config_path).expect("Failed to load config");
    
    assert_eq!(loaded_config.port, 9090);
    assert_eq!(loaded_config.bootstrap_peers, vec!["127.0.0.1:8080"]);
    assert_eq!(loaded_config.api_keys, Some(vec!["test-key".to_string()]));
}
