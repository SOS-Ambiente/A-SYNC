# MSSCS Mobile - Tauri v2 Migration Script
# This script migrates the project from Tauri v1 to v2 and sets up Android support

Write-Host "🚀 MSSCS Mobile - Tauri v2 Migration" -ForegroundColor Cyan
Write-Host "=====================================" -ForegroundColor Cyan
Write-Host ""

# Check if we're in the right directory
if (-not (Test-Path "package.json")) {
    Write-Host "✗ Error: package.json not found" -ForegroundColor Red
    Write-Host "  Please run this script from the msscs_mobile directory" -ForegroundColor Yellow
    exit 1
}

Write-Host "Step 1: Cleaning old dependencies..." -ForegroundColor Yellow
if (Test-Path "node_modules") {
    Remove-Item -Path "node_modules" -Recurse -Force
    Write-Host "✓ Removed node_modules" -ForegroundColor Green
}

if (Test-Path "src-tauri/target") {
    Remove-Item -Path "src-tauri/target" -Recurse -Force
    Write-Host "✓ Removed Rust target directory" -ForegroundColor Green
}

Write-Host ""
Write-Host "Step 2: Installing updated dependencies..." -ForegroundColor Yellow
npm install

if ($LASTEXITCODE -ne 0) {
    Write-Host "✗ npm install failed" -ForegroundColor Red
    exit 1
}
Write-Host "✓ Dependencies installed" -ForegroundColor Green

Write-Host ""
Write-Host "Step 3: Updating Rust dependencies..." -ForegroundColor Yellow
Set-Location "src-tauri"
cargo update

if ($LASTEXITCODE -ne 0) {
    Write-Host "✗ cargo update failed" -ForegroundColor Red
    Set-Location ..
    exit 1
}
Write-Host "✓ Rust dependencies updated" -ForegroundColor Green
Set-Location ..

Write-Host ""
Write-Host "Step 4: Checking Android environment..." -ForegroundColor Yellow

$androidOk = $true

if (-not $env:ANDROID_HOME) {
    Write-Host "✗ ANDROID_HOME not set" -ForegroundColor Red
    $androidOk = $false
} else {
    Write-Host "✓ ANDROID_HOME: $env:ANDROID_HOME" -ForegroundColor Green
}

if (-not $env:JAVA_HOME) {
    Write-Host "✗ JAVA_HOME not set" -ForegroundColor Red
    $androidOk = $false
} else {
    Write-Host "✓ JAVA_HOME: $env:JAVA_HOME" -ForegroundColor Green
}

# Check for NDK
if ($env:ANDROID_HOME) {
    $ndkPath = "$env:ANDROID_HOME\ndk"
    if (Test-Path $ndkPath) {
        $ndkVersions = Get-ChildItem -Path $ndkPath -Directory
        if ($ndkVersions.Count -gt 0) {
            Write-Host "✓ NDK found: $($ndkVersions[0].Name)" -ForegroundColor Green
        } else {
            Write-Host "✗ NDK not found in $ndkPath" -ForegroundColor Red
            $androidOk = $false
        }
    } else {
        Write-Host "✗ NDK directory not found" -ForegroundColor Red
        $androidOk = $false
    }
}

if (-not $androidOk) {
    Write-Host ""
    Write-Host "⚠️ Android environment not properly configured" -ForegroundColor Yellow
    Write-Host "  Run build-apk.ps1 to set up the complete Android environment" -ForegroundColor White
    Write-Host ""
    $continue = Read-Host "Continue anyway? (y/n) [n]"
    if ($continue -ne "y") {
        exit 1
    }
}

Write-Host ""
Write-Host "Step 5: Initializing Android project..." -ForegroundColor Yellow

if (Test-Path "src-tauri/gen/android") {
    Write-Host "  Android project already exists" -ForegroundColor Gray
    $reinit = Read-Host "  Reinitialize? (y/n) [n]"
    if ($reinit -eq "y") {
        Remove-Item -Path "src-tauri/gen/android" -Recurse -Force
        Write-Host "  Removed existing Android project" -ForegroundColor Gray
    } else {
        Write-Host "✓ Using existing Android project" -ForegroundColor Green
        Write-Host ""
        Write-Host "✅ Migration complete!" -ForegroundColor Green
        Write-Host ""
        Write-Host "Next steps:" -ForegroundColor Yellow
        Write-Host "  1. Review TAURI_V2_MIGRATION.md for API changes" -ForegroundColor White
        Write-Host "  2. Build APK: npm run tauri:android:build:apk" -ForegroundColor White
        Write-Host "  3. Test on device: npm run tauri:android:dev" -ForegroundColor White
        exit 0
    }
}

npm run tauri:android:init

if ($LASTEXITCODE -ne 0) {
    Write-Host "✗ Android initialization failed" -ForegroundColor Red
    Write-Host ""
    Write-Host "Troubleshooting:" -ForegroundColor Yellow
    Write-Host "  1. Verify ANDROID_HOME: $env:ANDROID_HOME" -ForegroundColor White
    Write-Host "  2. Verify JAVA_HOME: $env:JAVA_HOME" -ForegroundColor White
    Write-Host "  3. Install NDK: sdkmanager 'ndk;25.0.8775105'" -ForegroundColor White
    Write-Host "  4. Run build-apk.ps1 for full setup" -ForegroundColor White
    exit 1
}

Write-Host "✓ Android project initialized" -ForegroundColor Green

Write-Host ""
Write-Host "✅ Migration complete!" -ForegroundColor Green
Write-Host ""
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
Write-Host ""
Write-Host "What's new in Tauri v2:" -ForegroundColor Yellow
Write-Host "  ✓ Native Android support (no Capacitor needed)" -ForegroundColor Green
Write-Host "  ✓ Better performance and smaller APK size" -ForegroundColor Green
Write-Host "  ✓ Unified Rust codebase for desktop and mobile" -ForegroundColor Green
Write-Host "  ✓ Direct access to Android APIs" -ForegroundColor Green
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Yellow
Write-Host "  1. Review TAURI_V2_MIGRATION.md for API changes" -ForegroundColor White
Write-Host "  2. Build APK: npm run tauri:android:build:apk" -ForegroundColor White
Write-Host "  3. Test on device: npm run tauri:android:dev" -ForegroundColor White
Write-Host ""
Write-Host "Build commands:" -ForegroundColor Yellow
Write-Host "  Debug APK:   npm run tauri:android:build:apk" -ForegroundColor White
Write-Host "  Release APK: npm run tauri:android:build:apk -- --release" -ForegroundColor White
Write-Host "  Dev mode:    npm run tauri:android:dev" -ForegroundColor White
Write-Host ""
Write-Host "For full automated setup, run: .\build-apk.ps1" -ForegroundColor Cyan
Write-Host ""
