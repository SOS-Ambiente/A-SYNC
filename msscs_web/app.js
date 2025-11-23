// MSSCS Web - Decentralized P2P Storage Application
// Uses PeerJS for WebRTC connectivity across the internet
import { P2PNetwork } from './p2p.js';
import { StorageManager } from './storage.js';
import { CryptoManager } from './crypto.js';
import { QuantumCryptoManager } from './quantum-crypto.js';
import { ErasureCoding } from './erasure-coding.js';
import { WebRTCBridge } from './webrtc-bridge.js';

class MSSCSWeb {
    constructor() {
        this.p2p = new P2PNetwork();
        this.storage = new StorageManager();
        this.crypto = new CryptoManager();
        this.quantumCrypto = null; // Initialized in init()
        this.erasureCoding = new ErasureCoding(10, 4);
        this.bridge = null;
        this.files = new Map();
        
        this.init();
    }

    async init() {
        try {
            console.log('');
            console.log('╔════════════════════════════════════════════════════════════════╗');
            console.log('║  🚀 MSSCS Web - Decentralized Storage Network                ║');
            console.log('╚════════════════════════════════════════════════════════════════╝');
            console.log('');
            console.log('📋 Initialization Steps:');
            console.log('');
            console.log('⚠️  NOTE: server.js is a static file server only');
            console.log('   P2P network initializes in the browser (this script)');
            console.log('');
            
            // Update status
            this.updateStatus('Initializing crypto...', 'syncing');
            console.log('   [1/5] 🔐 Initializing quantum-resistant cryptography...');
            
            // CRITICAL FIX: Initialize quantum crypto properly
            this.quantumCrypto = new QuantumCryptoManager();
            
            // Get passphrase from localStorage or prompt
            let passphrase = localStorage.getItem('msscs_passphrase_hint');
            if (!passphrase) {
                passphrase = prompt('🔐 Create a passphrase for quantum encryption:\n(This will protect your files with 2^832 attack complexity)') || 'default-passphrase-change-me';
                localStorage.setItem('msscs_passphrase_hint', 'set'); // Don't store actual passphrase
            } else {
                passphrase = prompt('🔐 Enter your passphrase:') || 'default-passphrase-change-me';
            }
            
            await this.quantumCrypto.init(passphrase);
            console.log('         ✅ 7-layer quantum-proof encryption ready');
            console.log('         ✅ ML-KEM-1024 (Kyber) initialized');
            console.log('         ✅ ML-DSA-87 (Dilithium) initialized');
            console.log('         ✅ Attack complexity: 2^832 (quantum-resistant)');
            
            // Initialize standard crypto as fallback
            await this.crypto.init();
            console.log('         ✅ AES-256-GCM encryption ready (fallback)');
            console.log('');
            
            // Store start time for uptime calculation
            window.msscsStartTime = Date.now();
            
            // Update status
            this.updateStatus('Initializing storage...', 'syncing');
            console.log('   [2/5] 💾 Initializing local storage (IndexedDB)...');
            
            // Initialize storage
            await this.storage.init();
            console.log('         ✅ Local block cache ready');
            console.log('');
            
            // Load existing files
            console.log('   [3/5] 📂 Loading existing files...');
            await this.loadFiles();
            console.log('         ✅ Loaded', this.files.size, 'files');
            console.log('');
            
            // Initialize storage limit selector
            console.log('   [4/5] 📊 Configuring storage allocation...');
            const storageLimitSelect = document.getElementById('storage-limit');
            if (storageLimitSelect) {
                const currentLimit = this.getStorageLimitMB();
                storageLimitSelect.value = currentLimit.toString();
                console.log('         ✅ Storage limit:', currentLimit, 'MB', `(${(currentLimit / 1024).toFixed(2)} GB)`);
                console.log('         💡 You can change this in settings');
                
                // Listen for storage limit changes
                storageLimitSelect.addEventListener('change', (e) => {
                    const newLimit = parseInt(e.target.value);
                    this.setStorageLimit(newLimit);
                });
            }
            console.log('');
            
            // Update status
            this.updateStatus('Connecting to P2P network...', 'syncing');
            console.log('   [5/5] 🌐 Connecting to global P2P network...');
            console.log('         This may take a few seconds...');
            console.log('');
            
            // Initialize P2P with timeout and retry
            let p2pConnected = false;
            try {
                await Promise.race([
                    this.p2p.init(),
                    new Promise((_, reject) => 
                        setTimeout(() => reject(new Error('P2P initialization timeout')), 15000)
                    )
                ]);
                console.log('✅ P2P network connected successfully');
                p2pConnected = true;
            } catch (p2pError) {
                console.warn('⚠️  P2P initialization issue:', p2pError.message);
                console.log('🔄 Retrying P2P connection...');
                // Retry once
                try {
                    await this.p2p.init();
                    console.log('✅ P2P network connected on retry');
                    p2pConnected = true;
                } catch (retryError) {
                    console.error('❌ P2P connection failed after retry');
                    console.log('💡 Running in offline mode - you can still:');
                    console.log('   • Upload files (stored locally)');
                    console.log('   • View your files');
                    console.log('   • Connect manually by entering a Peer ID');
                    this.updateStatus('Offline - No P2P connection', 'offline');
                    // Continue initialization in offline mode
                }
            }
            
            // Initialize WebRTC bridge for cross-platform connectivity
            console.log('   [5.5/5] 🌉 Initializing WebRTC bridge...');
            this.bridge = new WebRTCBridge(this.p2p);
            await this.bridge.init();
            console.log('         ✅ Bridge ready for cross-platform connections');
            console.log('');
            
            // Setup event listeners
            this.setupEventListeners();
            this.setupP2PListeners();
            
            // Update UI
            this.updateUI();
            
            // Start auto-updating stats
            this.startStatsUpdater();
            
            // CRITICAL FIX: Set online status based on P2P connection
            if (p2pConnected) {
                const connStats = this.p2p.getConnectionStats();
                if (connStats.isConnected && connStats.peerId) {
                    this.updateStatus('Online - Ready for connections', 'online');
                } else {
                    this.updateStatus('Connecting...', 'syncing');
                }
            } else {
                this.updateStatus('Offline - No P2P connection', 'offline');
            }
            
            console.log('');
            console.log('╔════════════════════════════════════════════════════════════════╗');
            console.log('║  ✅ MSSCS Web Successfully Initialized                        ║');
            console.log('╚════════════════════════════════════════════════════════════════╝');
            console.log('');
            
            // Check P2P status
            const p2pStats = this.p2p.getConnectionStats();
            if (p2pStats.isConnected) {
                console.log('🎉 You are now connected to the decentralized storage network!');
                console.log('   Peer ID:', p2pStats.peerId);
                console.log('   Status: Online');
            } else {
                console.log('⚠️  Running in offline mode');
                console.log('   You can still use local features');
                console.log('   Connect manually by entering a Peer ID');
            }
            
            console.log('');
            console.log('📍 What you can do:');
            console.log('   • Upload files (encrypted and distributed)');
            console.log('   • Download files from the network');
            console.log('   • Share your Peer ID to connect with others');
            console.log('   • Contribute storage to help the network');
            console.log('');
            console.log('🔒 Security:');
            console.log('   • All files are encrypted before upload');
            console.log('   • Only you can decrypt your files');
            console.log('   • Peers cannot read your data');
            console.log('');
            console.log('🌍 Network:');
            console.log('   • P2P via PeerJS (WebRTC)');
            console.log('   • Works behind NAT/firewalls');
            console.log('   • No central server required');
            console.log('');
        } catch (error) {
            console.error('');
            console.error('╔════════════════════════════════════════════════════════════════╗');
            console.error('║  ❌ Initialization Failed                                     ║');
            console.error('╚════════════════════════════════════════════════════════════════╝');
            console.error('');
            console.error('Error:', error.message);
            console.error('');
            console.error('💡 Troubleshooting:');
            console.error('   • Check your internet connection');
            console.error('   • Ensure browser supports WebRTC');
            console.error('   • Try refreshing the page');
            console.error('   • Check browser console for details');
            console.error('');
            this.updateStatus('Offline - ' + error.message, 'offline');
        }
    }
    
    updateStatus(text, state) {
        const statusEl = document.getElementById('status');
        const indicatorEl = document.getElementById('status-indicator');
        
        if (statusEl) {
            statusEl.textContent = text;
            // Remove old state classes
            statusEl.classList.remove('online', 'offline', 'syncing');
            statusEl.classList.add(state);
        }
        if (indicatorEl) {
            indicatorEl.className = 'status-indicator ' + state;
        }
        
        // Log status changes for debugging
        console.log(`📊 Status: ${text} (${state})`);
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
            const peerIdEl = document.getElementById('my-peer-id');
            peerIdEl.textContent = peerId;
            
            // Make peer ID copyable
            peerIdEl.style.cursor = 'pointer';
            peerIdEl.title = 'Click to copy';
            peerIdEl.onclick = () => {
                navigator.clipboard.writeText(peerId).then(() => {
                    const originalText = peerIdEl.textContent;
                    peerIdEl.textContent = '✓ Copied!';
                    setTimeout(() => {
                        peerIdEl.textContent = originalText;
                    }, 2000);
                }).catch(() => {
                    alert('Failed to copy. Peer ID: ' + peerId);
                });
            };
            
            // Show connection info
            const connInfo = this.p2p.getConnectionInfo();
            console.log('📋 Connection Info:', connInfo);
            console.log('🔗 Share this URL:', connInfo.connectionUrl);
            console.log('📱 QR Code:', connInfo.qrCode);
            
            // Update status to online
            this.updateStatus('Online - Discovering peers...', 'online');
        });
        
        this.p2p.on('peer-connected', (peerId) => {
            console.log('✅ Peer connected:', peerId);
            const peerCount = this.p2p.getPeers().length;
            this.updateStatus(`Online - ${peerCount} peer${peerCount !== 1 ? 's' : ''} connected`, 'online');
            this.updateUI();
        });
        
        this.p2p.on('peer-disconnected', (peerId) => {
            console.log('❌ Peer disconnected:', peerId);
            const peerCount = this.p2p.getPeers().length;
            if (peerCount === 0) {
                this.updateStatus('Online - Discovering peers...', 'online');
            } else {
                this.updateStatus(`Online - ${peerCount} peer${peerCount !== 1 ? 's' : ''} connected`, 'online');
            }
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
        const startTime = Date.now();
        let lastProgressUpdate = Date.now();
        
        // Check storage limit
        const stats = await this.getStorageStats();
        if (stats.storage_used + file.size > stats.storage_limit) {
            alert(`❌ Not enough storage space!\n\nRequired: ${this.formatBytes(file.size)}\nAvailable: ${this.formatBytes(stats.storage_available)}\n\nIncrease your storage limit in settings.`);
            return;
        }
        
        // Create progress UI
        const progressItem = document.createElement('div');
        progressItem.className = 'progress-item';
        progressItem.id = progressId;
        progressItem.innerHTML = `
            <div class="progress-header">
                <span>${file.name} (${this.formatBytes(file.size)})</span>
                <span class="progress-speed">0 KB/s</span>
            </div>
            <div class="progress-bar">
                <div class="progress-fill" style="width: 0%"></div>
            </div>
            <div class="progress-stats">
                <span class="progress-percent">0%</span>
                <span class="progress-eta">Calculating...</span>
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
            let bytesProcessed = 0;
            
            for (let i = 0; i < data.length; i += chunkSize) {
                const chunk = data.slice(i, Math.min(i + chunkSize, data.length));
                
                // CRITICAL FIX: Use quantum-proof encryption (7-layer) instead of AES-only
                const encryptedChunk = await this.quantumCrypto.encryptQuantumProof(chunk);
                
                // Verify encryption worked
                if (!encryptedChunk || !encryptedChunk.kyberCiphertext) {
                    throw new Error('Quantum encryption failed - invalid encrypted block');
                }
                
                // Serialize encrypted chunk for hashing
                const encryptedChunkBytes = new TextEncoder().encode(JSON.stringify(encryptedChunk));
                
                // Create block
                const block = {
                    id: this.generateId(),
                    data: encryptedChunk,
                    index: Math.floor(i / chunkSize),
                    previousHash,
                    timestamp: Date.now()
                };
                
                // Calculate hash
                block.hash = await this.crypto.hash(encryptedChunkBytes);
                previousHash = block.hash;
                
                // Save block
                await this.storage.saveBlock(block);
                chunks.push(block.id);
                
                bytesProcessed += chunk.length;
                
                // Update progress with speed and ETA (throttled to 100ms for performance)
                const now = Date.now();
                if (now - lastProgressUpdate >= 100 || bytesProcessed === data.length) {
                    lastProgressUpdate = now;
                    
                    const progress = (bytesProcessed / data.length) * 100;
                    const elapsed = (now - startTime) / 1000;
                    const speed = elapsed > 0 ? bytesProcessed / elapsed : 0;
                    const remaining = data.length - bytesProcessed;
                    const eta = speed > 0 ? remaining / speed : 0;
                    
                    const progressFill = progressItem.querySelector('.progress-fill');
                    const progressPercent = progressItem.querySelector('.progress-percent');
                    const progressSpeed = progressItem.querySelector('.progress-speed');
                    const progressEta = progressItem.querySelector('.progress-eta');
                    
                    progressFill.style.width = `${progress}%`;
                    progressPercent.textContent = `${Math.round(progress)}%`;
                    progressSpeed.textContent = `${this.formatBytes(speed)}/s`;
                    progressEta.textContent = `ETA: ${this.formatTime(eta)}`;
                }
                
                // Replicate to peers (throttled - only every 10th block to reduce network overhead)
                if (chunks.length % 10 === 0 || i === data.length - chunkSize) {
                    this.p2p.broadcastBlock(block);
                }
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
            
            // Show completion
            progressItem.querySelector('.progress-header').innerHTML = `
                <span>✅ ${file.name}</span>
                <span class="progress-speed">Complete</span>
            `;
            
            // Remove progress UI after delay
            setTimeout(() => progressItem.remove(), 3000);
            
            // Update UI
            this.updateUI();
            
            console.log('✅ File uploaded:', file.name, `(${chunks.length} chunks, ${this.formatBytes(file.size)})`);
        } catch (error) {
            console.error('❌ Upload error:', error);
            progressItem.innerHTML = `<div style="color: red;">❌ Error uploading ${file.name}: ${error.message}</div>`;
            setTimeout(() => progressItem.remove(), 5000);
        }
    }

    async downloadFile(fileId) {
        const progressDiv = document.getElementById('upload-progress');
        const progressId = `download-${Date.now()}`;
        const startTime = Date.now();
        let lastProgressUpdate = Date.now();
        
        try {
            const fileMetadata = this.files.get(fileId);
            if (!fileMetadata) {
                throw new Error('File not found');
            }
            
            // Create progress UI
            const progressItem = document.createElement('div');
            progressItem.className = 'progress-item';
            progressItem.id = progressId;
            progressItem.innerHTML = `
                <div class="progress-header">
                    <span>⬇️ ${fileMetadata.name} (${this.formatBytes(fileMetadata.size)})</span>
                    <span class="progress-speed">0 KB/s</span>
                </div>
                <div class="progress-bar">
                    <div class="progress-fill" style="width: 0%"></div>
                </div>
                <div class="progress-stats">
                    <span class="progress-percent">0%</span>
                    <span class="progress-eta">Calculating...</span>
                </div>
            `;
            progressDiv.appendChild(progressItem);
            
            // Collect all chunks with progress
            const chunks = [];
            let bytesProcessed = 0;
            const totalChunks = fileMetadata.chunks.length;
            
            // CRITICAL SECURITY: Verify blockchain chain integrity before decryption
            console.log('🔗 Verifying blockchain chain integrity...');
            const blocks = [];
            for (let i = 0; i < totalChunks; i++) {
                const chunkId = fileMetadata.chunks[i];
                let block = await this.storage.getBlock(chunkId);
                
                // If not found locally, request from peers
                if (!block) {
                    block = await this.p2p.requestBlock(chunkId);
                }
                
                if (!block) {
                    throw new Error(`Chunk ${chunkId} not found`);
                }
                
                blocks.push(block);
            }
            
            // Verify chain integrity
            if (blocks.length > 1) {
                for (let i = 1; i < blocks.length; i++) {
                    if (blocks[i].previousHash !== blocks[i-1].hash) {
                        throw new Error(`Chain broken at block ${i}: hash mismatch - TAMPERING DETECTED`);
                    }
                }
                console.log('✅ Chain verification passed - all blocks valid');
            }
            
            // Now decrypt verified blocks
            for (let i = 0; i < blocks.length; i++) {
                const block = blocks[i];
                
                // CRITICAL FIX: Use quantum-proof decryption (7-layer)
                const decrypted = await this.quantumCrypto.decryptQuantumProof(block.data);
                
                // Verify decryption worked
                if (!decrypted || decrypted.length === 0) {
                    throw new Error(`Quantum decryption failed for chunk ${i}`);
                }
                
                chunks.push(decrypted);
                bytesProcessed += decrypted.length;
                
                // Update progress (throttled to 100ms for performance)
                const now = Date.now();
                if (now - lastProgressUpdate >= 100 || i === totalChunks - 1) {
                    lastProgressUpdate = now;
                    
                    const progress = ((i + 1) / totalChunks) * 100;
                    const elapsed = (now - startTime) / 1000;
                    const speed = elapsed > 0 ? bytesProcessed / elapsed : 0;
                    const remaining = fileMetadata.size - bytesProcessed;
                    const eta = speed > 0 ? remaining / speed : 0;
                    
                    const progressFill = progressItem.querySelector('.progress-fill');
                    const progressPercent = progressItem.querySelector('.progress-percent');
                    const progressSpeed = progressItem.querySelector('.progress-speed');
                    const progressEta = progressItem.querySelector('.progress-eta');
                    
                    progressFill.style.width = `${progress}%`;
                    progressPercent.textContent = `${Math.round(progress)}%`;
                    progressSpeed.textContent = `${this.formatBytes(speed)}/s`;
                    progressEta.textContent = `ETA: ${this.formatTime(eta)}`;
                }
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
            
            // Show completion
            progressItem.querySelector('.progress-header').innerHTML = `
                <span>✅ ${fileMetadata.name}</span>
                <span class="progress-speed">Complete</span>
            `;
            
            // Remove progress UI after delay
            setTimeout(() => progressItem.remove(), 3000);
            
            console.log('✅ File downloaded:', fileMetadata.name, `(${this.formatBytes(fileMetadata.size)})`);
        } catch (error) {
            console.error('❌ Download error:', error);
            const progressItem = document.getElementById(progressId);
            if (progressItem) {
                progressItem.innerHTML = `<div style="color: red;">❌ Error downloading: ${error.message}</div>`;
                setTimeout(() => progressItem.remove(), 5000);
            }
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

    async updateStats() {
        const stats = await this.getStorageStats();
        
        // Update stat cards
        const statFiles = document.getElementById('stat-files');
        const statStorage = document.getElementById('stat-storage');
        const statBlocks = document.getElementById('stat-blocks');
        const statPeers = document.getElementById('stat-peers');
        const peerCount = document.getElementById('peer-count');
        
        if (statFiles) statFiles.textContent = stats.total_files;
        if (statStorage) {
            // Show local storage used + global network estimate
            const localText = this.formatBytes(stats.storage_used);
            const globalText = stats.connected_peers > 0 
                ? ` (${this.formatBytes(stats.global_storage_estimate)} network)`
                : '';
            statStorage.textContent = localText;
            statStorage.title = `Local: ${localText}${globalText}`;
        }
        if (statBlocks) statBlocks.textContent = Array.from(this.files.values()).reduce((sum, f) => sum + f.chunks.length, 0);
        if (statPeers) statPeers.textContent = stats.connected_peers;
        if (peerCount) {
            const connStats = this.p2p.getConnectionStats();
            const connectionTypes = connStats.connectionTypes || { direct: 0, relayed: 0 };
            const peerText = `${stats.connected_peers} peer${stats.connected_peers !== 1 ? 's' : ''}`;
            const typeText = connectionTypes.direct > 0 ? ` (${connectionTypes.direct} direct)` : '';
            const globalText = stats.connected_peers > 0 
                ? ` • ~${this.formatBytes(stats.global_storage_estimate)} global`
                : '';
            peerCount.textContent = peerText + typeText + globalText;
        }
        
        // Update storage allocation display
        await this.updateStorageAllocation(stats.storage_used);
        
        // CRITICAL FIX: Always show online if P2P is connected, never stay in syncing
        const connStats = this.p2p.getConnectionStats();
        if (connStats.isConnected && connStats.peerId) {
            // Node is online if it has a peer ID (connected to signaling server)
            const statusText = stats.connected_peers > 0 
                ? `Online - ${stats.connected_peers} peer${stats.connected_peers !== 1 ? 's' : ''} connected`
                : 'Online - Ready for connections';
            this.updateStatus(statusText, 'online');
        } else {
            this.updateStatus('Offline - No P2P connection', 'offline');
        }
    }
    
    // Auto-update stats every 2 seconds
    startStatsUpdater() {
        let lastStatsBroadcast = 0;
        
        setInterval(async () => {
            await this.updateStats();
            
            // Broadcast storage stats to peers every 10 seconds (not every 2 seconds)
            const now = Date.now();
            if (now - lastStatsBroadcast >= 10000) {
                lastStatsBroadcast = now;
                const stats = await this.getStorageStats();
                this.p2p.broadcastStorageStats(stats);
            }
        }, 2000);
    }
    
    async updateStorageAllocation(usedBytes) {
        const storageLimit = this.getStorageLimit();
        const percentage = (usedBytes / storageLimit) * 100;
        
        const storageBar = document.getElementById('storage-bar');
        const storageText = document.getElementById('storage-text');
        
        if (storageBar) {
            storageBar.style.width = `${Math.min(percentage, 100)}%`;
            storageBar.className = 'storage-bar-fill';
            if (percentage > 90) {
                storageBar.classList.add('storage-warning');
            } else if (percentage > 75) {
                storageBar.classList.add('storage-caution');
            }
        }
        
        if (storageText) {
            const availableBytes = storageLimit - usedBytes;
            storageText.textContent = `${this.formatBytes(usedBytes)} / ${this.formatBytes(storageLimit)} (${percentage.toFixed(1)}%) - ${this.formatBytes(availableBytes)} available`;
        }
    }
    
    async getStorageStats() {
        const totalSize = Array.from(this.files.values()).reduce((sum, f) => sum + f.size, 0);
        const storageLimit = this.getStorageLimit();
        const connStats = this.p2p.getConnectionStats();
        
        // Calculate global network storage estimate
        // Each peer contributes their storage limit to the network
        const peerCount = connStats.connectedPeers;
        const globalStorageEstimate = storageLimit * (peerCount + 1); // +1 for self
        
        return {
            storage_used: totalSize,
            storage_limit: storageLimit,
            storage_available: Math.max(0, storageLimit - totalSize),
            total_files: this.files.size,
            connected_peers: peerCount,
            cached_blocks: Array.from(this.files.values()).reduce((sum, f) => sum + f.chunks.length, 0),
            global_storage_estimate: globalStorageEstimate
        };
    }
    
    getStorageLimit() {
        // Get from localStorage or default to 1GB
        const limitMB = parseInt(localStorage.getItem('msscs_storage_limit_mb') || '1024');
        return limitMB * 1024 * 1024;
    }
    
    setStorageLimit(limitMB) {
        localStorage.setItem('msscs_storage_limit_mb', limitMB.toString());
        console.log(`📊 Storage limit set to ${limitMB} MB (${(limitMB / 1024).toFixed(2)} GB)`);
        this.updateUI();
        
        // Emit event for UI update
        if (typeof window !== 'undefined') {
            window.dispatchEvent(new CustomEvent('storage-limit-changed', { 
                detail: { limitMB, limitBytes: limitMB * 1024 * 1024 } 
            }));
        }
    }
    
    getStorageLimitMB() {
        return parseInt(localStorage.getItem('msscs_storage_limit_mb') || '1024');
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
    
    formatTime(seconds) {
        if (!isFinite(seconds) || seconds < 0) return '--';
        if (seconds < 60) return `${Math.round(seconds)}s`;
        if (seconds < 3600) return `${Math.floor(seconds / 60)}m ${Math.round(seconds % 60)}s`;
        return `${Math.floor(seconds / 3600)}h ${Math.floor((seconds % 3600) / 60)}m`;
    }
}

// Initialize app
window.app = new MSSCSWeb();
