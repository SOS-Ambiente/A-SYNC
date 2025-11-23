#!/bin/bash
# MSSCS Web Server Startup Script

# Get the directory where this script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

echo ""
echo "╔════════════════════════════════════════════════════════════════╗"
echo "║  🌐 MSSCS Web Server - Starting...                           ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""

echo "📋 Important Information:"
echo "   • server.js is a STATIC FILE SERVER only"
echo "   • P2P network initializes in the BROWSER (app.js)"
echo "   • Node Status shows 'offline' until browser loads"
echo ""

echo "🔧 Starting static file server..."
echo "   Working directory: $SCRIPT_DIR"
echo ""

# Change to script directory
cd "$SCRIPT_DIR"

# Check if node_modules exists
if [ ! -d "node_modules" ]; then
    echo "⚠️  node_modules not found. Installing dependencies..."
    npm install
    echo ""
fi

# Start the server
echo "🚀 Starting server on http://localhost:8000"
echo ""
echo "📍 Next Steps:"
echo "   1. Open http://localhost:8000 in your browser"
echo "   2. Wait for P2P initialization (5-10 seconds)"
echo "   3. Check browser console for connection status"
echo "   4. Share your Peer ID to connect with others"
echo ""

node server.js
