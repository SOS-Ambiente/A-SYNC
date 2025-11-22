// Integration tests for MSSCS v4.0
use msscs_v4::Config;
use msscs_v4::vfs::VirtualFileSystem;
use msscs_v4::persistence::PersistenceManager;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;

#[tokio::test]
async fn test_vfs_write_and_read_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let data_dir = temp_dir.path().to_path_buf();
    
    let config = Arc::new(Config {
        port: 8080,
        data_dir: data_dir.clone(),
        replication_factor: 1,
        chunk_size: 1024,
        log_level: "info".to_string(),
        bootstrap_peers: vec![],
        api_keys: None,
    });
    
    let persistence = Arc::new(PersistenceManager::new(data_dir).expect("Failed to create persistence"));
    let mut vfs = VirtualFileSystem::new(config, persistence).expect("Failed to create VFS");
    
    // Write file
    let test_data = b"Hello, MSSCS v4.0!";
    let path = PathBuf::from("test.txt");
    vfs.write_file(&path, test_data).await.expect("Failed to write file");
    
    // Read file
    let read_data = vfs.read_file(&path).await.expect("Failed to read file");
    
    assert_eq!(test_data, read_data.as_slice());
}

#[tokio::test]
async fn test_vfs_write_large_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let data_dir = temp_dir.path().to_path_buf();
    
    let config = Arc::new(Config {
        port: 8080,
        data_dir: data_dir.clone(),
        replication_factor: 1,
        chunk_size: 1024,
        log_level: "info".to_string(),
        bootstrap_peers: vec![],
        api_keys: None,
    });
    
    let persistence = Arc::new(PersistenceManager::new(data_dir).expect("Failed to create persistence"));
    let mut vfs = VirtualFileSystem::new(config, persistence).expect("Failed to create VFS");
    
    // Create 10KB file
    let test_data = vec![0xAB; 10240];
    let path = PathBuf::from("large.bin");
    vfs.write_file(&path, &test_data).await.expect("Failed to write large file");
    
    // Read file
    let read_data = vfs.read_file(&path).await.expect("Failed to read large file");
    
    assert_eq!(test_data, read_data);
    assert_eq!(read_data.len(), 10240);
}

#[tokio::test]
async fn test_vfs_list_files() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let data_dir = temp_dir.path().to_path_buf();
    
    let config = Arc::new(Config {
        port: 8080,
        data_dir: data_dir.clone(),
        replication_factor: 1,
        chunk_size: 1024,
        log_level: "info".to_string(),
        bootstrap_peers: vec![],
        api_keys: None,
    });
    
    let persistence = Arc::new(PersistenceManager::new(data_dir).expect("Failed to create persistence"));
    let mut vfs = VirtualFileSystem::new(config, persistence).expect("Failed to create VFS");
    
    // Write multiple files
    vfs.write_file(&PathBuf::from("file1.txt"), b"data1").await.expect("Failed to write file1");
    vfs.write_file(&PathBuf::from("file2.txt"), b"data2").await.expect("Failed to write file2");
    vfs.write_file(&PathBuf::from("file3.txt"), b"data3").await.expect("Failed to write file3");
    
    // List files
    let files = vfs.list_files();
    
    assert_eq!(files.len(), 3);
    assert!(files.contains(&"file1.txt".to_string()));
    assert!(files.contains(&"file2.txt".to_string()));
    assert!(files.contains(&"file3.txt".to_string()));
}

#[tokio::test]
async fn test_vfs_delete_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let data_dir = temp_dir.path().to_path_buf();
    
    let config = Arc::new(Config {
        port: 8080,
        data_dir: data_dir.clone(),
        replication_factor: 1,
        chunk_size: 1024,
        log_level: "info".to_string(),
        bootstrap_peers: vec![],
        api_keys: None,
    });
    
    let persistence = Arc::new(PersistenceManager::new(data_dir).expect("Failed to create persistence"));
    let mut vfs = VirtualFileSystem::new(config, persistence).expect("Failed to create VFS");
    
    // Write file
    let path = PathBuf::from("delete_me.txt");
    vfs.write_file(&path, b"temporary data").await.expect("Failed to write file");
    
    // Verify file exists
    assert_eq!(vfs.list_files().len(), 1);
    
    // Delete file
    vfs.delete_file(&path).await.expect("Failed to delete file");
    
    // Verify file is gone from manifest
    assert_eq!(vfs.list_files().len(), 0);
}

#[tokio::test]
async fn test_vfs_persistence() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let data_dir = temp_dir.path().to_path_buf();
    
    let config = Arc::new(Config {
        port: 8080,
        data_dir: data_dir.clone(),
        replication_factor: 1,
        chunk_size: 1024,
        log_level: "info".to_string(),
        bootstrap_peers: vec![],
        api_keys: None,
    });
    
    // Create VFS and write file
    {
        let persistence = Arc::new(PersistenceManager::new(data_dir.clone()).expect("Failed to create persistence"));
        let mut vfs = VirtualFileSystem::new(config.clone(), persistence).expect("Failed to create VFS");
        
        vfs.write_file(&PathBuf::from("persistent.txt"), b"persistent data").await.expect("Failed to write file");
    }
    
    // Create new VFS instance (simulating restart)
    {
        let persistence = Arc::new(PersistenceManager::new(data_dir.clone()).expect("Failed to create persistence"));
        let mut vfs = VirtualFileSystem::new(config.clone(), persistence).expect("Failed to create VFS");
        
        // File should still be accessible
        let files = vfs.list_files();
        assert_eq!(files.len(), 1);
        assert!(files.contains(&"persistent.txt".to_string()));
        
        // Read file
        let data = vfs.read_file(&PathBuf::from("persistent.txt")).await.expect("Failed to read file");
        assert_eq!(data, b"persistent data");
    }
}
