# Quick Start - MSSCS Web (Vue.js Version)

## 🚀 Get Started in 3 Steps

### 1. Install Dependencies

```bash
cd msscs_web
pnpm install
```

### 2. Start Development Server

**Windows:**
```powershell
pwsh start-vue-web.ps1
```

**Linux/Mac:**
```bash
bash start-vue-web.sh
```

**Or directly:**
```bash
pnpm dev
```

### 3. Open in Browser

Navigate to: **http://localhost:8000**

## ✅ What to Expect

1. **Initialization (5-15 seconds)**
   - Quantum encryption setup
   - P2P network connection
   - Storage initialization

2. **Status Transitions**
   ```
   Offline → Syncing → Online ✅
   ```

3. **UI Features**
   - Same design as desktop client
   - Dashboard with quick stats
   - File management
   - Peer connections
   - Network statistics

## 🔍 Verify It's Working

### Check Browser Console

You should see:
```
✅ MSSCS Web Vue app initialized
🚀 Initializing MSSCS Web node...
✅ Encryption ready
✅ Storage ready
✅ P2P network connected
✅ Node is now ONLINE
🆔 Peer ID: [your-peer-id]
```

### Check UI

- **Sidebar Status:** Should show "online" (green indicator)
- **Peer ID:** Displayed in dashboard
- **Storage:** Shows usage and limit
- **Peers:** Shows connected peer count

## 🐛 Troubleshooting

### Status Stuck on "Syncing"
- Wait 15 seconds for P2P initialization
- Check browser console for errors
- Try refreshing the page

### Components Not Loading
- Ensure you ran `pnpm install`
- Check that `msscs_client` directory exists
- Verify Vite is running without errors

### P2P Not Connecting
- Check internet connection
- Verify WebRTC is supported in your browser
- Try connecting manually with a peer ID

## 📚 More Information

- Full migration guide: `WEB_VUE_MIGRATION_GUIDE.md`
- Legacy version: `pnpm dev:legacy`
- Build for production: `pnpm build`

## 🎉 Success!

If you see:
- ✅ Green "online" status
- ✅ Peer ID displayed
- ✅ Same UI as desktop client

You're ready to use MSSCS Web!
