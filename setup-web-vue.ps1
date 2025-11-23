# MSSCS Web - Vue.js Setup Script
Write-Host ""
Write-Host "╔════════════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║  🚀 MSSCS Web - Vue.js Setup                                  ║" -ForegroundColor Cyan
Write-Host "╚════════════════════════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""

# Check if pnpm is installed
Write-Host "📦 Checking for pnpm..." -ForegroundColor Yellow
$pnpmVersion = pnpm --version 2>$null
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ pnpm not found!" -ForegroundColor Red
    Write-Host ""
    Write-Host "Please install pnpm first:" -ForegroundColor Yellow
    Write-Host "  npm install -g pnpm" -ForegroundColor Cyan
    Write-Host ""
    exit 1
}
Write-Host "✅ pnpm version: $pnpmVersion" -ForegroundColor Green
Write-Host ""

# Navigate to msscs_web
Write-Host "📂 Navigating to msscs_web..." -ForegroundColor Yellow
Set-Location msscs_web

# Install dependencies
Write-Host "📦 Installing dependencies..." -ForegroundColor Yellow
Write-Host ""
pnpm install

if ($LASTEXITCODE -ne 0) {
    Write-Host ""
    Write-Host "❌ Installation failed!" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "╔════════════════════════════════════════════════════════════════╗" -ForegroundColor Green
Write-Host "║  ✅ Setup Complete!                                           ║" -ForegroundColor Green
Write-Host "╚════════════════════════════════════════════════════════════════╝" -ForegroundColor Green
Write-Host ""
Write-Host "🚀 To start the development server:" -ForegroundColor Cyan
Write-Host ""
Write-Host "   pnpm dev" -ForegroundColor Yellow
Write-Host ""
Write-Host "   Or use the startup script:" -ForegroundColor Cyan
Write-Host "   pwsh start-vue-web.ps1" -ForegroundColor Yellow
Write-Host ""
Write-Host "🌐 Server will be available at:" -ForegroundColor Cyan
Write-Host "   http://localhost:8000" -ForegroundColor Yellow
Write-Host ""
Write-Host "📚 Documentation:" -ForegroundColor Cyan
Write-Host "   - Quick Start: ../QUICK_START_WEB_VUE.md" -ForegroundColor Yellow
Write-Host "   - Migration Guide: ../WEB_VUE_MIGRATION_GUIDE.md" -ForegroundColor Yellow
Write-Host "   - Status Fix: ../WEB_STATUS_FIX_SUMMARY.md" -ForegroundColor Yellow
Write-Host ""
