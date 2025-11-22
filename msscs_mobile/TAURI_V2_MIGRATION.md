# Tauri v2 Migration Guide for MSSCS Mobile

## Overview
This guide covers the migration from Tauri v1.5 to Tauri v2.0 for Android support.

## What Changed

### 1. Dependencies Updated

**Cargo.toml:**
- `tauri-build = "2.0"` (was 1.5)
- `tauri = "2.0"` (was 1.5)
- Added `tauri-plugin-dialog = "2.0"`
- Added `tauri-plugin-shell = "2.0"`
- Removed manual Android dependencies (jni, ndk, ndk-context) - now handled by Tauri

**package.json:**
- `@tauri-apps/api = "^2.0.0"` (was ^1.5.0)
- `@tauri-apps/cli = "^2.0.0"` (was ^1.5.0)

### 2. Build Commands Updated

**New npm scripts:**
```json
"tauri:android:init": "tauri android init",
"tauri:android:dev": "tauri android dev",
"tauri:android:build": "tauri android build",
"tauri:android:build:apk": "tauri android build --apk"
```

### 3. Rust Code Changes Required

The main.rs file needs minimal changes for Tauri v2:

**Before (v1):**
```rust
use tauri::State;
```

**After (v2):**
```rust
use tauri::State;
// No changes needed for basic State usage
```

**Plugin usage changes:**
```rust
// v1: Built-in features
tauri = { version = "1.5", features = ["dialog-all", "shell-open"] }

// v2: Separate plugins
tauri-plugin-dialog = "2.0"
tauri-plugin-shell = "2.0"
```

### 4. Frontend API Changes

**Before (v1):**
```typescript
import { invoke } from '@tauri-apps/api/tauri';
import { open } from '@tauri-apps/api/dialog';
```

**After (v2):**
```typescript
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
```

## Migration Steps

### Step 1: Update Dependencies

```powershell
# In msscs_mobile directory
npm install
cd src-tauri
cargo update
```

### Step 2: Initialize Android Project

```powershell
npm run tauri:android:init
```

This will:
- Create `src-tauri/gen/android` directory
- Generate Android project structure
- Configure Gradle build files

### Step 3: Build APK

```powershell
# Debug build
npm run tauri:android:build:apk

# Release build
npm run tauri:android:build:apk -- --release
```

### Step 4: Update Frontend Code (if needed)

Check all imports in Vue files:

```typescript
// Update imports
import { invoke } from '@tauri-apps/api/core';

// Commands remain the same
await invoke('start_node');
```

## Android-Specific Configuration

### tauri.conf.json Changes

Tauri v2 uses a different configuration structure. The main changes:

1. **Identifier format:** Must be reverse domain notation (e.g., `com.msscs.mobile`)
2. **Android-specific settings:** Now in separate `android` section
3. **Permissions:** Declared explicitly in config

### Required Android Permissions

Add to `tauri.conf.json`:

```json
{
  "android": {
    "permissions": [
      "android.permission.INTERNET",
      "android.permission.ACCESS_NETWORK_STATE",
      "android.permission.READ_EXTERNAL_STORAGE",
      "android.permission.WRITE_EXTERNAL_STORAGE"
    ]
  }
}
```

## Build Script Updates

The `build-apk.ps1` script has been updated to:
- Use `npm run tauri:android:init` instead of `npm run tauri android init`
- Use `npm run tauri:android:build:apk` instead of `npm run tauri android build`
- Check for NDK installation explicitly

## Troubleshooting

### Error: "unrecognized subcommand 'android'"
**Solution:** You're using Tauri v1. Run `npm install` to upgrade to v2.

### Error: "NDK not found"
**Solution:** Install NDK via Android SDK Manager:
```powershell
$env:ANDROID_HOME\cmdline-tools\latest\bin\sdkmanager.bat "ndk;25.0.8775105"
```

### Error: "JAVA_HOME not set"
**Solution:** Set environment variable:
```powershell
[Environment]::SetEnvironmentVariable("JAVA_HOME", "C:\MSSCS_Tools\jdk", "User")
```

### Build fails with Gradle errors
**Solution:** Clean and rebuild:
```powershell
cd src-tauri/gen/android
.\gradlew clean
cd ../../..
npm run tauri:android:build:apk
```

## Testing

### On Emulator
```powershell
npm run tauri:android:dev
```

### On Physical Device
1. Enable USB debugging on device
2. Connect via USB
3. Run: `npm run tauri:android:dev`

### Install APK
```powershell
adb install src-tauri/gen/android/app/build/outputs/apk/debug/app-debug.apk
```

## Key Differences from v1

1. **Android support is native** - No need for Capacitor or Cordova
2. **Better Rust integration** - Direct access to Android APIs via JNI
3. **Smaller APK size** - Optimized build process
4. **Better performance** - Native rendering, no WebView overhead for Rust code
5. **Unified codebase** - Same Rust code for desktop and mobile

## Resources

- [Tauri v2 Documentation](https://v2.tauri.app/)
- [Tauri Android Guide](https://v2.tauri.app/develop/mobile/)
- [Migration Guide](https://v2.tauri.app/develop/migrate/)

## Next Steps

After successful migration:
1. Test all features on Android device
2. Update UI for mobile screen sizes
3. Add mobile-specific features (camera, GPS, etc.)
4. Optimize performance for mobile
5. Test on different Android versions (API 24+)
