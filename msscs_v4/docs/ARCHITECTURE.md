# MSSCS v4.0 Architecture

## System Overview

MSSCS (Multi-State Chain-based Secure Storage) v4.0 is a distributed storage system that combines encryption, compression, and blockchain-inspired data integrity mechanisms.

## Core Components

### 1. DataBlock Module (`src/block.rs`)

The fundamental unit of storage in MSSCS. Each block contains:

- **UUID**: Unique identifier
- **Node Index**: Position in the chain (0 = genesis)
- **Previous UUID**: Link to previous block
- **Previous Hash**: SHA-256 hash of previous block
- **Nonce**: 12-byte random value for AES-GCM
- **Encrypted Payload**: The actual data after encoding, compression, and encryption

#### Data Processing Pipeline

```
Original Data
    ↓
Base-16 Encoding (nibble splitting)
    ↓
Huffman Compression
    ↓
AES-256-GCM Encryption
    ↓
DataBlock
```

#### Decoding Pipeline

```
DataBlock
    ↓
AES-256-GCM Decryption
    ↓
Huffman Decompression
    ↓
Base-16 Decoding (nibble combining)
    ↓
Original Data
```

### 2. Huffman Compression Module (`src/huffman.rs`)

Implements variable-length encoding for efficient compression:

- **HuffmanNode**: Tree structure (Leaf or Internal nodes)
- **BitWriter**: Bit-level writing for compressed data
- **BitReader**: Bit-level reading for decompression
- **Tree Serialization**: Pre-order traversal for tree storage

The compression format:
```
[Tree Size: 4 bytes][Serialized Tree][Data Length: 4 bytes][Compressed Data]
```

### 3. Virtual File System (`src/vfs.rs`)

Manages file-to-block mapping and provides a file-like interface:

- **File Manifest**: Maps file paths to first block UUID
- **Block Chain**: Files are split into chunks, each chunk becomes a block
- **Chain Traversal**: Follows previous_uuid links to reconstruct files
- **Local Cache**: Stores blocks in memory for fast access

#### File Write Process

```
1. Split file into chunks (config.chunk_size)
2. Create blocks in reverse order (last chunk first)
3. Link blocks with previous_uuid and previous_hash
4. Store blocks locally
5. Replicate to network peers
6. Update manifest with path → first_block_uuid
7. Persist manifest to disk
```

#### File Read Process

```
1. Look up first block UUID from manifest
2. Retrieve first block (local or network)
3. Decode block data
4. Follow previous_uuid chain to get all blocks
5. Concatenate decoded data
6. Return complete file
```

### 4. Network Module (`src/network.rs`)

Implements P2P communication using libp2p Kademlia DHT:

- **Message Types**: RequestBlock, ResponseBlock, StoreBlock, Ping, Pong
- **Node**: Manages local blocks and peer connections
- **DHT**: Distributed hash table for peer discovery
- **Replication**: Sends blocks to N closest peers
- **Retry Logic**: Exponential backoff for failed operations

#### Network Protocol

```
Message Format:
[Length: 4 bytes][Serialized Message]

Message Types:
- RequestBlock { uuid }
- ResponseBlock { block: Option<DataBlock> }
- StoreBlock { block }
- Ping { node_id }
- Pong { node_id }
```

### 5. Persistence Module (`src/persistence.rs`)

Handles disk I/O for blocks and metadata:

- **Directory Structure**:
  ```
  data_dir/
  ├── blocks/
  │   ├── {uuid}.block
  │   └── ...
  ├── manifest.json
  └── logs/
  ```

- **Block Storage**: Bincode serialization
- **Manifest Storage**: JSON format for human readability
- **Orphan Cleanup**: Removes blocks not referenced in manifest

### 6. API Module (`src/api.rs`)

REST API built with Axum framework:

#### Endpoints

- `POST /files` - Write file
- `GET /files/:path` - Read file
- `DELETE /files/:path` - Delete file
- `GET /files` - List files
- `GET /blocks/:uuid` - Get block info
- `GET /health` - Health check
- `GET /metrics` - System metrics

#### Authentication

Optional API key authentication via `X-API-Key` header.

### 7. Metrics Module (`src/metrics.rs`)

Tracks system performance:

- Block count
- Storage bytes
- Peer count
- Uptime
- Request success/failure rates

## Data Flow

### Writing a File

```
Client
  ↓ POST /files
API Handler
  ↓ decode base64
VFS.write_file()
  ↓ split into chunks
DataBlock.new() (for each chunk)
  ↓ encode → compress → encrypt
Persistence.save_block()
  ↓ write to disk
Node.replicate_block()
  ↓ send to peers
Network
```

### Reading a File

```
Client
  ↓ GET /files/:path
API Handler
  ↓
VFS.read_file()
  ↓ lookup manifest
VFS.get_block() (for each block in chain)
  ↓ check local → check disk → query network
DataBlock.decode()
  ↓ decrypt → decompress → decode
VFS (concatenate chunks)
  ↓ encode base64
Client
```

## Security Model

### Encryption

- **Algorithm**: AES-256-GCM (authenticated encryption)
- **Key Derivation**: SHA256(UUID || node_index)
- **Nonce**: Random 12-byte value per block
- **Authentication**: GCM provides integrity verification

### Data Integrity

- **Block Hashing**: SHA-256 of (uuid, node_index, previous_uuid, nonce, encrypted_payload)
- **Chain Verification**: Each block references previous block's hash
- **Tamper Detection**: Modified blocks break the chain

## Performance Considerations

### Compression Ratio

Base-16 encoding doubles data size, but Huffman compression typically achieves:
- Highly redundant data: 50-80% compression
- Random data: ~100% (no compression benefit)
- Typical text: 60-70% compression

### Network Overhead

- Replication factor N means N copies of each block
- DHT lookups add latency for block retrieval
- Retry logic ensures reliability at cost of latency

### Storage Overhead

- Each block has ~100 bytes of metadata
- Manifest grows linearly with file count
- Orphaned blocks cleaned up periodically

## Scalability

### Horizontal Scaling

- Add more nodes to increase storage capacity
- DHT automatically distributes load
- No single point of failure

### Limitations

- Manifest is stored per-node (not distributed)
- Large files create many blocks (overhead)
- Network latency affects read performance

## Future Enhancements

1. **Distributed Manifest**: Replicate manifest across nodes
2. **Block Deduplication**: Reuse identical blocks
3. **Erasure Coding**: Reduce storage overhead vs replication
4. **Caching Layer**: LRU cache for frequently accessed blocks
5. **Compression Tuning**: Adaptive chunk sizes based on data type
