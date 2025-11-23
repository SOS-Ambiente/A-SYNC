# MSSCS Web - Vue.js Version

This is the web version of MSSCS that now uses the same Vue.js components as the desktop client.

## Features

- ✅ Same UI/UX as desktop client
- ✅ Shared Vue.js components
- ✅ P2P networking via WebRTC
- ✅ Quantum-resistant encryption
- ✅ Works in any modern browser

## Development

```bash
# Install dependencies
pnpm install

# Start development server (Vue.js + Vite)
pnpm dev

# Build for production
pnpm build

# Preview production build
pnpm preview
```

## Legacy Version

The legacy vanilla JS version is still available:

```bash
# Start legacy server
pnpm dev:legacy
```

## Architecture

- **Frontend**: Vue 3 + Vite
- **Components**: Shared with `msscs_client`
- **P2P**: PeerJS (WebRTC)
- **Storage**: IndexedDB
- **Encryption**: Quantum-resistant (ML-KEM-1024 + ML-DSA-87)

## URLs

- Development: http://localhost:8000
- Legacy: http://localhost:8000 (with `dev:legacy`)
