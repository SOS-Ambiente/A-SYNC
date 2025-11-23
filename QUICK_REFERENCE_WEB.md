# MSSCS Web - Quick Reference Card

## 🚀 Quick Start

```bash
# Setup (first time)
pwsh setup-web-vue.ps1      # Windows
bash setup-web-vue.sh       # Linux/Mac

# Start development
cd msscs_web
pnpm dev

# Open browser
http://localhost:8000
```

## ✅ Status Check

**Expected within 15 seconds:**
- ✅ Green "online" indicator
- ✅ Peer ID displayed
- ✅ Same UI as desktop

**Browser console should show:**
```
✅ Node is now ONLINE
🆔 Peer ID: abc123...
```

## 📁 Key Files

```
msscs_web/
├── src/App.vue           # Main Vue app
├── src/main.js           # Entry point
├── src/stores/nodeStore.js  # State management
├── vite.config.js        # Vite config
└── package.json          # Dependencies
```

## 🔧 Commands

```bash
# Development
pnpm dev              # Start Vue.js version
pnpm dev:legacy       # Start legacy version

# Build
pnpm build            # Build for production
pnpm preview          # Preview production build

# Setup
pnpm install          # Install dependencies
```

## 🐛 Troubleshooting

| Problem | Solution |
|---------|----------|
| Status stuck on "syncing" | Wait 15s, check console, refresh |
| Components not loading | Run `pnpm install` |
| P2P not connecting | Check internet, verify WebRTC support |
| Styles not loading | Clear cache, check import paths |

## 📊 Status Flow

```
Offline → Syncing → Online
   ↓         ↓         ↓
   └─────────┴─────────┘
    (Real-time updates)
```

## 🔗 Component Sharing

```javascript
// Import shared components
import Component from '@shared/components/Component.vue'

// Shared from msscs_client:
- DashboardView.vue
- FilesView.vue
- SyncView.vue
- PeersView.vue
- SettingsView.vue
- All other components
```

## 📚 Documentation

- `README_WEB_VUE.md` - Main docs
- `QUICK_START_WEB_VUE.md` - Quick start
- `WEB_VUE_MIGRATION_GUIDE.md` - Migration guide
- `WEB_STATUS_FIX_SUMMARY.md` - Status fix details

## 🎯 Key Features

- ✅ Same UI as desktop client
- ✅ Shared Vue.js components
- ✅ Fixed status transitions
- ✅ P2P networking (WebRTC)
- ✅ Quantum-resistant encryption
- ✅ Hot Module Replacement

## 🔒 Security

- **ML-KEM-1024** (Kyber)
- **ML-DSA-87** (Dilithium)
- **AES-256-GCM**
- **Attack complexity:** 2^832

## 🌐 Network

- **Protocol:** WebRTC (PeerJS)
- **NAT traversal:** STUN/TURN
- **Discovery:** Automatic (localStorage)
- **Cross-platform:** ✅ Web ↔ Desktop ↔ Mobile

## ✨ Benefits

| Feature | Legacy | Vue.js |
|---------|--------|--------|
| Component Reuse | ❌ | ✅ |
| Status Fix | ✅ | ✅ |
| HMR | ❌ | ✅ |
| Unified Design | ❌ | ✅ |

## 📞 Quick Help

**Status not online?**
1. Wait 15 seconds
2. Check browser console
3. Verify internet connection
4. Try refreshing page

**Components not working?**
1. Run `pnpm install`
2. Check `msscs_client` exists
3. Verify Vite is running
4. Check console for errors

**P2P not connecting?**
1. Check internet
2. Verify WebRTC support
3. Try manual peer connection
4. Check STUN/TURN servers

---

**Quick Start:** `pwsh setup-web-vue.ps1` → `pnpm dev` → http://localhost:8000
