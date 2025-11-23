// MSSCS v4.0 - Multi-State Chain-based Secure Storage
// Public API exports

pub mod error;
pub mod config;
pub mod block;
pub mod huffman;
pub mod identity;
pub mod unlocked_identity;
pub mod persistence;
pub mod network;
pub mod p2p_network;
pub mod webrtc_bridge;
pub mod vfs;
pub mod api;
pub mod metrics;
pub mod workspace;
pub mod p2p_storage;

// Re-export commonly used types
pub use block::{DataBlock, calculate_checksum};
pub use error::{MSSCSError, Result};
pub use config::Config;
pub use identity::{QuantumIdentity, IdentityManager, ReputationTier};
pub use unlocked_identity::UnlockedIdentity;
pub use p2p_network::{P2PNode, P2PConfig, P2PEvent};
