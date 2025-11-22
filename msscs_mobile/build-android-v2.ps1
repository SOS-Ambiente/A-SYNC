# MSSCS Mobile - Tauri v2 Android Build Script
Write-Host "=== MSSCS Mobile - Tauri v2 Android Build ===" -ForegroundColor Cyan

# Step 1: Clean previous builds
Write-Host "`n[1/5] Cleaning previous builds..." -ForegroundColor Yellow
if (Test-Path "dist") {
    Remove-Item -Recurse -Force "dist"
}
if (Test-Path "src-tauri/target") {
    Remove-Item -Recurse -Force "src-tauri/target"
}

# Step 2: Install dependencies
Write-Host "`n[2/5] Installing Node dependencies..." -ForegroundColor Yellow
npm install

# Step 3: Build frontend
Write-Host "`n[3/5] Building frontend..." -ForegroundColor Yellow
npm run build

# Step 4: Initialize Android (if needed)
Write-Host "`n[4/5] Checking Android initialization..." -ForegroundColor Yellow
if (-not (Test-Path "src-tauri/gen/android")) {
    Write-Host "Initializing Android project..." -ForegroundColor Yellow
    npm run tauri android init
}

# Step 5: Build APK
Write-Host "`n[5/5] Building Android APK..." -ForegroundColor Yellow
npm run tauri android build --apk

Write-Host "`n=== Build Complete! ===" -ForegroundColor Green
Write-Host "APK location: src-tauri/gen/android/app/build/outputs/apk/universal/release/" -ForegroundColor Cyan
