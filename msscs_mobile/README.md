# MSSCS Mobile Client (Android)

Mobile Android client for MSSCS v4.0 - Multi-State Chain-based Secure Storage

## 🚀 ONE-CLICK BUILD (Recommended)

**Don't have Node.js, Rust, Java, or Android SDK?**  
**No problem! This script installs EVERYTHING automatically:**

```powershell
# Open PowerShell as Administrator
cd msscs_mobile
.\setup-and-build.ps1
```

**That's it!** The script will:
- ✅ Download and install all required tools
- ✅ Set up your environment
- ✅ Build the APK automatically

**Time:** 20-30 minutes  
**Result:** Ready-to-install `MSSCS-Mobile.apk`

👉 **See [RUN_THIS.md](RUN_THIS.md) for detailed instructions**

---

## Features

- 📱 Native Android APK
- 🔍 **Automatic local network node discovery** (mDNS)
- 📁 File browsing and management
- 🖼️ **Built-in image viewer** (JPEG, PNG, GIF, WebP) with zoom/pan
- 🎥 **Built-in video player** (MP4, WebM, AVI)
- 📄 **Built-in text viewer** (TXT, MD, LOG)
- 🔗 Open any file with Android system apps (PDF, DOC, ZIP, etc.)
- 🌐 P2P network connectivity (Kademlia DHT)
- 🔒 End-to-end encryption (AES-256-GCM)
- 📊 Real-time sync status and metrics
- 🎨 AMOLED dark theme with neon accents

---

## Quick Start Options

### Option 1: Fully Automated (Recommended)
```powershell
.\setup-and-build.ps1
```
Downloads and installs everything, then builds APK.

### Option 2: Manual Build (If you have prerequisites)
```bash
npm install
npm run tauri android init
npm run tauri android build
```

### Option 3: Development Mode
```bash
npm run tauri android dev
```
Runs on emulator/device with hot reload.

---

## Build Requirements (Manual Build Only)

- Node.js 18+
- Rust 1.70+
- Android SDK (API 24+)
- Android NDK 25.0.8775105
- Java JDK 11+

**Don't have these?** Use the automated script instead!

## Architecture

```
Mobile App (Android)
    ↓
Tauri Mobile (Rust Backend)
    ↓
MSSCS v4 Core Library
    ↓
Local Network Discovery (mDNS)
    ↓
Connect to MSSCS Nodes (P2P)
```

## Network Discovery

The app automatically discovers MSSCS nodes on the local network using mDNS/Bonjour:
- Service type: `_msscs._tcp.local`
- Scans every 10 seconds
- Connects to discovered nodes automatically

## File Viewing

- **Images**: Built-in image viewer with zoom/pan
- **Videos**: Native video player
- **PDFs**: WebView-based PDF viewer
- **Text**: Built-in text viewer
- **Others**: Opens with system default apps

## Permissions

Required Android permissions:
- `INTERNET` - Network communication
- `ACCESS_NETWORK_STATE` - Network status
- `READ_EXTERNAL_STORAGE` - File access
- `WRITE_EXTERNAL_STORAGE` - File downloads
- `CAMERA` - QR code scanning (optional)

## Configuration

Edit `src-tauri/tauri.conf.json` to customize:
- App name and identifier
- Permissions
- Window settings
- Build options
