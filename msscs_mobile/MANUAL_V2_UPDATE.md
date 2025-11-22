# Manual Tauri v2 Update Guide

## Files Already Updated ✅

The following files have been automatically updated for Tauri v2:

### 1. `package.json` ✅
```json
{
  "dependencies": {
    "@tauri-apps/api": "^2.0.0"  // Changed from ^1.5.0
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2.0.0"  // Changed from ^1.5.0
  },
  "scripts": {
    "tauri:android:init": "tauri android init",
    "tauri:android:dev": "tauri android dev",
    "tauri:android:build": "tauri android build",
    "tauri:android:build:apk": "tauri android build --apk"
  }
}
```

### 2. `src-tauri/Cargo.toml` ✅
```toml
[workspace]
# This prevents the package from being part of the parent workspace

[package]
name = "msscs-mobile"
version = "1.0.0"
description = "MSSCS Mobile Client"
authors = ["MSSCS Team"]
edition = "2021"

[build-dependencies]
tauri-build = { version = "2.0", features = [] }

[dependencies]
tauri = { version = "2.0", features = [] }
tauri-plugin-dialog = "2.0"
tauri-plugin-shell = "2.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1", features = ["full"] }
msscs_v4 = { path = "../../msscs_v4" }

# Network discovery
mdns-sd = "0.10"
local-ip-address = "0.5"

# File handling
mime_guess = "2.0"
base64 = "0.21"

# Android-specific (handled by Tauri v2)

[features]
default = ["custom-protocol"]
custom-protocol = ["tauri/custom-protocol"]
```

### 3. Root `Cargo.toml` ✅
```toml
[workspace]
members = [
    "msscs_v4",
    "msscs_client/src-tauri"
]
exclude = [
    "msscs_mobile/src-tauri"  // Added to prevent workspace conflicts
]
resolver = "2"
```

### 4. `build-apk.ps1` ✅
Updated to use new Tauri v2 commands:
- `npm run tauri:android:init`
- `npm run tauri:android:build:apk`

## Files That Need Manual Updates

### 1. `src-tauri/src/main.rs` - Plugin Registration

**Current code works but can be optimized for v2:**

```rust
// Add at the top if using dialog plugin
use tauri_plugin_dialog;
use tauri_plugin_shell;

fn main() {
    let app_state: Arc<RwLock<Option<AppStateWrapper>>> = Arc::new(RwLock::new(None));

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())  // Add if using dialogs
        .plugin(tauri_plugin_shell::init())   // Add if using shell
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            start_node,
            discover_nodes,
            connect_to_node,
            list_files,
            upload_file,
            download_file,
            preview_file,
            open_with_system,
            delete_file,
            get_metrics,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 2. Frontend Files - API Import Changes

**Update all Vue/TypeScript files that use Tauri APIs:**

#### `src/main.ts`
```typescript
// OLD (v1):
import { invoke } from '@tauri-apps/api/tauri';

// NEW (v2):
import { invoke } from '@tauri-apps/api/core';
```

#### `src/stores/nodeStore.ts`
```typescript
// OLD (v1):
import { invoke } from '@tauri-apps/api/tauri';

// NEW (v2):
import { invoke } from '@tauri-apps/api/core';
```

#### `src/stores/filesStore.ts`
```typescript
// OLD (v1):
import { invoke } from '@tauri-apps/api/tauri';

// NEW (v2):
import { invoke } from '@tauri-apps/api/core';
```

#### If using dialog plugin:
```typescript
// OLD (v1):
import { open, save } from '@tauri-apps/api/dialog';

// NEW (v2):
import { open, save } from '@tauri-apps/plugin-dialog';
```

#### If using shell plugin:
```typescript
// OLD (v1):
import { open } from '@tauri-apps/api/shell';

// NEW (v2):
import { open } from '@tauri-apps/plugin-shell';
```

### 3. `src-tauri/tauri.conf.json` - Configuration Updates

**Add Android-specific configuration:**

```json
{
  "productName": "MSSCS Mobile",
  "version": "1.0.0",
  "identifier": "com.msscs.mobile",
  "build": {
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build",
    "devUrl": "http://localhost:1420",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [
      {
        "title": "MSSCS Mobile",
        "width": 400,
        "height": 800,
        "minWidth": 360,
        "minHeight": 640,
        "resizable": true,
        "fullscreen": false
      }
    ],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  },
  "plugins": {
    "dialog": {
      "all": true
    },
    "shell": {
      "open": true
    }
  }
}
```

## Step-by-Step Manual Update Process

### Step 1: Clean Dependencies
```powershell
# Remove old dependencies
Remove-Item -Recurse -Force node_modules
Remove-Item -Recurse -Force src-tauri/target

# Install new dependencies
npm install
cd src-tauri
cargo update
cd ..
```

### Step 2: Update Frontend Imports

Search and replace in all `.ts` and `.vue` files:

```powershell
# Find files that need updating
Get-ChildItem -Path src -Recurse -Include *.ts,*.vue | Select-String "@tauri-apps/api/tauri"
```

Replace:
- `@tauri-apps/api/tauri` → `@tauri-apps/api/core`
- `@tauri-apps/api/dialog` → `@tauri-apps/plugin-dialog`
- `@tauri-apps/api/shell` → `@tauri-apps/plugin-shell`

### Step 3: Update Rust Code (Optional but Recommended)

Add plugin initialization in `src-tauri/src/main.rs`:

```rust
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        // ... rest of your code
}
```

### Step 4: Initialize Android Project

```powershell
npm run tauri:android:init
```

This creates `src-tauri/gen/android/` with the Android project structure.

### Step 5: Build APK

```powershell
# Debug build
npm run tauri:android:build:apk

# Release build
npm run tauri:android:build:apk -- --release
```

## Verification Checklist

- [ ] `npm install` completes without errors
- [ ] `cargo update` completes without errors (in src-tauri directory)
- [ ] All TypeScript imports updated to v2 API
- [ ] `npm run tauri:android:init` succeeds
- [ ] `npm run tauri:android:build:apk` succeeds
- [ ] APK file created in `src-tauri/gen/android/app/build/outputs/apk/`
- [ ] APK installs and runs on Android device

## Common Issues and Solutions

### Issue: "unrecognized subcommand 'android'"
**Solution:** Tauri CLI not updated. Run `npm install` again.

### Issue: Workspace conflict error
**Solution:** Already fixed - `msscs_mobile/src-tauri` excluded from parent workspace.

### Issue: Import errors in TypeScript
**Solution:** Update imports from `@tauri-apps/api/tauri` to `@tauri-apps/api/core`.

### Issue: Build fails with "NDK not found"
**Solution:** Install NDK:
```powershell
$env:ANDROID_HOME\cmdline-tools\latest\bin\sdkmanager.bat "ndk;25.0.8775105"
```

### Issue: Gradle build fails
**Solution:** Clean and rebuild:
```powershell
cd src-tauri/gen/android
.\gradlew clean
cd ../../..
npm run tauri:android:build:apk
```

## Testing

### Desktop (Windows)
```powershell
npm run tauri dev
```

### Android Emulator
```powershell
npm run tauri:android:dev
```

### Android Device (USB)
```powershell
# Enable USB debugging on device
npm run tauri:android:dev
```

## Next Steps After Migration

1. **Test all features** on Android device
2. **Update UI** for mobile screen sizes and touch interactions
3. **Add mobile permissions** in tauri.conf.json as needed
4. **Optimize performance** for mobile devices
5. **Test on multiple Android versions** (API 24+)

## Resources

- [Tauri v2 Documentation](https://v2.tauri.app/)
- [Tauri v2 Migration Guide](https://v2.tauri.app/develop/migrate/)
- [Tauri Android Guide](https://v2.tauri.app/develop/mobile/)
- [API Reference](https://v2.tauri.app/reference/)

## Quick Reference

### Build Commands
```powershell
# Initialize Android
npm run tauri:android:init

# Build debug APK
npm run tauri:android:build:apk

# Build release APK
npm run tauri:android:build:apk -- --release

# Dev mode (hot reload)
npm run tauri:android:dev

# Desktop dev
npm run tauri dev
```

### APK Location
- Debug: `src-tauri/gen/android/app/build/outputs/apk/debug/app-debug.apk`
- Release: `src-tauri/gen/android/app/build/outputs/apk/release/app-release.apk`
