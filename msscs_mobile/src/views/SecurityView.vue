<template>
  <div class="security-view">
    <div class="header">
      <h1>🔐 Security & Privacy</h1>
    </div>

    <div class="content">
      <!-- Quantum-Resistant Encryption -->
      <div class="info-card">
        <div class="card-icon">🛡️</div>
        <h2>Quantum-Resistant Encryption</h2>
        <p class="description">
          Your files are protected with <strong>AES-256-GCM</strong> encryption, 
          combined with position-based key derivation using <strong>SHA-256 + UUID</strong>.
        </p>
        <div class="tech-details">
          <div class="tech-item">
            <span class="label">Algorithm:</span>
            <span class="value">{{ securityInfo?.encryption }}</span>
          </div>
          <div class="tech-item">
            <span class="label">Hash Function:</span>
            <span class="value">{{ securityInfo?.hashing }}</span>
          </div>
          <div class="tech-item">
            <span class="label">Compression:</span>
            <span class="value">{{ securityInfo?.compression }}</span>
          </div>
        </div>
      </div>

      <!-- Decentralized Storage -->
      <div class="info-card">
        <div class="card-icon">🌐</div>
        <h2>Decentralized P2P Network</h2>
        <p class="description">
          Your data is <strong>NOT stored on any central server</strong>. 
          Files are split into encrypted blocks and distributed across peer nodes.
        </p>
        <div class="features">
          <div class="feature">
            <span class="check">✓</span>
            <span>No single point of failure</span>
          </div>
          <div class="feature">
            <span class="check">✓</span>
            <span>Peer-to-peer replication</span>
          </div>
          <div class="feature">
            <span class="check">✓</span>
            <span>DHT-based discovery</span>
          </div>
          <div class="feature">
            <span class="check">✓</span>
            <span>Cross-platform support</span>
          </div>
        </div>
      </div>

      <!-- Local Storage -->
      <div class="info-card">
        <div class="card-icon">💾</div>
        <h2>Your Data Stays Local</h2>
        <p class="description">
          All files remain on <strong>YOUR device</strong> and connected peer devices. 
          Nobody else can access your encrypted blocks without your keys.
        </p>
        <div class="privacy-points">
          <div class="point">
            <strong>🔒 Private by Design</strong>
            <p>Only you have the decryption keys</p>
          </div>
          <div class="point">
            <strong>📱 Device Control</strong>
            <p>Data stored locally on your devices</p>
          </div>
          <div class="point">
            <strong>🚫 No Cloud Servers</strong>
            <p>Zero third-party data access</p>
          </div>
        </div>
      </div>

      <!-- Who Can See Your Files -->
      <div class="info-card warning">
        <div class="card-icon">👁️</div>
        <h2>Who Can Access Your Files?</h2>
        <div class="access-info">
          <div class="access-item allowed">
            <span class="icon">✓</span>
            <div>
              <strong>YOU</strong>
              <p>Full access with your encryption keys</p>
            </div>
          </div>
          <div class="access-item allowed">
            <span class="icon">✓</span>
            <div>
              <strong>YOUR TRUSTED PEERS</strong>
              <p>Only if you explicitly share with them</p>
            </div>
          </div>
          <div class="access-item denied">
            <span class="icon">✗</span>
            <div>
              <strong>EVERYONE ELSE</strong>
              <p>Cannot decrypt or access your data</p>
            </div>
          </div>
        </div>
      </div>

      <!-- How It Works -->
      <div class="info-card">
        <div class="card-icon">⚙️</div>
        <h2>How MSSCS Works</h2>
        <div class="steps">
          <div class="step">
            <div class="step-number">1</div>
            <div class="step-content">
              <strong>File Upload</strong>
              <p>Your file is split into chunks</p>
            </div>
          </div>
          <div class="step">
            <div class="step-number">2</div>
            <div class="step-content">
              <strong>Encryption</strong>
              <p>Each chunk encrypted with unique keys (UUID + Position)</p>
            </div>
          </div>
          <div class="step">
            <div class="step-number">3</div>
            <div class="step-content">
              <strong>Compression</strong>
              <p>Huffman coding reduces storage size</p>
            </div>
          </div>
          <div class="step">
            <div class="step-number">4</div>
            <div class="step-content">
              <strong>Distribution</strong>
              <p>Blocks replicated to peer nodes via P2P</p>
            </div>
          </div>
          <div class="step">
            <div class="step-number">5</div>
            <div class="step-content">
              <strong>Blockchain Chain</strong>
              <p>Blocks linked with SHA-256 hashes</p>
            </div>
          </div>
        </div>
      </div>

      <!-- Platform Support -->
      <div class="info-card">
        <div class="card-icon">📱</div>
        <h2>Cross-Platform Support</h2>
        <div class="platforms">
          <div class="platform">
            <span class="platform-icon">🪟</span>
            <span>Windows</span>
          </div>
          <div class="platform">
            <span class="platform-icon">🐧</span>
            <span>Linux</span>
          </div>
          <div class="platform">
            <span class="platform-icon">🤖</span>
            <span>Android</span>
          </div>
          <div class="platform">
            <span class="platform-icon">🍎</span>
            <span>iOS</span>
          </div>
        </div>
        <p class="platform-note">
          All platforms use the same encryption and network protocol
        </p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

interface SecurityInfo {
  encryption: string
  hashing: string
  compression: string
  decentralized: boolean
  local_storage: boolean
  p2p_network: boolean
}

const securityInfo = ref<SecurityInfo | null>(null)

onMounted(async () => {
  try {
    securityInfo.value = await invoke<SecurityInfo>('get_security_info')
  } catch (error) {
    console.error('Failed to load security info:', error)
  }
})
</script>

<style scoped>
.security-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #000;
  overflow-y: auto;
}

.header {
  padding: 16px;
  border-bottom: 1px solid #222;
  position: sticky;
  top: 0;
  background: #000;
  z-index: 10;
}

h1 {
  font-size: 24px;
  font-weight: 600;
}

.content {
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.info-card {
  background: #111;
  border: 1px solid #222;
  border-radius: 16px;
  padding: 20px;
}

.info-card.warning {
  border-color: #ff8800;
}

.card-icon {
  font-size: 48px;
  margin-bottom: 12px;
}

.info-card h2 {
  font-size: 20px;
  font-weight: 600;
  margin-bottom: 12px;
}

.description {
  color: #ccc;
  line-height: 1.6;
  margin-bottom: 16px;
}

.description strong {
  color: #00ff88;
}

.tech-details {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.tech-item {
  display: flex;
  justify-content: space-between;
  padding: 12px;
  background: #0a0a0a;
  border-radius: 8px;
}

.label {
  color: #888;
  font-size: 14px;
}

.value {
  color: #00ff88;
  font-size: 14px;
  font-weight: 500;
  text-align: right;
}

.features {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.feature {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  background: #0a0a0a;
  border-radius: 8px;
}

.check {
  color: #00ff88;
  font-size: 20px;
  font-weight: bold;
}

.privacy-points {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.point {
  padding: 16px;
  background: #0a0a0a;
  border-radius: 8px;
}

.point strong {
  display: block;
  margin-bottom: 4px;
  color: #00ff88;
}

.point p {
  color: #888;
  font-size: 14px;
}

.access-info {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.access-item {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 16px;
  border-radius: 8px;
}

.access-item.allowed {
  background: rgba(0, 255, 136, 0.1);
  border: 1px solid rgba(0, 255, 136, 0.3);
}

.access-item.denied {
  background: rgba(255, 68, 68, 0.1);
  border: 1px solid rgba(255, 68, 68, 0.3);
}

.access-item .icon {
  font-size: 24px;
  font-weight: bold;
}

.access-item.allowed .icon {
  color: #00ff88;
}

.access-item.denied .icon {
  color: #ff4444;
}

.access-item strong {
  display: block;
  margin-bottom: 4px;
}

.access-item p {
  color: #888;
  font-size: 14px;
}

.steps {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.step {
  display: flex;
  align-items: flex-start;
  gap: 16px;
  padding: 16px;
  background: #0a0a0a;
  border-radius: 8px;
}

.step-number {
  width: 32px;
  height: 32px;
  background: #00ff88;
  color: #000;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: bold;
  flex-shrink: 0;
}

.step-content strong {
  display: block;
  margin-bottom: 4px;
  color: #fff;
}

.step-content p {
  color: #888;
  font-size: 14px;
}

.platforms {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
  margin-bottom: 16px;
}

.platform {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 16px;
  background: #0a0a0a;
  border-radius: 8px;
}

.platform-icon {
  font-size: 32px;
}

.platform-note {
  text-align: center;
  color: #888;
  font-size: 14px;
}
</style>
