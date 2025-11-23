import { P2PNetwork } from './p2p.js';
import { StorageManager } from './storage.js';
import { CryptoManager } from './crypto.js';

class MSSCSWeb {
    constructor() {
        this.p2p = new P2PNetwork();
        this.storage = new StorageManager();
        this.crypto = new CryptoManager();
        this.files = new Map();
        
        this.init();
    }

    async init() {
        // Initialize crypto
        await this.crypto.init();
        
        // Initialize storage
        await this.storage.init();
        
        // Load existing files
        await this.loadFiles();
        
        // Initialize P2P
        await this.p2p.init();
        
        // Setup event listeners
        this.setupEventListeners();
        this.setupP2PListeners();
        
        // Update UI
        this.updateUI();
        
        console.log('MSSCS Web initialized');
    }

    setupEventListeners() {
        // File upload
        const fileInput = document.getElementById('file-input');
        const uploadArea = document.getElementById('upload-area');
        
        fileInput.addEventListener('change', (e) => this.handleFiles(e.target.files));
        
        uploadArea.addEventListener('dragover', (e) => {
            e.preventDefault();
            uploadArea.classList.add('drag-over');
        });
        
        uploadArea.addEventListener('dragleave', () => {
            uploadArea.classList.remove('drag-over');
        });
        
        uploadArea.addEventListener('drop', (e) => {
            e.preventDefault();
            uploadArea.classList.remove('drag-over');
            this.handleFiles(e.dataTransfer.files);
        });
        
        // Peer connection
        document.getElementById('connect-peer-btn').addEventListener('click', () => {
            const peerId = document.getElementById('peer-id-input').value.trim();
            if (peerId) {
                this.p2p.connectToPeer(peerId);
                document.getElementById('peer-id-input').value = '';
            }
        });
    }

    setupP2PListeners() {
        this.p2p.on('ready', (peerId) => {
            document.getElementById('my-peer-id').textContent = peerId;
            document.getElementById('status').textContent = 'Online';
            document.getElementById('status').classList.add('online');
        });
        
        this.p2p.on('peer-connected', (peerId) => {
            console.log('Peer connected:', peerId);
            this.updateUI();
        });
        
        this.p2p.on('peer-disconnected', (peerId) => {
            console.log('Peer disconnected:', peerId);
            this.updateUI();
        });
        
        this.p2p.on('block-request', async (data) => {
            const { blockId, peerId } = data;
            const block = await this.storage.getBlock(blockId);
            if (block) {
                this.p2p.sendBlock(peerId, block);
            }
        });
        
        this.p2p.on('block-received', async (data) => {
            const { block } = data;
            await this.storage.saveBlock(block);
            this.updateUI();
        });
    }

    async handleFiles(files) {
        for (const file of files) {
            await this.uploadFile(file);
        }
    }

    async uploadFile(file) {
        const progressDiv = document.getElementById('upload-progress');
        const progressId = `progress-${Date.now()}`;
        
        // Create progress UI
        const progressItem = document.createElement('div');
        progressItem.className = 'progress-item';
        progressItem.id = progressId;
        progressItem.innerHTML = `
            <div>${file.name} (${this.formatBytes(file.size)})</div>
            <div class="progress-bar">
                <div class="progress-fill" style="width: 0%"></div>
            </div>
        `;
        progressDiv.appendChild(progressItem);
        
        try {
            // Read file
            const arrayBuffer = await file.arrayBuffer();
            const data = new Uint8Array(arrayBuffer);
            
            // Encrypt and chunk
            const chunkSize = 256 * 1024; // 256KB chunks
            const chunks = [];
            let previousHash = null;
            
            for (let i = 0; i < data.length; i += chunkSize) {
                const chunk = data.slice(i, Math.min(i + chunkSize, data.length));
                
                // Encrypt chunk
                const encryptedChunk = await this.crypto.encrypt(chunk);
                
                // Create block
                const block = {
                    id: this.generateId(),
                    data: encryptedChunk,
                    index: Math.floor(i / chunkSize),
                    previousHash,
                    timestamp: Date.now()
                };
                
                // Calculate hash
                block.hash = await this.crypto.hash(encryptedChunk);
                previousHash = block.hash;
                
                // Save block
                await this.storage.saveBlock(block);
                chunks.push(block.id);
                
                // Update progress
                const progress = ((i + chunk.length) / data.length) * 100;
                const progressFill = progressItem.querySelector('.progress-fill');
                progressFill.style.width = `${progress}%`;
                
                // Replicate to peers
                this.p2p.broadcastBlock(block);
            }
            
            // Save file metadata
            const fileMetadata = {
                id: this.generateId(),
                name: file.name,
                size: file.size,
                type: file.type,
                chunks,
                timestamp: Date.now()
            };
            
            await this.storage.saveFile(fileMetadata);
            this.files.set(fileMetadata.id, fileMetadata);
            
            // Remove progress UI
            setTimeout(() => progressItem.remove(), 2000);
            
            // Update UI
            this.updateUI();
            
            console.log('File uploaded:', file.name);
        } catch (error) {
            console.error('Upload error:', error);
            progressItem.innerHTML = `<div style="color: red;">Error uploading ${file.name}</div>`;
        }
    }

    async downloadFile(fileId) {
        try {
            const fileMetadata = this.files.get(fileId);
            if (!fileMetadata) {
                throw new Error('File not found');
            }
            
            // Collect all chunks
            const chunks = [];
            for (const chunkId of fileMetadata.chunks) {
                let block = await this.storage.getBlock(chunkId);
                
                // If not found locally, request from peers
                if (!block) {
                    block = await this.p2p.requestBlock(chunkId);
                }
                
                if (!block) {
                    throw new Error(`Chunk ${chunkId} not found`);
                }
                
                // Decrypt chunk
                const decrypted = await this.crypto.decrypt(block.data);
                chunks.push(decrypted);
            }
            
            // Combine chunks
            const totalLength = chunks.reduce((sum, chunk) => sum + chunk.length, 0);
            const fileData = new Uint8Array(totalLength);
            let offset = 0;
            for (const chunk of chunks) {
                fileData.set(chunk, offset);
                offset += chunk.length;
            }
            
            // Download
            const blob = new Blob([fileData], { type: fileMetadata.type });
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = fileMetadata.name;
            a.click();
            URL.revokeObjectURL(url);
            
            console.log('File downloaded:', fileMetadata.name);
        } catch (error) {
            console.error('Download error:', error);
            alert('Failed to download file: ' + error.message);
        }
    }

    async deleteFile(fileId) {
        if (!confirm('Are you sure you want to delete this file?')) {
            return;
        }
        
        const fileMetadata = this.files.get(fileId);
        if (fileMetadata) {
            // Delete chunks
            for (const chunkId of fileMetadata.chunks) {
                await this.storage.deleteBlock(chunkId);
            }
            
            // Delete metadata
            await this.storage.deleteFile(fileId);
            this.files.delete(fileId);
            
            this.updateUI();
        }
    }

    async loadFiles() {
        const files = await this.storage.getAllFiles();
        for (const file of files) {
            this.files.set(file.id, file);
        }
    }

    updateUI() {
        this.updateFilesList();
        this.updatePeersList();
        this.updateStats();
    }

    updateFilesList() {
        const filesList = document.getElementById('files-list');
        
        if (this.files.size === 0) {
            filesList.innerHTML = '<p class="empty-state">No files yet. Upload some files to get started!</p>';
            return;
        }
        
        filesList.innerHTML = '';
        for (const [fileId, file] of this.files) {
            const fileCard = document.createElement('div');
            fileCard.className = 'file-card';
            fileCard.innerHTML = `
                <div class="file-info">
                    <div class="file-name">${file.name}</div>
                    <div class="file-meta">
                        ${this.formatBytes(file.size)} • ${file.chunks.length} chunks • ${new Date(file.timestamp).toLocaleString()}
                    </div>
                </div>
                <div class="file-actions">
                    <button onclick="app.downloadFile('${fileId}')">Download</button>
                    <button class="danger" onclick="app.deleteFile('${fileId}')">Delete</button>
                </div>
            `;
            filesList.appendChild(fileCard);
        }
    }

    updatePeersList() {
        const peersList = document.getElementById('peers-list');
        const peers = this.p2p.getPeers();
        
        if (peers.length === 0) {
            peersList.innerHTML = '<p class="empty-state">No peers connected</p>';
            return;
        }
        
        peersList.innerHTML = '';
        for (const peer of peers) {
            const peerCard = document.createElement('div');
            peerCard.className = 'peer-card';
            peerCard.innerHTML = `
                <div class="peer-info">
                    <div class="peer-id">${peer.id}</div>
                    <div class="peer-meta">Connected</div>
                </div>
            `;
            peersList.appendChild(peerCard);
        }
    }

    updateStats() {
        document.getElementById('stat-files').textContent = this.files.size;
        
        const totalSize = Array.from(this.files.values()).reduce((sum, f) => sum + f.size, 0);
        document.getElementById('stat-storage').textContent = this.formatBytes(totalSize);
        
        const totalBlocks = Array.from(this.files.values()).reduce((sum, f) => sum + f.chunks.length, 0);
        document.getElementById('stat-blocks').textContent = totalBlocks;
        
        document.getElementById('stat-peers').textContent = this.p2p.getPeers().length;
        document.getElementById('peer-count').textContent = `${this.p2p.getPeers().length} peers`;
    }

    generateId() {
        return Array.from(crypto.getRandomValues(new Uint8Array(16)))
            .map(b => b.toString(16).padStart(2, '0'))
            .join('');
    }

    formatBytes(bytes) {
        if (bytes === 0) return '0 B';
        const k = 1024;
        const sizes = ['B', 'KB', 'MB', 'GB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i];
    }
}

// Initialize app
window.app = new MSSCSWeb();
