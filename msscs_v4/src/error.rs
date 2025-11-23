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

    #[error("JSON error: {0}")]
    Json(String),

    #[error("UUID error: {0}")]
    Uuid(String),
}

impl From<serde_json::Error> for MSSCSError {
    fn from(err: serde_json::Error) -> Self {
        MSSCSError::Json(err.to_string())
    }
}

impl From<uuid::Error> for MSSCSError {
    fn from(err: uuid::Error) -> Self {
        MSSCSError::Uuid(err.to_string())
    }
}

/// Result type alias for MSSCS operations
pub type Result<T> = std::result::Result<T, MSSCSError>;
