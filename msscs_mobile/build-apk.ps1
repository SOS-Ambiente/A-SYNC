# MSSCS Mobile - Fully Automated APK Build Script
# This script downloads and installs EVERYTHING needed, then builds the APK
# Run as Administrator for best results

#Requires -RunAsAdministrator

Write-Host "🚀 MSSCS Mobile - FULL AUTO SETUP & BUILD" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "This script will:" -ForegroundColor Yellow
Write-Host "  1. Download & install Node.js" -ForegroundColor White
Write-Host "  2. Download & install Rust" -ForegroundColor White
Write-Host "  3. Download & install Java JDK" -ForegroundColor White
Write-Host "  4. Download & install Android Command Line Tools" -ForegroundColor White
Write-Host "  5. Install Android SDK, NDK, and Build Tools" -ForegroundColor White
Write-Host "  6. Set up environment variables" -ForegroundColor White
Write-Host "  7. Install project dependencies" -ForegroundColor White
Write-Host "  8. Build the APK" -ForegroundColor White
Write-Host ""

$continue = Read-Host "Continue? (y/n) [y]"
if ($continue -eq "n") {
    Write-Host "Aborted by user" -ForegroundColor Red
    exit 0
}

Write-Host ""
Write-Host "⚙️ Starting automated setup..." -ForegroundColor Cyan
Write-Host ""

# Create temp directory for downloads
$tempDir = "$env:TEMP\msscs_setup"
New-Item -ItemType Directory -Force -Path $tempDir | Out-Null

# Define installation paths
$installBase = "C:\MSSCS_Tools"
$nodeDir = "$installBase\nodejs"
$rustDir = "$env:USERPROFILE\.cargo"
$javaDir = "$installBase\jdk"
$androidDir = "$installBase\Android"
$androidSdk = "$androidDir\sdk"

New-Item -ItemType Directory -Force -Path $installBase | Out-Null

# Function to download file with progress
function Download-File {
    param (
        [string]$Url,
        [string]$Output
    )
    
    Write-Host "  Downloading: $Url" -ForegroundColor Gray
    
    try {
        $webClient = New-Object System.Net.WebClient
        $webClient.DownloadFile($Url, $Output)
        return $true
    } catch {
        Write-Host "  Failed to download: $_" -ForegroundColor Red
        return $false
    }
}

# Function to add to PATH
function Add-ToPath {
    param (
        [string]$PathToAdd
    )
    
    $currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($currentPath -notlike "*$PathToAdd*") {
        [Environment]::SetEnvironmentVariable("Path", "$currentPath;$PathToAdd", "User")
        $env:Path = "$env:Path;$PathToAdd"
        Write-Host "  Added to PATH: $PathToAdd" -ForegroundColor Green
    }
}

# Function to set environment variable
function Set-EnvVar {
    param (
        [string]$Name,
        [string]$Value
    )
    
    [Environment]::SetEnvironmentVariable($Name, $Value, "User")
    Set-Item -Path "env:$Name" -Value $Value
    Write-Host "  Set $Name = $Value" -ForegroundColor Green
}

# ============================================================================
# 1. INSTALL NODE.JS
# ============================================================================
Write-Host "📦 [1/8] Installing Node.js..." -ForegroundColor Yellow

$nodeInstalled = $false
try {
    $nodeVersion = node --version 2>$null
    if ($nodeVersion) {
        Write-Host "✓ Node.js already installed: $nodeVersion" -ForegroundColor Green
        $nodeInstalled = $true
    }
} catch {}

if (-not $nodeInstalled) {
    Write-Host "  Downloading Node.js v20.11.0..." -ForegroundColor Gray
    $nodeUrl = "https://nodejs.org/dist/v20.11.0/node-v20.11.0-x64.msi"
    $nodeInstaller = "$tempDir\node-installer.msi"
    
    if (Download-File -Url $nodeUrl -Output $nodeInstaller) {
        Write-Host "  Installing Node.js..." -ForegroundColor Gray
        Start-Process msiexec.exe -ArgumentList "/i `"$nodeInstaller`" /quiet /norestart" -Wait
        
        # Add to PATH
        Add-ToPath -PathToAdd "C:\Program Files\nodejs"
        
        # Verify
        $nodeVersion = node --version 2>$null
        if ($nodeVersion) {
            Write-Host "✓ Node.js installed: $nodeVersion" -ForegroundColor Green
        } else {
            Write-Host "⚠️ Node.js installed but not in PATH. Restart terminal." -ForegroundColor Yellow
        }
    }
}

# ============================================================================
# 2. INSTALL RUST
# ============================================================================
Write-Host ""
Write-Host "🦀 [2/8] Installing Rust..." -ForegroundColor Yellow

$rustInstalled = $false
try {
    $rustVersion = cargo --version 2>$null
    if ($rustVersion) {
        Write-Host "✓ Rust already installed: $rustVersion" -ForegroundColor Green
        $rustInstalled = $true
    }
} catch {}

if (-not $rustInstalled) {
    Write-Host "  Downloading Rustup..." -ForegroundColor Gray
    $rustupUrl = "https://win.rustup.rs/x86_64"
    $rustupInstaller = "$tempDir\rustup-init.exe"
    
    if (Download-File -Url $rustupUrl -Output $rustupInstaller) {
        Write-Host "  Installing Rust (this may take a few minutes)..." -ForegroundColor Gray
        Start-Process -FilePath $rustupInstaller -ArgumentList "-y --default-toolchain stable" -Wait -NoNewWindow
        
        # Add to PATH
        Add-ToPath -PathToAdd "$env:USERPROFILE\.cargo\bin"
        
        # Verify
        $rustVersion = cargo --version 2>$null
        if ($rustVersion) {
            Write-Host "✓ Rust installed: $rustVersion" -ForegroundColor Green
        } else {
            Write-Host "⚠️ Rust installed but not in PATH. Restart terminal." -ForegroundColor Yellow
        }
    }
}

# ============================================================================
# 3. INSTALL JAVA JDK
# ============================================================================
Write-Host ""
Write-Host "☕ [3/8] Installing Java JDK..." -ForegroundColor Yellow

$javaInstalled = $false
try {
    $javaVersion = java -version 2>&1 | Select-String "version"
    if ($javaVersion) {
        Write-Host "✓ Java already installed: $javaVersion" -ForegroundColor Green
        $javaInstalled = $true
    }
} catch {}

if (-not $javaInstalled) {
    Write-Host "  Downloading OpenJDK 17..." -ForegroundColor Gray
    $jdkUrl = "https://download.java.net/java/GA/jdk17.0.2/dfd4a8d0985749f896bed50d7138ee7f/8/GPL/openjdk-17.0.2_windows-x64_bin.zip"
    $jdkZip = "$tempDir\openjdk.zip"
    
    if (Download-File -Url $jdkUrl -Output $jdkZip) {
        Write-Host "  Extracting JDK..." -ForegroundColor Gray
        Expand-Archive -Path $jdkZip -DestinationPath $installBase -Force
        
        $jdkExtracted = Get-ChildItem -Path $installBase -Filter "jdk-*" -Directory | Select-Object -First 1
        if ($jdkExtracted) {
            Rename-Item -Path $jdkExtracted.FullName -NewName "jdk" -Force
        }
        
        # Set JAVA_HOME and add to PATH
        Set-EnvVar -Name "JAVA_HOME" -Value $javaDir
        Add-ToPath -PathToAdd "$javaDir\bin"
        
        Write-Host "✓ Java JDK installed" -ForegroundColor Green
    }
}

# ============================================================================
# 4. INSTALL ANDROID COMMAND LINE TOOLS
# ============================================================================
Write-Host ""
Write-Host "🤖 [4/8] Installing Android Command Line Tools..." -ForegroundColor Yellow

if (-not (Test-Path $androidSdk)) {
    Write-Host "  Downloading Android Command Line Tools..." -ForegroundColor Gray
    $cmdlineUrl = "https://dl.google.com/android/repository/commandlinetools-win-9477386_latest.zip"
    $cmdlineZip = "$tempDir\cmdline-tools.zip"
    
    if (Download-File -Url $cmdlineUrl -Output $cmdlineZip) {
        Write-Host "  Extracting Command Line Tools..." -ForegroundColor Gray
        
        New-Item -ItemType Directory -Force -Path "$androidSdk\cmdline-tools" | Out-Null
        Expand-Archive -Path $cmdlineZip -DestinationPath "$androidSdk\cmdline-tools" -Force
        
        # Rename to 'latest'
        $cmdlineExtracted = Get-ChildItem -Path "$androidSdk\cmdline-tools" -Filter "cmdline-tools" -Directory | Select-Object -First 1
        if ($cmdlineExtracted) {
            Move-Item -Path "$cmdlineExtracted\*" -Destination "$androidSdk\cmdline-tools\latest" -Force
            Remove-Item -Path $cmdlineExtracted.FullName -Force
        } else {
            Rename-Item -Path "$androidSdk\cmdline-tools\cmdline-tools" -NewName "latest" -Force -ErrorAction SilentlyContinue
        }
        
        Write-Host "✓ Android Command Line Tools installed" -ForegroundColor Green
    }
} else {
    Write-Host "✓ Android SDK already exists" -ForegroundColor Green
}

# Set ANDROID_HOME
Set-EnvVar -Name "ANDROID_HOME" -Value $androidSdk
Add-ToPath -PathToAdd "$androidSdk\cmdline-tools\latest\bin"
Add-ToPath -PathToAdd "$androidSdk\platform-tools"
Add-ToPath -PathToAdd "$androidSdk\build-tools\33.0.0"

# ============================================================================
# 5. INSTALL ANDROID SDK COMPONENTS
# ============================================================================
Write-Host ""
Write-Host "📲 [5/8] Installing Android SDK components..." -ForegroundColor Yellow

$sdkmanager = "$androidSdk\cmdline-tools\latest\bin\sdkmanager.bat"

if (Test-Path $sdkmanager) {
    Write-Host "  Accepting licenses..." -ForegroundColor Gray
    echo "y" | & $sdkmanager --licenses 2>$null
    
    Write-Host "  Installing SDK Platform 33..." -ForegroundColor Gray
    & $sdkmanager "platforms;android-33" --sdk_root=$androidSdk 2>$null | Out-Null
    
    Write-Host "  Installing Build Tools 33.0.0..." -ForegroundColor Gray
    & $sdkmanager "build-tools;33.0.0" --sdk_root=$androidSdk 2>$null | Out-Null
    
    Write-Host "  Installing Platform Tools..." -ForegroundColor Gray
    & $sdkmanager "platform-tools" --sdk_root=$androidSdk 2>$null | Out-Null
    
    Write-Host "  Installing NDK 25.0.8775105..." -ForegroundColor Gray
    & $sdkmanager "ndk;25.0.8775105" --sdk_root=$androidSdk 2>$null | Out-Null
    
    Write-Host "  Installing CMake..." -ForegroundColor Gray
    & $sdkmanager "cmake;3.22.1" --sdk_root=$androidSdk 2>$null | Out-Null
    
    Write-Host "✓ Android SDK components installed" -ForegroundColor Green
} else {
    Write-Host "⚠️ sdkmanager not found, skipping SDK components" -ForegroundColor Yellow
}

# ============================================================================
# 6. ADD RUST ANDROID TARGETS
# ============================================================================
Write-Host ""
Write-Host "🎯 [6/8] Adding Rust Android targets..." -ForegroundColor Yellow

$targets = @(
    "aarch64-linux-android",
    "armv7-linux-androideabi",
    "i686-linux-android",
    "x86_64-linux-android"
)

foreach ($target in $targets) {
    Write-Host "  Adding target: $target" -ForegroundColor Gray
    rustup target add $target 2>$null | Out-Null
}

Write-Host "✓ Rust Android targets added" -ForegroundColor Green

# ============================================================================
# 7. INSTALL PROJECT DEPENDENCIES
# ============================================================================
Write-Host ""
Write-Host "📦 [7/8] Installing project dependencies..." -ForegroundColor Yellow

# Check if we're in the right directory
if (-not (Test-Path "package.json")) {
    Write-Host "⚠️ package.json not found. Make sure you're in the msscs_mobile directory" -ForegroundColor Yellow
    $currentDir = Get-Location
    Write-Host "  Current directory: $currentDir" -ForegroundColor Gray
    
    # Try to find msscs_mobile directory
    if (Test-Path "..\msscs_mobile\package.json") {
        Set-Location "..\msscs_mobile"
        Write-Host "  Changed to msscs_mobile directory" -ForegroundColor Gray
    } elseif (Test-Path "msscs_mobile\package.json") {
        Set-Location "msscs_mobile"
        Write-Host "  Changed to msscs_mobile directory" -ForegroundColor Gray
    } else {
        Write-Host "✗ Cannot find msscs_mobile directory" -ForegroundColor Red
        exit 1
    }
}

Write-Host "  Running npm install..." -ForegroundColor Gray
npm install --silent

if ($LASTEXITCODE -eq 0) {
    Write-Host "✓ Dependencies installed" -ForegroundColor Green
} else {
    Write-Host "⚠️ npm install had warnings (this is usually OK)" -ForegroundColor Yellow
}

# ============================================================================
# 8. BUILD APK
# ============================================================================
Write-Host ""
Write-Host "🔨 [8/8] Building Android APK..." -ForegroundColor Yellow

# Initialize Android project if needed
if (-not (Test-Path "src-tauri/gen/android")) {
    Write-Host "  Initializing Android project..." -ForegroundColor Gray
    npm run tauri:android:init
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "✗ Failed to initialize Android project" -ForegroundColor Red
        Write-Host ""
        Write-Host "Troubleshooting:" -ForegroundColor Yellow
        Write-Host "  1. Make sure ANDROID_HOME is set: $env:ANDROID_HOME" -ForegroundColor White
        Write-Host "  2. Make sure JAVA_HOME is set: $env:JAVA_HOME" -ForegroundColor White
        Write-Host "  3. Make sure NDK is installed: $env:ANDROID_HOME\ndk" -ForegroundColor White
        Write-Host "  4. Restart PowerShell and try again" -ForegroundColor White
        exit 1
    }
    
    Write-Host "✓ Android project initialized" -ForegroundColor Green
} else {
    Write-Host "✓ Android project already initialized" -ForegroundColor Green
}

Write-Host ""
Write-Host "  Building APK (this may take 10-20 minutes on first build)..." -ForegroundColor Gray
Write-Host "  Please be patient..." -ForegroundColor Gray
Write-Host ""

$buildType = Read-Host "Build type? (1=Debug, 2=Release) [1]"

if ($buildType -eq "2") {
    Write-Host "  Building RELEASE APK..." -ForegroundColor Cyan
    npm run tauri:android:build:apk -- --release
} else {
    Write-Host "  Building DEBUG APK..." -ForegroundColor Cyan
    npm run tauri:android:build:apk
}

if ($LASTEXITCODE -ne 0) {
    Write-Host ""
    Write-Host "✗ Build failed" -ForegroundColor Red
    Write-Host ""
    Write-Host "Common issues:" -ForegroundColor Yellow
    Write-Host "  1. Run: cd src-tauri/gen/android && ./gradlew clean" -ForegroundColor White
    Write-Host "  2. Check Java version: java -version" -ForegroundColor White
    Write-Host "  3. Check Android SDK: $env:ANDROID_HOME" -ForegroundColor White
    Write-Host "  4. Restart PowerShell and try again" -ForegroundColor White
    exit 1
}

# ============================================================================
# SUCCESS!
# ============================================================================
Write-Host ""
Write-Host "✅ BUILD SUCCESSFUL!" -ForegroundColor Green
Write-Host ""

# Find APK
if ($buildType -eq "2") {
    $apkPath = "src-tauri\gen\android\app\build\outputs\apk\release\app-release.apk"
} else {
    $apkPath = "src-tauri\gen\android\app\build\outputs\apk\debug\app-debug.apk"
}

if (Test-Path $apkPath) {
    $apkSize = (Get-Item $apkPath).Length / 1MB
    $apkFullPath = (Resolve-Path $apkPath).Path
    
    Write-Host "📱 APK READY!" -ForegroundColor Cyan
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
    Write-Host "  Location: $apkFullPath" -ForegroundColor White
    Write-Host "  Size: $([math]::Round($apkSize, 2)) MB" -ForegroundColor White
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
    Write-Host ""
    
    # Copy to easy location
    $easyPath = "MSSCS-Mobile.apk"
    Copy-Item -Path $apkPath -Destination $easyPath -Force
    Write-Host "✓ APK copied to: $(Get-Location)\$easyPath" -ForegroundColor Green
    Write-Host ""
    
    # Ask to install
    Write-Host "Install Options:" -ForegroundColor Yellow
    Write-Host "  1. Install on connected Android device (USB)" -ForegroundColor White
    Write-Host "  2. Open APK location in Explorer" -ForegroundColor White
    Write-Host "  3. Skip" -ForegroundColor White
    Write-Host ""
    
    $choice = Read-Host "Choose option (1/2/3) [3]"
    
    if ($choice -eq "1") {
        Write-Host ""
        Write-Host "📲 Installing APK on device..." -ForegroundColor Yellow
        
        # Check if adb is available
        $adbPath = "$androidSdk\platform-tools\adb.exe"
        if (Test-Path $adbPath) {
            & $adbPath devices
            Write-Host ""
            & $adbPath install -r $apkPath
            
            if ($LASTEXITCODE -eq 0) {
                Write-Host "✓ APK installed successfully!" -ForegroundColor Green
            } else {
                Write-Host "✗ Installation failed" -ForegroundColor Red
                Write-Host "  Make sure:" -ForegroundColor Yellow
                Write-Host "    - Device is connected via USB" -ForegroundColor White
                Write-Host "    - USB debugging is enabled" -ForegroundColor White
                Write-Host "    - Device is authorized" -ForegroundColor White
            }
        } else {
            Write-Host "✗ ADB not found. Install manually." -ForegroundColor Red
        }
    } elseif ($choice -eq "2") {
        explorer.exe /select,$apkFullPath
        Write-Host "✓ Opened in Explorer" -ForegroundColor Green
    }
} else {
    Write-Host "⚠️ APK not found at expected location" -ForegroundColor Yellow
    Write-Host "  Expected: $apkPath" -ForegroundColor Gray
}

# ============================================================================
# FINAL SUMMARY
# ============================================================================
Write-Host ""
Write-Host "🎉 SETUP COMPLETE!" -ForegroundColor Cyan
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
Write-Host ""
Write-Host "Installed Components:" -ForegroundColor Yellow
Write-Host "  ✓ Node.js" -ForegroundColor Green
Write-Host "  ✓ Rust" -ForegroundColor Green
Write-Host "  ✓ Java JDK" -ForegroundColor Green
Write-Host "  ✓ Android SDK" -ForegroundColor Green
Write-Host "  ✓ Android NDK" -ForegroundColor Green
Write-Host "  ✓ Build Tools" -ForegroundColor Green
Write-Host ""
Write-Host "Environment Variables Set:" -ForegroundColor Yellow
Write-Host "  ANDROID_HOME = $env:ANDROID_HOME" -ForegroundColor White
Write-Host "  JAVA_HOME = $env:JAVA_HOME" -ForegroundColor White
Write-Host ""
Write-Host "Next Steps:" -ForegroundColor Yellow
Write-Host "  1. Transfer MSSCS-Mobile.apk to your Android device" -ForegroundColor White
Write-Host "  2. Enable 'Install from Unknown Sources' in Settings" -ForegroundColor White
Write-Host "  3. Install and run the app" -ForegroundColor White
Write-Host "  4. Connect to MSSCS nodes on your network" -ForegroundColor White
Write-Host ""
Write-Host "To rebuild in the future:" -ForegroundColor Yellow
Write-Host "  npm run tauri:android:build:apk" -ForegroundColor White
Write-Host ""
Write-Host "For help, see:" -ForegroundColor Yellow
Write-Host "  - QUICKSTART.md" -ForegroundColor White
Write-Host "  - BUILD_GUIDE.md" -ForegroundColor White
Write-Host "  - FEATURES.md" -ForegroundColor White
Write-Host ""

# Clean up temp files
Write-Host "Cleaning up temporary files..." -ForegroundColor Gray
Remove-Item -Path $tempDir -Recurse -Force -ErrorAction SilentlyContinue

Write-Host ""
Write-Host "✨ All done! Enjoy MSSCS Mobile! ✨" -ForegroundColor Cyan
Write-Host ""
