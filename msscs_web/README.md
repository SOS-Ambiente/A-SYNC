# MSSCS Web - Browser-Based P2P Encrypted Storage

A fully browser-based peer-to-peer encrypted storage system that runs entirely in your web browser with no backend required.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Browser](https://img.shields.io/badge/browser-Chrome%20%7C%20Firefox%20%7C%20Safari-green.svg)
![WebRTC](https://img.shields.io/badge/WebRTC-enabled-orange.svg)

## 🚀 Quick Start

```bash
# Windows
.\start.ps1

# Linux/Mac
./start.sh

# Or use Python
python -m http.server 8000
```

Then open `http://localhost:8000` in your browser.

**See [QUICKSTART.md](QUICKSTART.md) for a 60-second tutorial!**

## Features

- **🔐 End-to-End Encryption**: All files are encrypted using AES-256-GCM before storage
- **🌐 P2P Network**: WebRTC-based peer-to-peer connections for direct file sharing
- **💾 Local Storage**: IndexedDB for persistent local storage
- **📦 Chunked Storage**: Files are split into encrypted chunks for efficient distribution
- **🔗 Blockchain-like Chaining**: Chunks are cryptographically linked
- **🚀 Zero Backend**: Runs entirely in the browser, no server required

## Quick Start

### Option 1: Simple HTTP Server

```bash
# Using Python
python -m http.server 8000

# Using Node.js
npx http-server -p 8000

# Using PHP
php -S localhost:8000
```

Then open `http://localhost:8000` in your browser.

### Option 2: Direct File Access

Some browsers allow opening `index.html` directly, but P2P features may be limited due to CORS restrictions.

## How It Works

### Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Browser Application                   │
├─────────────────────────────────────────────────────────┤
│  UI Layer (HTML/CSS)                                    │
├─────────────────────────────────────────────────────────┤
│  App Logic (app.js)                                     │
│  ├─ File Upload/Download                               │
│  ├─ Progress Tracking                                  │
│  └─ UI Updates                                         │
├─────────────────────────────────────────────────────────┤
│  Storage Manager (storage.js)                          │
│  ├─ IndexedDB Interface                                │
│  ├─ Block Storage                                      │
│  └─ File Metadata                                      │
├─────────────────────────────────────────────────────────┤
│  Crypto Manager (crypto.js)                            │
│  ├─ AES-256-GCM Encryption                             │
│  ├─ SHA-256 Hashing                                    │
│  └─ Key Management                                     │
├─────────────────────────────────────────────────────────┤
│  P2P Network (p2p.js)                                  │
│  ├─ WebRTC Connections                                 │
│  ├─ PeerJS Signaling                                   │
│  └─ Block Replication                                  │
└─────────────────────────────────────────────────────────┘
```

### Data Flow

1. **Upload**:
   - User selects file
   - File is read as ArrayBuffer
   - Split into 256KB chunks
   - Each chunk is encrypted with AES-256-GCM
   - Chunks are linked with SHA-256 hashes
   - Stored in IndexedDB
   - Broadcast to connected peers

2. **Download**:
   - Retrieve file metadata
   - Fetch all chunks (local or from peers)
   - Decrypt each chunk
   - Reassemble original file
   - Trigger browser download

3. **P2P Sync**:
   - Peers connect via WebRTC
   - Exchange block availability
   - Request missing blocks
   - Replicate blocks across network

## Usage

### Uploading Files

1. Click "Choose files" or drag & drop files into the upload area
2. Files are automatically encrypted and chunked
3. Progress is shown in real-time
4. Files appear in "Your Files" section when complete

### Downloading Files

1. Click "Download" button next to any file
2. System retrieves all chunks (from local storage or peers)
3. Decrypts and reassembles the file
4. Browser download is triggered

### Connecting to Peers

1. Share your Peer ID (shown at bottom of page)
2. Other users enter your Peer ID and click "Connect"
3. Once connected, files are automatically replicated
4. Connected peers appear in "Connected Peers" section

## Security

### Encryption

- **Algorithm**: AES-256-GCM (Galois/Counter Mode)
- **Key Generation**: Web Crypto API `generateKey()`
- **Key Storage**: Browser localStorage (encrypted at rest by browser)
- **IV**: Random 12-byte IV per chunk
- **Authentication**: GCM provides authenticated encryption

### Key Management

- Keys are generated per-browser instance
- Stored in localStorage
- Can be exported/imported for multi-device access
- Clearing browser data removes keys (data becomes unrecoverable)

### Privacy

- No data sent to servers (except STUN for NAT traversal)
- All file data stays on your device and connected peers
- Peer connections are direct (WebRTC)
- No tracking or analytics

## Browser Compatibility

- ✅ Chrome/Edge 90+
- ✅ Firefox 88+
- ✅ Safari 15+
- ✅ Opera 76+

Requirements:
- IndexedDB support
- Web Crypto API
- WebRTC support
- ES6 Modules

## Limitations

- **Storage**: Limited by browser IndexedDB quota (typically 50% of available disk space)
- **File Size**: Large files (>1GB) may cause memory issues
- **Peers**: Direct connections require both peers online simultaneously
- **NAT**: Some restrictive networks may block WebRTC connections
- **Key Loss**: Clearing browser data loses encryption keys (data unrecoverable)

## Advanced Features

### Custom Encryption Key

```javascript
// Export your key
const key = await app.crypto.exportKey();
console.log(JSON.stringify(key));

// Import on another device
await app.crypto.importKey(keyData);
```

### Manual Peer Connection

```javascript
// Connect to specific peer
app.p2p.connectToPeer('peer-id-here');
```

### Clear All Data

```javascript
// Clear storage
await app.storage.clear();
app.crypto.clearKey();
```

## Development

### Project Structure

```
msscs_web/
├── index.html      # Main HTML page
├── styles.css      # Styling
├── app.js          # Main application logic
├── storage.js      # IndexedDB storage manager
├── crypto.js       # Web Crypto API wrapper
├── p2p.js          # WebRTC P2P networking
└── README.md       # This file
```

### Customization

**Change chunk size** (in `app.js`):
```javascript
const chunkSize = 512 * 1024; // 512KB chunks
```

**Change encryption algorithm** (in `crypto.js`):
```javascript
// Currently uses AES-256-GCM
// Can be modified to use other Web Crypto algorithms
```

**Add compression**:
```javascript
// Before encryption
const compressed = pako.deflate(data);
const encrypted = await crypto.encrypt(compressed);
```

## Troubleshooting

### "Peer connection failed"
- Check if both browsers support WebRTC
- Try different network (some corporate networks block WebRTC)
- Ensure both peers are online simultaneously

### "Storage quota exceeded"
- Clear old files
- Check browser storage settings
- Some browsers limit storage per origin

### "Decryption failed"
- Ensure same encryption key is used
- Check if chunks are corrupted
- Verify all chunks are available

## Future Enhancements

- [ ] DHT for peer discovery
- [ ] Compression before encryption
- [ ] Multi-device key sync
- [ ] File versioning
- [ ] Selective sync
- [ ] Bandwidth throttling
- [ ] Mobile app (PWA)
- [ ] Offline mode improvements

## License

MIT License - Feel free to use and modify

## Contributing

Contributions welcome! Areas for improvement:
- Better error handling
- UI/UX enhancements
- Performance optimizations
- Additional encryption options
- Better peer discovery

## Support

For issues or questions, please open an issue on the repository.
