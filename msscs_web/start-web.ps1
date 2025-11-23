#!/usr/bin/env pwsh
# MSSCS Web Server Startup Script

# Get the directory where this script is located
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path

Write-Host ""
Write-Host "╔════════════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║  🌐 MSSCS Web Server - Starting...                           ║" -ForegroundColor Cyan
Write-Host "╚════════════════════════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""

Write-Host "📋 Important Information:" -ForegroundColor Yellow
Write-Host "   • server.js is a STATIC FILE SERVER only" -ForegroundColor White
Write-Host "   • P2P network initializes in the BROWSER (app.js)" -ForegroundColor White
Write-Host "   • Node Status shows 'offline' until browser loads" -ForegroundColor White
Write-Host ""

Write-Host "🔧 Starting static file server..." -ForegroundColor Green
Write-Host "   Working directory: $ScriptDir" -ForegroundColor Gray
Write-Host ""

# Check if node_modules exists
if (-not (Test-Path "$ScriptDir\node_modules")) {
    Write-Host "⚠️  node_modules not found. Installing dependencies..." -ForegroundColor Yellow
    Push-Location $ScriptDir
    npm install
    Pop-Location
    Write-Host ""
}

# Start the server
Write-Host "🚀 Starting server on http://localhost:8000" -ForegroundColor Green
Write-Host ""
Write-Host "📍 Next Steps:" -ForegroundColor Cyan
Write-Host "   1. Open http://localhost:8000 in your browser" -ForegroundColor White
Write-Host "   2. Wait for P2P initialization (5-10 seconds)" -ForegroundColor White
Write-Host "   3. Check browser console for connection status" -ForegroundColor White
Write-Host "   4. Share your Peer ID to connect with others" -ForegroundColor White
Write-Host ""

# Change to script directory and run server
Push-Location $ScriptDir
try {
    node server.js
} finally {
    Pop-Location
}
