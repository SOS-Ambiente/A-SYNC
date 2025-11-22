# 🚀 BUILD MSSCS MOBILE APK - START HERE

## ONE-CLICK AUTOMATED BUILD

This will download and install **EVERYTHING** you need, then build the APK.

### Step 1: Open PowerShell as Administrator

1. Press `Windows Key`
2. Type `PowerShell`
3. **Right-click** on "Windows PowerShell"
4. Click **"Run as Administrator"**

### Step 2: Navigate to Project

```powershell
cd path\to\msscs_mobile
```

### Step 3: Run the Setup Script

```powershell
.\setup-and-build.ps1
```

### That's It!

The script will:
- ✅ Download Node.js, Rust, Java, Android SDK/NDK
- ✅ Install everything automatically
- ✅ Set up environment variables
- ✅ Install project dependencies
- ✅ Build the APK

**Time:** 20-30 minutes  
**Output:** `MSSCS-Mobile.apk` ready to install

---

## What Gets Installed

| Tool | Version | Size | Purpose |
|------|---------|------|---------|
| Node.js | 20.11.0 | ~50 MB | JavaScript runtime |
| Rust | Latest | ~400 MB | Backend language |
| Java JDK | 17 | ~300 MB | Android build tools |
| Android SDK | 33 | ~1 GB | Android platform |
| Android NDK | 25.0.8775105 | ~1 GB | Native development |
| Build Tools | 33.0.0 | ~100 MB | Compilation tools |

**Total:** ~3 GB download, ~5 GB installed

---

## Installation Locations

Everything is installed to:
```
C:\MSSCS_Tools\
├── nodejs\          (if not already installed)
├── jdk\             (Java JDK 17)
└── Android\
    └── sdk\         (Android SDK + NDK)
```

Your user profile:
```
%USERPROFILE%\.cargo\    (Rust)
```

---

## After Build Completes

You'll get a file named:
```
MSSCS-Mobile-YYYYMMDD.apk
```

### Install on Android Device

**Method 1: USB (Recommended)**
1. Enable USB Debugging on your Android device
2. Connect via USB
3. Script will offer to install automatically

**Method 2: File Transfer**
1. Copy APK to your phone
2. Open file manager
3. Tap the APK file
4. Allow "Install from Unknown Sources"
5. Install

---

## Troubleshooting

### "Script execution is disabled"
Run this first:
```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

### "Access denied"
Make sure you're running PowerShell **as Administrator**

### Build fails
1. Close PowerShell
2. Open new PowerShell as Administrator
3. Run the script again

### Still having issues?
See `BUILD_GUIDE.md` for detailed manual instructions

---

## What's Next?

After installing the APK:

1. **Open MSSCS Mobile** on your Android device
2. **Go to "Nodes" tab** and tap the scan icon 🔍
3. **Connect to discovered nodes** on your network
4. **Go to "Files" tab** and start uploading!

The app will automatically discover MSSCS nodes on your WiFi network.

---

## Need Help?

- **Quick Start:** `QUICKSTART.md`
- **Detailed Guide:** `BUILD_GUIDE.md`
- **Features:** `FEATURES.md`
- **Technical Docs:** `MOBILE_COMPLETE.md`

---

## Summary

```powershell
# Just run this:
.\setup-and-build.ps1

# Wait 20-30 minutes

# Get: MSSCS-Mobile.apk

# Install on Android

# Done! 🎉
```

**It's that simple!**
