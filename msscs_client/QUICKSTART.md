# Quick Start Guide

Get the MSSCS Desktop Client running in 5 minutes!

## Step 1: Prerequisites

Install these if you haven't already:

### Node.js & npm
```bash
# Download from https://nodejs.org/
# Or use a package manager:
winget install OpenJS.NodeJS
```

### Rust
```bash
# Download from https://rustup.rs/
# Or run:
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Verify installations:
```bash
node --version  # Should be 18+
npm --version
rustc --version # Should be 1.70+
```

## Step 2: Install Dependencies

```bash
cd msscs_client
npm install
```

This will download all frontend dependencies (~200MB).

## Step 3: Run Development Mode

```bash
npm run tauri dev
```

First run will:
1. Download Rust dependencies (~500MB)
2. Compile MSSCS backend (~2-5 minutes)
3. Start Vite dev server
4. Launch the desktop app

**Subsequent runs are much faster (~10 seconds)!**

## Step 4: Use the App

### Upload Your First File

1. Click **"Upload File"** button
2. Select any file from your computer
3. Wait for upload to complete
4. File appears in the list with sync status

### Connect to Peers

1. Go to **"Peers"** tab
2. Click **"Add Peer"**
3. Enter peer address (e.g., `192.168.1.100:8080`)
4. Your files will sync automatically

### Monitor Sync

1. Go to **"Sync"** tab
2. View real-time statistics
3. See recent activity
4. Check network health

## Step 5: Build for Production (Optional)

```bash
npm run tauri build
```

Find installer in:
```
src-tauri/target/release/bundle/nsis/MSSCS Client_1.0.0_x64-setup.exe
```

## Common Issues

### "Port 8080 already in use"

Change port in Settings or kill the process:
```bash
netstat -ano | findstr :8080
taskkill /PID <PID> /F
```

### "Failed to start node"

1. Check `msscs_data` folder exists
2. Verify write permissions
3. Check logs in terminal

### "Compilation failed"

1. Update Rust: `rustup update`
2. Clear cache: `cargo clean`
3. Try again: `npm run tauri dev`

## Next Steps

- **Add more peers** to increase redundancy
- **Configure settings** for your use case
- **Build installer** to share with others
- **Read full docs** in README.md

## Tips

💡 **Keyboard Shortcuts**
- `F12` - Open DevTools (dev mode)
- `Ctrl+R` - Reload app (dev mode)
- `Alt+F4` - Close app

💡 **Performance**
- Keep 3-5 peers for best balance
- Use SSD for `msscs_data` folder
- Adjust chunk size for your files

💡 **Security**
- Enable API authentication in Settings
- Use strong API keys
- Keep software updated

## Support

Need help? Check:
- Full README.md
- MSSCS v4.0 documentation
- GitHub issues

Happy syncing! 🚀
