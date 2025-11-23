#!/bin/bash

# MSSCS Web - Vue.js Version Startup Script
echo ""
echo "╔════════════════════════════════════════════════════════════════╗"
echo "║  🚀 MSSCS Web - Vue.js Version                                ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""

# Check if node_modules exists
if [ ! -d "node_modules" ]; then
    echo "📦 Installing dependencies..."
    pnpm install
    echo ""
fi

echo "🌐 Starting Vite development server..."
echo ""
echo "📍 Features:"
echo "   ✓ Same UI as desktop client"
echo "   ✓ Shared Vue.js components"
echo "   ✓ Hot Module Replacement (HMR)"
echo "   ✓ P2P networking via WebRTC"
echo "   ✓ Quantum-resistant encryption"
echo ""
echo "🔗 Server will start at: http://localhost:8000"
echo ""

# Start Vite dev server
pnpm dev
