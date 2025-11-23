// Error handling module
use thiserror::Error;

/// Custom error types for MSSCS system
#[derive(Debug, Error)]
pub enum MSSCSError {
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Cryptography error: {0}")]
    Crypto(String),
    
    #[error("Compression error: {0}")]
    Compression(String),
    
    #[error("Block not found: {0}")]
    NotFound(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Invalid data: {0}")]
    InvalidData(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Corrupted data: {0}")]
    CorruptedData(String),

    #[error("Encryption error: {0}")]
    Encryption(String),
}

/// Result type alias for MSSCS operations
pub type Result<T> = std::result::Result<T, MSSCSError>;
