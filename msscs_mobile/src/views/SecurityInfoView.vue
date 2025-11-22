<template>
  <div class="security-view">
    <div class="header">
      <h1>🔐 Security & Privacy</h1>
    </div>

    <div class="content">
      <!-- Main Security Info -->
      <div class="info-card highlight">
        <div class="card-icon">🛡️</div>
        <h2>Your Data is Secure</h2>
        <p>MSSCS uses quantum-resistant encryption to protect your files across a decentralized network.</p>
      </div>

      <!-- Key Features -->
      <div class="features-grid">
        <div class="feature-card">
          <div class="feature-icon">🔒</div>
          <h3>AES-256-GCM</h3>
          <p>Military-grade encryption for every file block</p>
        </div>

        <div class="feature-card">
          <div class="feature-icon">🧬</div>
          <h3>SHA-256 Hashing</h3>
          <p>Cryptographic verification with UUID + position-based keys</p>
        </div>

        <div class="feature-card">
          <div class="feature-icon">🌐</div>
          <h3>P2P Network</h3>
          <p>Decentralized storage across peer nodes</p>
        </div>

        <div class="feature-card">
          <div class="feature-icon">⚡</div>
          <h3>Huffman Compression</h3>
          <p>Efficient data compression before encryption</p>
        </div>
      </div>

      <!-- FAQ Section -->
      <div class="faq-section">
        <h2>Frequently Asked Questions</h2>

        <div class="faq-item" @click="toggleFaq(0)">
          <div class="faq-question">
            <span>📍 Where is my data stored?</span>
            <span class="faq-toggle">{{ openFaq === 0 ? '−' : '+' }}</span>
          </div>
          <div v-if="openFaq === 0" class="faq-answer">
            <p><strong>Your data is stored in THREE places:</strong></p>
            <ul>
              <li><strong>Locally on your device</strong> - Always accessible offline</li>
              <li><strong>Encrypted blocks on P2P network</strong> - Distributed across peer nodes</li>
              <li><strong>Blockchain-style chain</strong> - Each block links to the previous one</li>
            </ul>
            <p class="highlight-text">✓ You maintain full control of your data</p>
          </div>
        </div>

        <div class="faq-item" @click="toggleFaq(1)">
          <div class="faq-question">
            <span>👁️ Can others see my files?</span>
            <span class="faq-toggle">{{ openFaq === 1 ? '−' : '+' }}</span>
          </div>
          <div v-if="openFaq === 1" class="faq-answer">
            <p><strong>NO. Your files are completely private:</strong></p>
            <ul>
              <li>Each block is encrypted with <strong>AES-256-GCM</strong></li>
              <li>Encryption key = <code>SHA256(UUID + block_position)</code></li>
              <li>Only YOU have the manifest linking blocks to files</li>
              <li>Peers only store encrypted chunks - they cannot decrypt them</li>
            </ul>
            <p class="highlight-text">✓ Even if someone intercepts a block, it's useless without your keys</p>
          </div>
        </div>

        <div class="faq-item" @click="toggleFaq(2)">
          <div class="faq-question">
            <span>🔬 What is "Quantum-Resistant"?</span>
            <span class="faq-toggle">{{ openFaq === 2 ? '−' : '+' }}</span>
          </div>
          <div v-if="openFaq === 2" class="faq-answer">
            <p><strong>Protection against future quantum computers:</strong></p>
            <ul>
              <li><strong>Position-based encryption</strong> - Keys depend on block position in chain</li>
              <li><strong>UUID randomization</strong> - Unique identifier per block</li>
              <li><strong>SHA-256 hashing</strong> - Quantum-resistant hash function</li>
              <li><strong>Chain verification</strong> - Each block cryptographically linked</li>
            </ul>
            <p class="highlight-text">✓ Your data remains secure even against quantum attacks</p>
          </div>
        </div>

        <div class="faq-item" @click="toggleFaq(3)">
          <div class="faq-question">
            <span>🌍 How does P2P work?</span>
            <span class="faq-toggle">{{ openFaq === 3 ? '−' : '+' }}</span>
          </div>
          <div v-if="openFaq === 3" class="faq-answer">
            <p><strong>Decentralized peer-to-peer network:</strong></p>
            <ul>
              <li><strong>DHT (Distributed Hash Table)</strong> - Find peers automatically</li>
              <li><strong>Block replication</strong> - Your encrypted blocks are copied to peers</li>
              <li><strong>No central server</strong> - Network runs on participant devices</li>
              <li><strong>Cross-platform</strong> - Works on Windows, Linux, Android, iOS</li>
            </ul>
            <p class="highlight-text">✓ More peers = better redundancy and availability</p>
          </div>
        </div>

        <div class="faq-item" @click="toggleFaq(4)">
          <div class="faq-question">
            <span>🗑️ What happens when I delete a file?</span>
            <span class="faq-toggle">{{ openFaq === 4 ? '−' : '+' }}</span>
          </div>
          <div v-if="openFaq === 4" class="faq-answer">
            <p><strong>File deletion process:</strong></p>
            <ul>
              <li>File is removed from your <strong>local manifest</strong></li>
              <li>You lose access to the file immediately</li>
              <li>Encrypted blocks may remain on peer nodes temporarily</li>
              <li>Without your manifest, blocks are <strong>unrecoverable</strong></li>
            </ul>
            <p class="highlight-text">⚠️ Deletion is permanent - no recovery possible</p>
          </div>
        </div>

        <div class="faq-item" @click="toggleFaq(5)">
          <div class="faq-question">
            <span>💾 Does this use my storage?</span>
            <span class="faq-toggle">{{ openFaq === 5 ? '−' : '+' }}</span>
          </div>
          <div v-if="openFaq === 5" class="faq-answer">
            <p><strong>Storage usage:</strong></p>
            <ul>
              <li><strong>Your files</strong> - Stored locally (compressed + encrypted)</li>
              <li><strong>Peer blocks</strong> - You help store others' encrypted blocks</li>
              <li><strong>Metadata</strong> - File manifest and block index</li>
            </ul>
            <p>Storage is shared across the network. You contribute storage to help others, and they help you.</p>
            <p class="highlight-text">✓ Fair exchange: Store ~same amount as you upload</p>
          </div>
        </div>
      </div>

      <!-- Technical Details -->
      <div class="tech-details">
        <h2>Technical Specifications</h2>
        <div class="spec-grid">
          <div class="spec-item">
            <span class="spec-label">Encryption</span>
            <span class="spec-value">AES-256-GCM</span>
          </div>
          <div class="spec-item">
            <span class="spec-label">Hashing</span>
            <span class="spec-value">SHA-256</span>
          </div>
          <div class="spec-item">
            <span class="spec-label">Compression</span>
            <span class="spec-value">Huffman Coding</span>
          </div>
          <div class="spec-item">
            <span class="spec-label">Network</span>
            <span class="spec-value">P2P DHT</span>
          </div>
          <div class="spec-item">
            <span class="spec-label">Block Chain</span>
            <span class="spec-value">SHA-256 Linked</span>
          </div>
          <div class="spec-item">
            <span class="spec-label">Key Derivation</span>
            <span class="spec-value">UUID + Position</span>
          </div>
        </div>
      </div>

      <!-- Security Guarantees -->
      <div class="guarantees">
        <h2>Security Guarantees</h2>
        <div class="guarantee-list">
          <div class="guarantee-item">
            <span class="check">✓</span>
            <span>End-to-end encryption</span>
          </div>
          <div class="guarantee-item">
            <span class="check">✓</span>
            <span>Zero-knowledge architecture</span>
          </div>
          <div class="guarantee-item">
            <span class="check">✓</span>
            <span>Quantum-resistant design</span>
          </div>
          <div class="guarantee-item">
            <span class="check">✓</span>
            <span>Decentralized storage</span>
          </div>
          <div class="guarantee-item">
            <span class="check">✓</span>
            <span>No central authority</span>
          </div>
          <div class="guarantee-item">
            <span class="check">✓</span>
            <span>Cross-platform compatibility</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'

const openFaq = ref<number | null>(null)

function toggleFaq(index: number) {
  openFaq.value = openFaq.value === index ? null : index
}
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
  padding-bottom: 80px;
}

.info-card {
  background: linear-gradient(135deg, #1a1a1a 0%, #0a0a0a 100%);
  border: 1px solid #00ff88;
  border-radius: 16px;
  padding: 24px;
  text-align: center;
  margin-bottom: 24px;
}

.info-card.highlight {
  box-shadow: 0 0 20px rgba(0, 255, 136, 0.2);
}

.card-icon {
  font-size: 48px;
  margin-bottom: 16px;
}

.info-card h2 {
  font-size: 20px;
  margin-bottom: 12px;
  color: #00ff88;
}

.info-card p {
  color: #ccc;
  line-height: 1.6;
}

.features-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
  margin-bottom: 32px;
}

.feature-card {
  background: #111;
  border: 1px solid #222;
  border-radius: 12px;
  padding: 16px;
  text-align: center;
}

.feature-icon {
  font-size: 32px;
  margin-bottom: 8px;
}

.feature-card h3 {
  font-size: 14px;
  font-weight: 600;
  margin-bottom: 8px;
  color: #00ff88;
}

.feature-card p {
  font-size: 12px;
  color: #888;
  line-height: 1.4;
}

.faq-section {
  margin-bottom: 32px;
}

.faq-section h2 {
  font-size: 18px;
  margin-bottom: 16px;
  color: #fff;
}

.faq-item {
  background: #111;
  border: 1px solid #222;
  border-radius: 12px;
  margin-bottom: 8px;
  overflow: hidden;
  cursor: pointer;
}

.faq-question {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px;
  font-weight: 500;
}

.faq-toggle {
  font-size: 24px;
  color: #00ff88;
  font-weight: 300;
}

.faq-answer {
  padding: 0 16px 16px 16px;
  color: #ccc;
  line-height: 1.6;
  animation: slideDown 0.3s ease;
}

@keyframes slideDown {
  from {
    opacity: 0;
    transform: translateY(-10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.faq-answer p {
  margin-bottom: 12px;
}

.faq-answer ul {
  margin: 12px 0;
  padding-left: 20px;
}

.faq-answer li {
  margin-bottom: 8px;
  color: #aaa;
}

.faq-answer code {
  background: #222;
  padding: 2px 6px;
  border-radius: 4px;
  font-family: 'Courier New', monospace;
  font-size: 12px;
  color: #00ff88;
}

.highlight-text {
  color: #00ff88 !important;
  font-weight: 500;
  margin-top: 12px;
}

.tech-details {
  margin-bottom: 32px;
}

.tech-details h2 {
  font-size: 18px;
  margin-bottom: 16px;
}

.spec-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
}

.spec-item {
  background: #111;
  border: 1px solid #222;
  border-radius: 8px;
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.spec-label {
  font-size: 11px;
  color: #888;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.spec-value {
  font-size: 14px;
  font-weight: 600;
  color: #00ff88;
}

.guarantees h2 {
  font-size: 18px;
  margin-bottom: 16px;
}

.guarantee-list {
  background: #111;
  border: 1px solid #222;
  border-radius: 12px;
  padding: 16px;
}

.guarantee-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 0;
  border-bottom: 1px solid #222;
}

.guarantee-item:last-child {
  border-bottom: none;
}

.check {
  color: #00ff88;
  font-size: 18px;
  font-weight: bold;
}
</style>
