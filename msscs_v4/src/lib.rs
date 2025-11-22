// MSSCS v4.0 - Multi-State Chain-based Secure Storage
// Public API exports

pub mod error;
pub mod config;
pub mod block;
pub mod huffman;
pub mod persistence;
pub mod network;
pub mod vfs;
pub mod api;
pub mod metrics;

// Re-export commonly used types
pub use block::DataBlock;
pub use error::{MSSCSError, Result};
pub use config::Config;
