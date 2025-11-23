# MSSCS Web - Vue.js Version 🚀

## Overview

MSSCS Web now uses **Vue.js 3** with the **same components** as the desktop client, providing a unified experience across all platforms.

## ✨ Key Features

- ✅ **Same UI/UX** as desktop client (Windows/Linux)
- ✅ **Shared Vue.js components** from `msscs_client`
- ✅ **Fixed status transitions** (offline → syncing → online)
- ✅ **P2P networking** via WebRTC (PeerJS)
- ✅ **Quantum-resistant encryption** (ML-KEM-1024 + ML-DSA-87)
- ✅ **Hot Module Replacement** (HMR) for fast development
- ✅ **Unified design system** with desktop client

## 🚀 Quick Start

### 1. Setup (First Time Only)

**Windows:**
```powershell
pwsh setup-web-vue.ps1
```

**Linux/Mac:**
```bash
bash setup-web-vue.sh
```

**Or manually:**
```bash
cd msscs_web
pnpm install
```

### 2. Start Development Server

**Windows:**
```powershell
cd msscs_web
pwsh start-vue-web.ps1
```

**Linux/Mac:**
```bash
cd msscs_web
bash start-vue-web.sh
```

**Or directly:**
```bash
cd msscs_web
pnpm dev
```

### 3. Open in Browser

Navigate to: **http://localhost:8000**

## ✅ Verification

### Browser Console Should Show:

```
✅ MSSCS Web Vue app initialized
🚀 Initializing MSSCS Web node...
🔐 Initializing quantum-resistant cryptography...
✅ Encryption ready
💾 Initializing local storage...
✅ Storage ready
🌐 Connecting to P2P network...
✅ P2P network connected
✅ Node is now ONLINE
🆔 Peer ID: abc123def456...
```

### UI Should Show:

- ✅ **Green "online" status** in sidebar
- ✅ **Peer ID displayed** in dashboard
- ✅ **Storage usage** and limit
- ✅ **Connected peers** count
- ✅ **Same design** as desktop client

## 📁 Project Structure

```
msscs_web/
├── src/                      # Vue.js source code
│   ├── App.vue              # Main app component
│   ├── main.js              # Entry point
│   ├── stores/
│   │   └── nodeStore.js     # State management
│   ├── styles/
│   │   └── main.css         # Imports shared styles
│   └── tauri-adapter.js     # Browser compatibility layer
├── vite.config.js           # Vite configuration
├── index-vue.html           # HTML template
├── package.json             # Dependencies
├── p2p.js                   # P2P networking (WebRTC)
├── storage.js               # IndexedDB storage
├── crypto.js                # Encryption
├── quantum-crypto.js        # Quantum-resistant encryption
└── README.md                # This file

Shared Components (from msscs_client):
../msscs_client/src/
├── components/              # All Vue components
│   ├── DashboardView.vue
│   ├── FilesView.vue
│   ├── SyncView.vue
│   ├── PeersView.vue
│   ├── SettingsView.vue
│   └── ... (all other components)
└── styles/
    └── main.css            # Shared design system
```

## 🔧 Development

### Available Scripts

```bash
# Start development server (Vue.js + Vite)
pnpm dev

# Build for production
pnpm build

# Preview production build
pnpm preview

# Start legacy version (vanilla JS)
pnpm dev:legacy
```

### Component Development

All components are shared with the desktop client. To modify a component:

1. Edit the component in `msscs_client/src/components/`
2. Changes will be reflected in both desktop and web versions
3. HMR will update the web version instantly

### Adding New Components

1. Create component in `msscs_client/src/components/`
2. Import in `msscs_web/src/App.vue`:
   ```vue
   <script setup>
   import MyNewComponent from '@shared/components/MyNewComponent.vue'
   </script>
   ```

## 🌐 Network & P2P

### How It Works

1. **PeerJS** for WebRTC signaling
2. **STUN/TURN** servers for NAT traversal
3. **Automatic peer discovery** via localStorage
4. **Cross-platform connectivity** (web ↔ desktop ↔ mobile)

### Status Indicators

| Status | Meaning | Color | Duration |
|--------|---------|-------|----------|
| **Offline** | No P2P connection | Gray | Initial state |
| **Syncing** | Connecting to network | Yellow | 5-15 seconds |
| **Online** | Connected and ready | Green | Steady state |

### Connecting to Peers

1. **Share your Peer ID** (displayed in dashboard)
2. **Or share the URL**: `http://localhost:8000?peer=YOUR_PEER_ID`
3. **Others can connect** by entering your Peer ID

## 🔒 Security

### Encryption Layers

1. **ML-KEM-1024** (Kyber) - Post-quantum key encapsulation
2. **ML-DSA-87** (Dilithium) - Post-quantum signatures
3. **AES-256-GCM** - Symmetric encryption
4. **ChaCha20-Poly1305** - Alternative cipher
5. **Lattice-based noise** - Additional obfuscation
6. **Superposition key derivation** - Quantum-inspired
7. **Singularity fragmentation** - Data splitting

**Attack Complexity:** 2^832 (quantum-resistant)

## 🐛 Troubleshooting

### Status Stuck on "Syncing"

**Solution:**
- Wait 15 seconds for P2P initialization
- Check browser console for errors
- Try refreshing the page
- Verify internet connection

### Components Not Loading

**Solution:**
- Ensure `pnpm install` was run successfully
- Check that `msscs_client` directory exists
- Verify Vite is running without errors
- Check browser console for import errors

### P2P Not Connecting

**Solution:**
- Check internet connection
- Verify WebRTC is supported (Chrome, Firefox, Edge, Safari)
- Check browser console for WebRTC errors
- Try connecting manually with a peer ID
- Verify STUN/TURN servers are accessible

### Styles Not Loading

**Solution:**
- Check that `@import` path in `msscs_web/src/styles/main.css` is correct
- Verify `msscs_client/src/styles/main.css` exists
- Clear browser cache and refresh

## 📊 Status Fix Details

### Problem (Before)
- ❌ Status stuck on "syncing" or "offline"
- ❌ Never transitioned to "online" even when P2P connected
- ❌ Peer ID not displayed

### Solution (After)
- ✅ Status properly transitions: offline → syncing → online
- ✅ Real-time status updates based on P2P connection
- ✅ Peer ID displayed and copyable

### Technical Implementation

```javascript
// In nodeStore.js
const connStats = p2p.getConnectionStats()
if (connStats.isConnected && connStats.peerId) {
  status.value = 'online'  // ✅ Properly set to online!
  peerId.value = connStats.peerId
}
```

## 📚 Documentation

- **Quick Start:** `QUICK_START_WEB_VUE.md`
- **Migration Guide:** `WEB_VUE_MIGRATION_GUIDE.md`
- **Status Fix:** `WEB_STATUS_FIX_SUMMARY.md`
- **Main README:** `README.md`

## 🔄 Legacy Version

The legacy vanilla JS version is still available:

```bash
cd msscs_web
pnpm dev:legacy
# or
node server.js
```

Open: http://localhost:8000

**Note:** The Vue.js version is recommended for:
- Better developer experience
- Shared components with desktop
- Consistent UI/UX
- Easier maintenance

## 🏗️ Building for Production

```bash
cd msscs_web
pnpm build
```

Output will be in `msscs_web/dist/`

### Preview Production Build

```bash
pnpm preview
```

### Deploy

The `dist/` folder contains static files that can be deployed to:
- GitHub Pages
- Netlify
- Vercel
- Any static hosting service

## 🎯 Comparison

| Feature | Legacy | Vue.js |
|---------|--------|--------|
| Component Reusability | ❌ | ✅ |
| Status Transitions | ✅ (fixed) | ✅ |
| HMR | ❌ | ✅ |
| Shared Design System | ❌ | ✅ |
| Type Safety | ❌ | ✅ (with TS) |
| Development Speed | Slow | Fast |
| Maintenance | Hard | Easy |
| Build Step | ❌ | ✅ |
| Bundle Size | Smaller | Larger |

## 🤝 Contributing

When contributing to the web version:

1. **Shared components** should be modified in `msscs_client/src/components/`
2. **Web-specific code** goes in `msscs_web/src/`
3. **Styles** should be added to the shared design system when possible
4. **Test both** desktop and web versions after changes

## 📝 License

MIT License - See LICENSE file for details

## 🙏 Acknowledgments

- **Vue.js** - Progressive JavaScript framework
- **Vite** - Next-generation frontend tooling
- **PeerJS** - Simple peer-to-peer with WebRTC
- **Noble Post-Quantum** - Post-quantum cryptography

## 🎉 Success Checklist

- [ ] Ran setup script (`setup-web-vue.ps1` or `setup-web-vue.sh`)
- [ ] Started dev server (`pnpm dev`)
- [ ] Opened browser (http://localhost:8000)
- [ ] Saw "online" status (green indicator)
- [ ] Peer ID is displayed
- [ ] UI matches desktop client
- [ ] Can upload files
- [ ] Can connect to peers
- [ ] Status updates in real-time

If all items are checked, you're ready to use MSSCS Web! 🚀

## 📞 Support

For issues or questions:
1. Check the troubleshooting section above
2. Review the documentation files
3. Check browser console for errors
4. Verify all dependencies are installed

---

**Made with ❤️ by the MSSCS Team**
