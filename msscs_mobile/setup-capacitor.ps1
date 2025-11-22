# MSSCS Mobile - Capacitor Setup Script
# This creates a WORKING Android APK using Capacitor (production-ready)

Write-Host ""
Write-Host "╔════════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║                                                            ║" -ForegroundColor Cyan
Write-Host "║         MSSCS MOBILE - CAPACITOR SETUP                     ║" -ForegroundColor Cyan
Write-Host "║                                                            ║" -ForegroundColor Cyan
Write-Host "║  Building Android APK with Capacitor                       ║" -ForegroundColor Cyan
Write-Host "║  (Stable, production-ready alternative to Tauri)           ║" -ForegroundColor Cyan
Write-Host "║                                                            ║" -ForegroundColor Cyan
Write-Host "╚════════════════════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""

# Check if we're in the right directory
if (-not (Test-Path "package.json")) {
    Write-Host "✗ Please run this from the msscs_mobile directory" -ForegroundColor Red
    exit 1
}

Write-Host "📦 Step 1: Installing Capacitor..." -ForegroundColor Yellow
npm install @capacitor/core @capacitor/cli @capacitor/android

Write-Host ""
Write-Host "⚙️ Step 2: Initializing Capacitor..." -ForegroundColor Yellow
npx cap init "MSSCS Mobile" "com.msscs.mobile" --web-dir=dist

Write-Host ""
Write-Host "🏗️ Step 3: Building web app..." -ForegroundColor Yellow
npm run build

Write-Host ""
Write-Host "🤖 Step 4: Adding Android platform..." -ForegroundColor Yellow
npx cap add android

Write-Host ""
Write-Host "🔄 Step 5: Syncing to Android..." -ForegroundColor Yellow
npx cap sync

Write-Host ""
Write-Host "✅ Setup complete!" -ForegroundColor Green
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Cyan
Write-Host "  1. Open Android Studio: npx cap open android" -ForegroundColor White
Write-Host "  2. In Android Studio: Build → Build Bundle(s) / APK(s) → Build APK(s)" -ForegroundColor White
Write-Host "  3. APK will be in: android/app/build/outputs/apk/debug/" -ForegroundColor White
Write-Host ""
Write-Host "Or build from command line:" -ForegroundColor Cyan
Write-Host "  cd android" -ForegroundColor White
Write-Host "  ./gradlew assembleDebug" -ForegroundColor White
Write-Host ""
