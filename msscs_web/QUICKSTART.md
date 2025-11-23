# Quick Start Guide

Get MSSCS Web running in 60 seconds!

## Step 1: Start the Server

### Windows
```powershell
.\start.ps1
```

### Linux/Mac
```bash
chmod +x start.sh
./start.sh
```

### Alternative (No Node.js)
```bash
# Python 3
python -m http.server 8000

# Python 2
python -m SimpleHTTPServer 8000
```

## Step 2: Open in Browser

Open your browser and go to:
```
http://localhost:8000
```

You should see the MSSCS Web interface!

## Step 3: Upload Your First File

1. Click **"Choose files"** or drag & drop a file
2. Watch the upload progress
3. File appears in "Your Files" section

## Step 4: Connect to a Friend

### On Your Computer:
1. Copy your **Peer ID** from the bottom of the page
2. Send it to your friend (via chat, email, etc.)

### On Friend's Computer:
1. They open MSSCS Web in their browser
2. Paste your Peer ID in the "Connect to Peer" box
3. Click **"Connect"**

### Result:
- You'll see each other in "Connected Peers"
- Your files automatically sync between you!

## Step 5: Download a File

1. Click **"Download"** next to any file
2. File is decrypted and downloaded to your computer

## That's It!

You now have a working P2P encrypted storage system!

## Common Issues

### "Cannot connect to peer"
- Make sure both browsers are open
- Check if you're on the same network
- Try refreshing both pages

### "Storage quota exceeded"
- Delete old files
- Check browser storage settings
- Try a different browser

### "Server won't start"
- Make sure port 8000 is not in use
- Try a different port: `PORT=3000 node server.js`
- Check if Node.js is installed: `node --version`

## Next Steps

- Read [README.md](README.md) for detailed features
- Check [DEPLOYMENT.md](DEPLOYMENT.md) for production setup
- See [FEATURES.md](FEATURES.md) for technical details

## Tips

### Backup Your Key
```javascript
// In browser console
const key = await app.crypto.exportKey();
console.log(JSON.stringify(key));
// Save this somewhere safe!
```

### Import Key on Another Device
```javascript
// In browser console
const keyData = { /* paste your key here */ };
await app.crypto.importKey(keyData);
```

### Clear All Data
```javascript
// In browser console
await app.storage.clear();
app.crypto.clearKey();
location.reload();
```

## Security Reminder

- Your encryption key is stored in browser localStorage
- Clearing browser data will delete your key
- Without the key, your files cannot be decrypted
- Always backup your key before clearing browser data!

## Need Help?

- Check the browser console for errors (F12)
- Read the full documentation
- Open an issue on GitHub
