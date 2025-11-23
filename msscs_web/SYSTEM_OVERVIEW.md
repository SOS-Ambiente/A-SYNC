# MSSCS Web - System Overview

Complete technical overview of the MSSCS Web implementation.

## System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      Browser Environment                     │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌────────────────────────────────────────────────────┐    │
│  │              User Interface Layer                   │    │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐        │    │
│  │  │  Upload  │  │   Files  │  │  Peers   │        │    │
│  │  │   Area   │  │   List   │  │   List   │        │    │
│  │  └──────────┘  └──────────┘  └──────────┘        │    │
│  └────────────────────────────────────────────────────┘    │
│                          ↕                                   │
│  ┌────────────────────────────────────────────────────┐    │
│  │           Application Logic Layer                   │    │
│  │  ┌──────────────────────────────────────────┐     │    │
│  │  │  MSSCSWeb (app.js)                       │     │    │
│  │  │  - File management                       │     │    │
│  │  │  - Event coordination                    │     │    │
│  │  │  - UI updates                            │     │    │
│  │  └──────────────────────────────────────────┘     │    │
│  └────────────────────────────────────────────────────┘    │
│                          ↕                                   │
│  ┌────────────────────────────────────────────────────┐    │
│  │              Service Layer                          │    │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐        │    │
│  │  │ Storage  │  │  Crypto  │  │   P2P    │        │    │
│  │  │ Manager  │  │ Manager  │  │ Network  │        │    │
│  │  └──────────┘  └──────────┘  └──────────┘        │    │
│  └────────────────────────────────────────────────────┘    │
│                          ↕                                   │
│  ┌────────────────────────────────────────────────────┐    │
│  │              Browser APIs Layer                     │    │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐        │    │
│  │  │ IndexedDB│  │ Web      │  │ WebRTC   │        │    │
│  │  │          │  │ Crypto   │  │          │        │    │
│  │  └──────────┘  └──────────┘  └──────────┘        │    │
│  └────────────────────────────────────────────────────┘    │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## Data Flow

### Upload Flow

```
User selects file
       ↓
Read as ArrayBuffer
       ↓
Split into chunks (256KB)
       ↓
For each chunk:
  ├─→ Generate random IV (12 bytes)
  ├─→ Encrypt with AES-256-GCM
  ├─→ Calculate SHA-256 hash
  ├─→ Link to previous chunk
  ├─→ Create block object
  ├─→ Save to IndexedDB
  └─→ Broadcast to peers
       ↓
Create file metadata
       ↓
Save metadata to IndexedDB
       ↓
Update UI
```

### Download Flow

```
User clicks download
       ↓
Retrieve file metadata
       ↓
For each chunk ID:
  ├─→ Try local IndexedDB
  ├─→ If not found, request from peers
  ├─→ Wait for chunk
  └─→ Decrypt chunk
       ↓
Concatenate all chunks
       ↓
Create Blob
       ↓
Trigger browser download
```

### P2P Sync Flow

```
Peer A uploads file
       ↓
Chunks saved to IndexedDB
       ↓
Broadcast to all connected peers
       ↓
Peer B receives broadcast
       ↓
Check if chunk exists locally
       ↓
If not, save to IndexedDB
       ↓
Update UI
```

## Component Details

### 1. Storage Manager (storage.js)

**Purpose**: Manage persistent storage using IndexedDB

**Key Methods**:
```javascript
async init()                    // Initialize database
async saveBlock(block)          // Save encrypted block
async getBlock(blockId)         // Retrieve block
async deleteBlock(blockId)      // Delete block
async saveFile(metadata)        // Save file metadata
async getFile(fileId)           // Get file metadata
async getAllFiles()             // List all files
async deleteFile(fileId)        // Delete file metadata
async clear()                   // Clear all data
```

**Database Schema**:
```javascript
// Object Store: blocks
{
    keyPath: 'id',
    indexes: none
}

// Object Store: files
{
    keyPath: 'id',
    indexes: none
}
```

**Storage Quota**:
- Managed by browser
- Typically 50% of available disk space
- Can be checked via `navigator.storage.estimate()`

### 2. Crypto Manager (crypto.js)

**Purpose**: Handle all cryptographic operations

**Key Methods**:
```javascript
async init()                    // Initialize/load key
async encrypt(data)             // Encrypt data
async decrypt(data)             // Decrypt data
async hash(data)                // Calculate SHA-256
async deriveKey(password, salt) // Derive key from password
async exportKey()               // Export key as JWK
async importKey(keyData)        // Import key from JWK
clearKey()                      // Clear stored key
```

**Encryption Details**:
```javascript
Algorithm: AES-256-GCM
Key Size: 256 bits
IV Size: 12 bytes (random per chunk)
Tag Size: 128 bits (authentication)
```

**Key Storage**:
```javascript
Location: localStorage['msscs-key']
Format: JWK (JSON Web Key)
Persistence: Until cleared
```

### 3. P2P Network (p2p.js)

**Purpose**: Manage peer-to-peer connections

**Key Methods**:
```javascript
async init()                    // Initialize PeerJS
connectToPeer(peerId)           // Connect to peer
sendBlock(peerId, block)        // Send block to peer
broadcastBlock(block)           // Broadcast to all peers
async requestBlock(blockId)     // Request block from peers
getPeers()                      // Get connected peers
on(event, handler)              // Register event handler
emit(event, data)               // Emit event
```

**Message Protocol**:
```javascript
// Handshake
{ type: 'handshake', peerId: string }

// Block Request
{ type: 'block-request', blockId: string }

// Block Response
{ type: 'block-response', block: Block }

// Block Broadcast
{ type: 'block-broadcast', block: Block }
```

**Connection Flow**:
```
Peer A                          Peer B
   │                               │
   ├─ new Peer() ─────────────────┤
   │                               │
   ├─ connect(peerB) ──────────→  │
   │                               │
   │  ←────────── WebRTC SDP ─────┤
   │                               │
   ├─ Connection established ──────┤
   │                               │
   ├─ send(handshake) ──────────→  │
   │  ←────────── handshake ───────┤
   │                               │
   ├─ Data channel ready ──────────┤
```

### 4. Main Application (app.js)

**Purpose**: Coordinate all components and manage UI

**Key Methods**:
```javascript
async init()                    // Initialize app
async handleFiles(files)        // Handle file upload
async uploadFile(file)          // Upload single file
async downloadFile(fileId)      // Download file
async deleteFile(fileId)        // Delete file
async loadFiles()               // Load files from storage
updateUI()                      // Update all UI elements
updateFilesList()               // Update files list
updatePeersList()               // Update peers list
updateStats()                   // Update statistics
```

**Event Handling**:
```javascript
// File events
'change' on file input
'dragover' on upload area
'drop' on upload area

// P2P events
'ready' - Peer initialized
'peer-connected' - Peer connected
'peer-disconnected' - Peer disconnected
'block-request' - Block requested
'block-received' - Block received
```

## Security Model

### Threat Model

**Protected Against**:
- ✅ Network eavesdropping (encryption)
- ✅ Unauthorized access (encryption)
- ✅ Data tampering (GCM authentication)
- ✅ MITM attacks (WebRTC encryption)

**NOT Protected Against**:
- ❌ Browser compromise (full access)
- ❌ Key theft (localStorage accessible)
- ❌ Malicious peers (no authentication)
- ❌ DoS attacks (no rate limiting)

### Security Layers

```
┌─────────────────────────────────────┐
│  Application Layer                  │
│  - Input validation                 │
│  - Access control (none)            │
└─────────────────────────────────────┘
           ↓
┌─────────────────────────────────────┐
│  Encryption Layer                   │
│  - AES-256-GCM encryption           │
│  - SHA-256 hashing                  │
│  - Random IV generation             │
└─────────────────────────────────────┘
           ↓
┌─────────────────────────────────────┐
│  Transport Layer                    │
│  - WebRTC encryption (DTLS/SRTP)    │
│  - TLS for signaling                │
└─────────────────────────────────────┘
           ↓
┌─────────────────────────────────────┐
│  Storage Layer                      │
│  - Browser sandbox                  │
│  - IndexedDB isolation              │
└─────────────────────────────────────┘
```

### Key Management

```
Key Generation:
  crypto.subtle.generateKey()
       ↓
  256-bit AES key
       ↓
  Export as JWK
       ↓
  Store in localStorage
       ↓
  Use for all encryption

Key Recovery:
  User exports key
       ↓
  Save JSON file
       ↓
  Import on new device
       ↓
  Access encrypted data
```

## Performance Optimization

### Chunking Strategy

```javascript
File Size    | Chunk Size | Reason
-------------|------------|---------------------------
< 1 MB       | 64 KB      | Minimize overhead
1-100 MB     | 256 KB     | Balance speed/memory
100 MB - 1GB | 1 MB       | Maximize throughput
> 1 GB       | 2 MB       | Reduce chunk count
```

### Memory Management

```javascript
// Streaming approach for large files
async function* readFileInChunks(file, chunkSize) {
    let offset = 0;
    while (offset < file.size) {
        const chunk = file.slice(offset, offset + chunkSize);
        const buffer = await chunk.arrayBuffer();
        yield new Uint8Array(buffer);
        offset += chunkSize;
    }
}
```

### Caching Strategy

```javascript
// LRU cache for frequently accessed blocks
class BlockCache {
    constructor(maxSize = 100) {
        this.cache = new Map();
        this.maxSize = maxSize;
    }
    
    get(blockId) {
        const block = this.cache.get(blockId);
        if (block) {
            // Move to end (most recently used)
            this.cache.delete(blockId);
            this.cache.set(blockId, block);
        }
        return block;
    }
    
    set(blockId, block) {
        if (this.cache.size >= this.maxSize) {
            // Remove least recently used
            const firstKey = this.cache.keys().next().value;
            this.cache.delete(firstKey);
        }
        this.cache.set(blockId, block);
    }
}
```

## Error Handling

### Error Types

```javascript
// Storage errors
class StorageError extends Error {
    constructor(message) {
        super(message);
        this.name = 'StorageError';
    }
}

// Crypto errors
class CryptoError extends Error {
    constructor(message) {
        super(message);
        this.name = 'CryptoError';
    }
}

// Network errors
class NetworkError extends Error {
    constructor(message) {
        super(message);
        this.name = 'NetworkError';
    }
}
```

### Error Recovery

```javascript
// Retry logic for network operations
async function retryOperation(operation, maxRetries = 3) {
    for (let i = 0; i < maxRetries; i++) {
        try {
            return await operation();
        } catch (error) {
            if (i === maxRetries - 1) throw error;
            await sleep(1000 * Math.pow(2, i)); // Exponential backoff
        }
    }
}
```

## Testing Strategy

### Unit Tests
```javascript
// Test crypto operations
test('encrypt and decrypt', async () => {
    const data = new Uint8Array([1, 2, 3]);
    const encrypted = await crypto.encrypt(data);
    const decrypted = await crypto.decrypt(encrypted);
    expect(decrypted).toEqual(data);
});

// Test storage operations
test('save and retrieve block', async () => {
    const block = { id: '123', data: new Uint8Array([1, 2, 3]) };
    await storage.saveBlock(block);
    const retrieved = await storage.getBlock('123');
    expect(retrieved).toEqual(block);
});
```

### Integration Tests
```javascript
// Test full upload/download cycle
test('upload and download file', async () => {
    const file = new File(['test'], 'test.txt');
    await app.uploadFile(file);
    const files = await storage.getAllFiles();
    expect(files.length).toBe(1);
    await app.downloadFile(files[0].id);
});
```

### E2E Tests
```javascript
// Test P2P sync
test('sync between peers', async () => {
    const peer1 = new MSSCSWeb();
    const peer2 = new MSSCSWeb();
    await peer1.init();
    await peer2.init();
    peer1.p2p.connectToPeer(peer2.p2p.peerId);
    // Upload on peer1
    await peer1.uploadFile(file);
    // Wait for sync
    await sleep(1000);
    // Check peer2 has file
    const files = await peer2.storage.getAllFiles();
    expect(files.length).toBe(1);
});
```

## Monitoring and Debugging

### Logging
```javascript
// Enable debug logging
localStorage.setItem('debug', 'msscs:*');

// Log levels
console.debug('Debug message');
console.info('Info message');
console.warn('Warning message');
console.error('Error message');
```

### Performance Monitoring
```javascript
// Measure operation time
console.time('upload');
await app.uploadFile(file);
console.timeEnd('upload');

// Monitor memory usage
console.log(performance.memory);
```

### Network Monitoring
```javascript
// Monitor WebRTC stats
peer.on('connection', (conn) => {
    setInterval(() => {
        const stats = conn.peerConnection.getStats();
        console.log('Connection stats:', stats);
    }, 5000);
});
```

## Future Enhancements

### Planned Features
1. **Compression**: Add zlib compression before encryption
2. **Deduplication**: Content-based chunking
3. **Versioning**: Track file versions
4. **Search**: Full-text search
5. **Sharing**: Share files with specific peers
6. **Sync**: Selective sync options

### Architecture Improvements
1. **Web Workers**: Move encryption to workers
2. **Service Worker**: Enable offline mode
3. **PWA**: Progressive Web App support
4. **Streaming**: Stream large files
5. **Batching**: Batch operations for efficiency

## Conclusion

MSSCS Web provides a complete P2P encrypted storage solution that runs entirely in the browser. It demonstrates the power of modern web APIs (IndexedDB, Web Crypto, WebRTC) to build sophisticated distributed systems without any backend infrastructure.

The modular architecture makes it easy to extend and customize for specific use cases, while the comprehensive documentation ensures developers can understand and modify the system as needed.
