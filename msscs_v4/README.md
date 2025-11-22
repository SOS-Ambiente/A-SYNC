# MSSCS v4.0 - Multi-State Chain-based Secure Storage

A distributed, encrypted, and compressed storage system with blockchain-inspired data integrity.

## Features

- **Distributed Storage**: P2P network with Kademlia DHT for decentralized data distribution
- **Encryption**: AES-256-GCM encryption with per-block derived keys
- **Compression**: Huffman coding for efficient storage
- **Data Integrity**: SHA-256 hashing with blockchain-style chain linking
- **Base-16 Encoding**: Enhanced compression through nibble-level encoding
- **REST API**: Simple HTTP API for file operations
- **Persistence**: Automatic disk persistence with crash recovery
- **Metrics**: Built-in monitoring and health checks

## Quick Start

### Installation

```bash
# Clone the repository
git clone <repository-url>
cd msscs_v4

# Build the project
cargo build --release

# Run the server
cargo run --release
```

### Basic Usage

```bash
# Start a node on default port (8080)
./target/release/msscs_v4

# Start with custom port
./target/release/msscs_v4 --port 9090

# Start with bootstrap peers
./target/release/msscs_v4 --peer 127.0.0.1:8080 --peer 127.0.0.1:8081

# Use custom config file
./target/release/msscs_v4 --config my-config.toml
```

### API Examples

#### Write a file
```bash
curl -X POST http://localhost:8080/files \
  -H "Content-Type: application/json" \
  -d '{
    "path": "test.txt",
    "content": "SGVsbG8sIE1TU0NTIHY0LjAh"
  }'
```

#### Read a file
```bash
curl http://localhost:8080/files/test.txt
```

#### List files
```bash
curl http://localhost:8080/files
```

#### Delete a file
```bash
curl -X DELETE http://localhost:8080/files/test.txt
```

#### Health check
```bash
curl http://localhost:8080/health
```

#### Metrics
```bash
curl http://localhost:8080/metrics
```

## Architecture

MSSCS v4.0 uses a multi-layered approach to secure and distribute data:

1. **Base-16 Encoding**: Data is split into nibbles (4-bit values) for better compression
2. **Huffman Compression**: Variable-length encoding reduces storage size
3. **AES-256-GCM Encryption**: Each block is encrypted with a unique derived key
4. **Blockchain Linking**: Blocks are chained together with cryptographic hashes
5. **Distributed Storage**: Blocks are replicated across multiple nodes via DHT

## Configuration

Copy `config.toml.example` to `config.toml` and adjust settings:

```toml
port = 8080
data_dir = "./msscs_data"
replication_factor = 3
chunk_size = 1024
log_level = "info"
bootstrap_peers = []
api_keys = []  # Optional authentication
```

## Testing

```bash
# Run unit tests
cargo test

# Run integration tests
cargo test --test integration_test

# Run specific test
cargo test test_block_encode_decode_roundtrip
```

## Development

### Project Structure

```
msscs_v4/
├── src/
│   ├── main.rs          # Entry point and CLI
│   ├── lib.rs           # Public API exports
│   ├── error.rs         # Error types
│   ├── config.rs        # Configuration management
│   ├── block.rs         # DataBlock implementation
│   ├── huffman.rs       # Compression algorithms
│   ├── persistence.rs   # Disk I/O
│   ├── network.rs       # P2P networking
│   ├── vfs.rs           # Virtual file system
│   ├── api.rs           # REST API handlers
│   └── metrics.rs       # Monitoring
├── tests/               # Integration tests
├── docs/                # Documentation
└── Cargo.toml           # Dependencies
```

## License

GNU V3