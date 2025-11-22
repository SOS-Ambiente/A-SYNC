# MSSCS Mobile - Quick Start

Get your Android APK built automatically!

## 🚀 FULLY AUTOMATED BUILD (Recommended)

**This script does EVERYTHING for you:**
- Downloads and installs Node.js, Rust, Java, Android SDK/NDK
- Sets up all environment variables
- Installs all dependencies
- Builds the APK

### Run This One Command:

```powershell
# Right-click PowerShell → Run as Administrator
cd msscs_mobile
.\setup-and-build.ps1
```

**That's it!** The script will:
1. Download all required tools (~2 GB)
2. Install everything automatically
3. Set up your environment
4. Build the APK

**Time:** 20-30 minutes (mostly downloading)  
**Result:** Ready-to-install APK file

---

## 📋 Manual Build (If You Have Prerequisites)

If you already have Node.js, Rust, Java, and Android SDK installed:

```powershell
# 1. Navigate to mobile project
cd msscs_mobile

# 2. Install dependencies
npm install

# 3. Add Android targets
rustup target add aarch64-linux-android armv7-linux-androideabi

# 4. Initialize Android project
npm run tauri android init

# 5. Build APK
npm run tauri android build
```

---

## ✅ Prerequisites Check (Manual Build Only)

Run these commands to verify you have everything:

```bash
node --version    # Need v18+
cargo --version   # Need Rust
java -version     # Need JDK 11+
```

If any are missing, use the automated script above or see BUILD_GUIDE.md.

## Find Your APK

After successful build:
```
msscs_mobile/src-tauri/gen/android/app/build/outputs/apk/debug/app-debug.apk
```

## Install on Device

### Method 1: USB
```bash
adb install src-tauri/gen/android/app/build/outputs/apk/debug/app-debug.apk
```

### Method 2: File Transfer
1. Copy APK to your phone
2. Open file manager
3. Tap APK file
4. Allow "Install from Unknown Sources"
5. Install

## First Run

1. Open MSSCS Mobile app
2. App will start local node automatically
3. Go to "Nodes" tab
4. Tap scan icon to discover local MSSCS nodes
5. Connect to discovered nodes
6. Go to "Files" tab
7. Upload your first file!

## Connect to Desktop Node

Make sure your desktop MSSCS node is running:

```bash
cd msscs_v4
cargo run -- --port 8080
```

The mobile app will auto-discover it on the same WiFi network.

## Common Issues

### "Android SDK not found"
Set environment variable:
```
ANDROID_HOME=C:\Users\YourUsername\AppData\Local\Android\Sdk
```

### "Build failed"
Try:
```bash
cd src-tauri/gen/android
./gradlew clean
cd ../../..
npm run tauri android build
```

### "Device not found"
Enable USB debugging on your Android device:
1. Settings → About Phone
2. Tap "Build Number" 7 times
3. Settings → Developer Options
4. Enable "USB Debugging"

## What's Next?

- Read BUILD_GUIDE.md for detailed instructions
- Check FEATURES.md for app capabilities
- See ARCHITECTURE.md for technical details

## Quick Commands

```bash
# Build debug APK
npm run tauri android build

# Build release APK
npm run tauri android build -- --release

# Run on emulator/device
npm run tauri android dev

# Check connected devices
adb devices

# View logs
adb logcat | grep MSSCS
```

## Network Discovery

The app uses mDNS to find MSSCS nodes automatically. Make sure:
- Mobile and desktop are on same WiFi
- Firewall allows mDNS (port 5353)
- MSSCS node is running

## File Viewing

Supported formats:
- **Images**: JPG, PNG, GIF, WebP (built-in viewer)
- **Videos**: MP4, WebM, AVI (built-in player)
- **Text**: TXT, MD, LOG (built-in viewer)
- **Others**: Opens with system default app

## Need Help?

1. Check BUILD_GUIDE.md for detailed setup
2. Check logs: `adb logcat`
3. Verify node is running: `curl http://localhost:8080/health`
4. Test network: `ping <desktop-ip>`

Happy building! 🚀
