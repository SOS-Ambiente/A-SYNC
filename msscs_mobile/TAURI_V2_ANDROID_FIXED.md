# MSSCS Mobile - Tauri v2 Android Build - FIXED ✅

## What Was Fixed

The Android build was failing with `error: no library targets found in package` because Tauri v2 mobile builds require a library crate structure, not just a binary.

### Changes Made:

1. **Cargo.toml** - Added `[lib]` section:
   ```toml
   [lib]
   name = "msscs_mobile_lib"
   crate-type = ["lib", "cdylib", "staticlib"]
   ```

2. **src/lib.rs** - Created library entry point with `#[cfg_attr(mobile, tauri::mobile_entry_point)]`
   - Contains all the application logic
   - Exports `pub fn run()` function
   - Uses mobile entry point attribute for Android/iOS

3. **src/main.rs** - Simplified to desktop entry point:
   ```rust
   fn main() {
       msscs_mobile_lib::run();
   }
   ```

4. **build.rs** - Created build script:
   ```rust
   fn main() {
       tauri_build::build()
   }
   ```

5. **tauri.conf.json** - Added Android configuration:
   ```json
   "bundle": {
     "android": {
       "minSdkVersion": 24
     }
   }
   ```

6. **Added `dirs` dependency** for cross-platform data directory support

## Build Instructions

### Quick Build (Recommended)
```powershell
cd msscs_mobile
.\build-android-v2.ps1
```

### Manual Build Steps

1. **Install dependencies:**
   ```powershell
   npm install
   ```

2. **Build frontend:**
   ```powershell
   npm run build
   ```

3. **Initialize Android (first time only):**
   ```powershell
   npm run android:init
   ```

4. **Build APK:**
   ```powershell
   npm run android:build:apk
   ```

5. **Or build AAB (for Play Store):**
   ```powershell
   npm run android:build:aab
   ```

## Output Location

APK: `src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release-unsigned.apk`

AAB: `src-tauri/gen/android/app/build/outputs/bundle/universalRelease/app-universal-release.aab`

## Architecture

### Tauri v2 Mobile Structure
```
src-tauri/
├── src/
│   ├── lib.rs          # Mobile entry point (library)
│   ├── main.rs         # Desktop entry point (binary)
│   ├── network_discovery.rs
│   └── file_viewer.rs
├── build.rs            # Build script
├── Cargo.toml          # With [lib] section
└── gen/
    └── android/        # Generated Android project
```

### Key Differences from v1:
- **v1**: Single `main.rs` with binary target
- **v2**: Separate `lib.rs` (mobile) and `main.rs` (desktop)
- **v2**: Requires `[lib]` section in Cargo.toml
- **v2**: Uses `#[cfg_attr(mobile, tauri::mobile_entry_point)]`

## Verification

To verify the build works:

```powershell
# Check library target exists
cargo metadata --manifest-path src-tauri/Cargo.toml | Select-String "lib"

# Test desktop build
cd src-tauri
cargo build

# Test Android build
cd ..
npm run android:build:apk
```

## Troubleshooting

### Issue: "no library targets found"
**Solution**: Ensure `[lib]` section exists in Cargo.toml

### Issue: "cannot find function `run`"
**Solution**: Ensure lib.rs exports `pub fn run()`

### Issue: Android SDK not found
**Solution**: Set ANDROID_HOME environment variable:
```powershell
$env:ANDROID_HOME = "C:\Users\YourUser\AppData\Local\Android\Sdk"
```

### Issue: NDK not found
**Solution**: Install NDK via Android Studio SDK Manager (version 25.x recommended)

## Testing on Device

1. Enable USB debugging on Android device
2. Connect device via USB
3. Run: `npm run android:dev`

Or install the APK manually:
```powershell
adb install src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release-unsigned.apk
```

## Next Steps

- Sign the APK for production release
- Test on multiple Android versions (API 24+)
- Optimize bundle size
- Add app icons and splash screens
- Configure permissions in AndroidManifest.xml

## Status: ✅ READY FOR BUILD

The project is now properly configured for Tauri v2 Android builds!
