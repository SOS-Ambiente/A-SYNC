import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import './styles/main.css'
import { PeerJSBridge } from './peerjs-bridge'

const app = createApp(App)
const pinia = createPinia()

app.use(pinia)
app.mount('#app')

// Initialize PeerJS bridge for cross-platform P2P
const peerBridge = new PeerJSBridge();

// Make it globally available
(window as any).peerBridge = peerBridge;

// Initialize on app start
peerBridge.init()
    .then(peerId => {
        console.log('✅ PeerJS Bridge initialized with ID:', peerId);
    })
    .catch(err => {
        console.warn('⚠️  PeerJS Bridge initialization failed:', err);
        console.log('💡 App will work without WebRTC peer connections');
    });
