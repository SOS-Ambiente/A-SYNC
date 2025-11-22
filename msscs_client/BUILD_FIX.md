# Windows Build Fix - Node Offline Issue

## Problem
When building the Tauri app for Windows, the exe showed "Node Status: offline" and files never loaded.

## Root Causes
1. **Config file path**: The app was looking for `config.toml` in the current working directory, which changes in production builds
2. **Data directory**: The default data directory `./msscs_data` was relative and not accessible in production
3. **No error visibility**: Initialization errors were only logged to console

## Changes Made

### 1. Fixed Path Resolution (main.rs)
- Added platform-specific app data directory detection:
  - Windows: `%APPDATA%\MSSCS`
  - macOS: `~/Library/Application Support/MSSCS`
  - Linux: `~/.msscs`
- Config and data now stored in proper app data locations
- Directories are created automatically if they don't exist

### 2. Added Dependencies (Cargo.toml)
- Added `dirs = "5.0"` for cross-platform directory detection

### 3. Improved Error Handling (nodeStore.ts)
- Added console logging for initialization steps
- Added user-visible error alerts when node fails to start
- Better debugging information

### 4. Fixed File Loading (FilesView.vue)
- Files now load only after node is online
- Added watcher to reload files when node status changes
- Prevents "Node not started" errors

## How to Rebuild

1. Clean previous build artifacts:
```powershell
cd msscs_client
Remove-Item -Recurse -Force src-tauri/target -ErrorAction SilentlyContinue
```

2. Install dependencies:
```powershell
pnpm install
```

3. Build the app:
```powershell
pnpm tauri build
```

4. The exe will be in: `src-tauri/target/release/msscs-client.exe`

## Testing the Fix

1. Run the built exe
2. Check that "Node Status" shows "online" (may take a few seconds)
3. Verify files load in the Files view
4. Check app data location:
   - Windows: `%APPDATA%\MSSCS\`
   - Should contain `config.toml` and `data/` folder

## If Issues Persist

1. Check the console output (run from terminal to see logs)
2. Verify the app data directory is writable
3. Check Windows Firewall isn't blocking the app
4. Ensure no antivirus is interfering with file operations

## Additional Notes

- First run will create default config automatically
- Data persists between app restarts
- Config can be edited at `%APPDATA%\MSSCS\config.toml`
