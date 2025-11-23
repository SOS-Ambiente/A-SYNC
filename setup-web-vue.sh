#!/bin/bash

# MSSCS Web - Vue.js Setup Script
echo ""
echo "╔════════════════════════════════════════════════════════════════╗"
echo "║  🚀 MSSCS Web - Vue.js Setup                                  ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""

# Check if pnpm is installed
echo "📦 Checking for pnpm..."
if ! command -v pnpm &> /dev/null; then
    echo "❌ pnpm not found!"
    echo ""
    echo "Please install pnpm first:"
    echo "  npm install -g pnpm"
    echo ""
    exit 1
fi

pnpmVersion=$(pnpm --version)
echo "✅ pnpm version: $pnpmVersion"
echo ""

# Navigate to msscs_web
echo "📂 Navigating to msscs_web..."
cd msscs_web

# Install dependencies
echo "📦 Installing dependencies..."
echo ""
pnpm install

if [ $? -ne 0 ]; then
    echo ""
    echo "❌ Installation failed!"
    exit 1
fi

echo ""
echo "╔════════════════════════════════════════════════════════════════╗"
echo "║  ✅ Setup Complete!                                           ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""
echo "🚀 To start the development server:"
echo ""
echo "   pnpm dev"
echo ""
echo "   Or use the startup script:"
echo "   bash start-vue-web.sh"
echo ""
echo "🌐 Server will be available at:"
echo "   http://localhost:8000"
echo ""
echo "📚 Documentation:"
echo "   - Quick Start: ../QUICK_START_WEB_VUE.md"
echo "   - Migration Guide: ../WEB_VUE_MIGRATION_GUIDE.md"
echo "   - Status Fix: ../WEB_STATUS_FIX_SUMMARY.md"
echo ""

# Make startup script executable
chmod +x start-vue-web.sh
echo "✅ Made start-vue-web.sh executable"
echo ""
