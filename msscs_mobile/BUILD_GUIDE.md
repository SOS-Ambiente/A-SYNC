# MSSCS Mobile - Android Build Guide

Complete guide to building the MSSCS Mobile APK for Android.

## Prerequisites

### 1. Install Node.js
```bash
# Download from https://nodejs.org/ (v18 or higher)
node --version  # Should be v18+
npm --version
```

### 2. Install Rust
```bash
# Windows (PowerShell)
winget install Rustlang.Rustup

# Or download from https://rustup.rs/
rustup --version
cargo --version
```

### 3. Install Android Studio
1. Download from https://developer.android.com/studio
2. Install Android Studio
3. Open Android Studio → SDK Manager
4. Install:
   - Android SDK Platform 33
   - Android SDK Build-Tools 33.0.0
   - Android SDK Command-line Tools
   - Android SDK Platform-Tools
   - NDK (Side by side) version 25.0.8775105

### 4. Install Java JDK
```bash
# Windows
winget install Oracle.JDK.17

# Verify
java -version  # Should be 11 or higher
```

### 5. Set Environment Variables (Windows)

Add to System Environment Variables:
```
ANDROID_HOME = C:\Users\YourUsername\AppData\Local\Android\Sdk
JAVA_HOME = C:\Program Files\Java\jdk-17

Add to PATH:
%ANDROID_HOME%\platform-tools
%ANDROID_HOME%\cmdline-tools\latest\bin
%ANDROID_HOME%\build-tools\33.0.0
%JAVA_HOME%\bin
```

## Build Steps

### 1. Install Dependencies
```bash
cd msscs_mobile
npm install
```

### 2. Initialize Tauri Android
```bash
npm run tauri android init
```

This will:
- Create Android project structure
- Configure Gradle
- Set up NDK toolchains

### 3. Add Android Targets to Rust
```bash
rustup target add aarch64-linux-android
rustup target add armv7-linux-androideabi
rustup target add i686-linux-android
rustup target add x86_64-linux-android
```

### 4. Build APK (Debug)
```bash
npm run tauri android build
```

The APK will be generated at:
```
src-tauri/gen/android/app/build/outputs/apk/debug/app-debug.apk
```

### 5. Build APK (Release)
```bash
npm run tauri android build -- --release
```

For signed release APK, you need a keystore:
```bash
# Generate keystore
keytool -genkey -v -keystore msscs-release.keystore -alias msscs -keyalg RSA -keysize 2048 -validity 10000

# Build signed APK
npm run tauri android build -- --release --keystore msscs-release.keystore --keystore-password YOUR_PASSWORD
```

## Testing

### Test on Emulator
```bash
# Start Android emulator from Android Studio
# Then run:
npm run tauri android dev
```

### Test on Physical Device
1. Enable Developer Options on your Android device
2. Enable USB Debugging
3. Connect device via USB
4. Run:
```bash
adb devices  # Verify device is connected
npm run tauri android dev
```

### Install APK Manually
```bash
adb install src-tauri/gen/android/app/build/outputs/apk/debug/app-debug.apk
```

## Troubleshooting

### Error: "Android SDK not found"
- Verify ANDROID_HOME is set correctly
- Restart terminal/IDE after setting environment variables

### Error: "NDK not found"
- Install NDK from Android Studio SDK Manager
- Ensure NDK version 25.0.8775105 is installed

### Error: "Rust target not found"
```bash
rustup target add aarch64-linux-android
```

### Error: "Gradle build failed"
- Check Java version: `java -version`
- Clear Gradle cache:
```bash
cd src-tauri/gen/android
./gradlew clean
```

### Error: "Permission denied"
```bash
# On Linux/Mac, make gradlew executable
chmod +x src-tauri/gen/android/gradlew
```

## APK Size Optimization

### 1. Enable ProGuard (Minification)
Edit `src-tauri/gen/android/app/build.gradle`:
```gradle
buildTypes {
    release {
        minifyEnabled true
        proguardFiles getDefaultProguardFile('proguard-android-optimize.txt'), 'proguard-rules.pro'
    }
}
```

### 2. Split APKs by Architecture
```gradle
splits {
    abi {
        enable true
        reset()
        include 'armeabi-v7a', 'arm64-v8a', 'x86', 'x86_64'
        universalApk false
    }
}
```

### 3. Strip Debug Symbols
In `Cargo.toml`:
```toml
[profile.release]
strip = true
opt-level = "z"
lto = true
codegen-units = 1
```

## Distribution

### Google Play Store
1. Build signed release APK
2. Create app listing at https://play.google.com/console
3. Upload APK
4. Fill in store listing details
5. Submit for review

### Direct Distribution
1. Build release APK
2. Host on your website
3. Users must enable "Install from Unknown Sources"

## Network Configuration

The app automatically discovers MSSCS nodes on the local network using mDNS.

To manually add nodes:
1. Open app
2. Go to "Nodes" tab
3. Tap "Add Manual Node"
4. Enter IP:PORT (e.g., 192.168.1.100:8080)

## File Permissions

The app requires storage permissions to:
- Upload files from device
- Download files to device
- Open files with system apps

Permissions are requested at runtime when needed.

## Performance Tips

1. **Reduce APK size**: Use split APKs for different architectures
2. **Faster builds**: Use `--target aarch64-linux-android` for single architecture
3. **Debug faster**: Use `npm run tauri android dev` for hot reload

## Next Steps

After building:
1. Test on multiple devices
2. Test network discovery
3. Test file upload/download
4. Test file viewing
5. Test with real MSSCS nodes
6. Optimize performance
7. Submit to Play Store

## Support

For issues:
- Check Tauri docs: https://tauri.app/v2/guides/building/android
- MSSCS issues: GitHub repository
- Android issues: Stack Overflow
