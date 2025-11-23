import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import './styles/main.css'

// Import web-specific P2P modules
import { P2PNetwork } from '../p2p.js'
import { StorageManager } from '../storage.js'
import { CryptoManager } from '../crypto.js'
import { QuantumCryptoManager } from '../quantum-crypto.js'
import { ErasureCoding } from '../erasure-coding.js'
import { WebRTCBridge } from '../webrtc-bridge.js'

const app = createApp(App)
const pinia = createPinia()

app.use(pinia)

// Make P2P modules globally available
window.P2PNetwork = P2PNetwork
window.StorageManager = StorageManager
window.CryptoManager = CryptoManager
window.QuantumCryptoManager = QuantumCryptoManager
window.ErasureCoding = ErasureCoding
window.WebRTCBridge = WebRTCBridge

app.mount('#app')

console.log('✅ MSSCS Web Vue app initialized')
