# MSSCS Web - Vue.js Version Startup Script
Write-Host ""
Write-Host "╔════════════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║  🚀 MSSCS Web - Vue.js Version                                ║" -ForegroundColor Cyan
Write-Host "╚════════════════════════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""

# Check if node_modules exists
if (-not (Test-Path "node_modules")) {
    Write-Host "📦 Installing dependencies..." -ForegroundColor Yellow
    pnpm install
    Write-Host ""
}

Write-Host "🌐 Starting Vite development server..." -ForegroundColor Green
Write-Host ""
Write-Host "📍 Features:" -ForegroundColor Cyan
Write-Host "   ✓ Same UI as desktop client" -ForegroundColor Green
Write-Host "   ✓ Shared Vue.js components" -ForegroundColor Green
Write-Host "   ✓ Hot Module Replacement (HMR)" -ForegroundColor Green
Write-Host "   ✓ P2P networking via WebRTC" -ForegroundColor Green
Write-Host "   ✓ Quantum-resistant encryption" -ForegroundColor Green
Write-Host ""
Write-Host "🔗 Server will start at: http://localhost:8000" -ForegroundColor Cyan
Write-Host ""

# Start Vite dev server
pnpm dev
