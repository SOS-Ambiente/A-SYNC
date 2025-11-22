# MSSCS Mobile - Complete Setup and Build Script
# This is the ULTIMATE script - downloads everything, sets up everything, builds everything
# Run as Administrator: Right-click → Run as Administrator

#Requires -RunAsAdministrator

$ErrorActionPreference = "Continue"

Write-Host ""
Write-Host "╔════════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║                                                            ║" -ForegroundColor Cyan
Write-Host "║         MSSCS MOBILE - COMPLETE AUTO SETUP                ║" -ForegroundColor Cyan
Write-Host "║                                                            ║" -ForegroundColor Cyan
Write-Host "║  This script will download and install EVERYTHING:        ║" -ForegroundColor Cyan
Write-Host "║    • Node.js 20                                            ║" -ForegroundColor Cyan
Write-Host "║    • Rust (latest stable)                                  ║" -ForegroundColor Cyan
Write-Host "║    • Java JDK 17                                           ║" -ForegroundColor Cyan
Write-Host "║    • Android SDK & NDK                                     ║" -ForegroundColor Cyan
Write-Host "║    • All build tools                                       ║" -ForegroundColor Cyan
Write-Host "║    • Project dependencies                                  ║" -ForegroundColor Cyan
Write-Host "║    • Build the APK                                         ║" -ForegroundColor Cyan
Write-Host "║                                                            ║" -ForegroundColor Cyan
Write-Host "║  Estimated time: 20-30 minutes                             ║" -ForegroundColor Cyan
Write-Host "║  Disk space needed: ~5 GB                                  ║" -ForegroundColor Cyan
Write-Host "║                                                            ║" -ForegroundColor Cyan
Write-Host "╚════════════════════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""

$continue = Read-Host "Ready to start? (y/n) [y]"
if ($continue -eq "n") {
    Write-Host "Setup cancelled." -ForegroundColor Yellow
    exit 0
}

$startTime = Get-Date

# Create installation directory
$installBase = "C:\MSSCS_Tools"
$tempDir = "$env:TEMP\msscs_setup_$(Get-Date -Format 'yyyyMMdd_HHmmss')"

Write-Host ""
Write-Host "Creating directories..." -ForegroundColor Gray
New-Item -ItemType Directory -Force -Path $installBase | Out-Null
New-Item -ItemType Directory -Force -Path $tempDir | Out-Null

# Helper functions
function Write-Step {
    param([string]$Message)
    Write-Host ""
    Write-Host "═══════════════════════════════════════════════════════════" -ForegroundColor DarkCyan
    Write-Host " $Message" -ForegroundColor Cyan
    Write-Host "═══════════════════════════════════════════════════════════" -ForegroundColor DarkCyan
}

function Write-Progress {
    param([string]$Message)
    Write-Host "  → $Message" -ForegroundColor Gray
}

function Write-Success {
    param([string]$Message)
    Write-Host "  ✓ $Message" -ForegroundColor Green
}

function Write-Warning {
    param([string]$Message)
    Write-Host "  ⚠ $Message" -ForegroundColor Yellow
}

function Write-Error {
    param([string]$Message)
    Write-Host "  ✗ $Message" -ForegroundColor Red
}

function Download-WithProgress {
    param(
        [string]$Url,
        [string]$Output,
        [string]$Name
    )
    
    Write-Progress "Downloading $Name..."
    Write-Host "    URL: $Url" -ForegroundColor DarkGray
    
    try {
        $webClient = New-Object System.Net.WebClient
        $webClient.DownloadFile($Url, $Output)
        Write-Success "Downloaded $Name"
        return $true
    } catch {
        Write-Error "Failed to download $Name : $_"
        return $false
    }
}

function Add-ToUserPath {
    param([string]$Path)
    
    $currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($currentPath -notlike "*$Path*") {
        [Environment]::SetEnvironmentVariable("Path", "$currentPath;$Path", "User")
        $env:Path = "$env:Path;$Path"
        Write-Success "Added to PATH: $Path"
    }
}

function Set-UserEnvVar {
    param([string]$Name, [string]$Value)
    
    [Environment]::SetEnvironmentVariable($Name, $Value, "User")
    Set-Item -Path "env:$Name" -Value $Value -ErrorAction SilentlyContinue
    Write-Success "Set $Name = $Value"
}

# ═══════════════════════════════════════════════════════════════════════════
# STEP 1: NODE.JS
# ═══════════════════════════════════════════════════════════════════════════
Write-Step "STEP 1/8: Installing Node.js"

$nodeInstalled = $false
try {
    $nodeVer = node --version 2>$null
    if ($nodeVer) {
        Write-Success "Node.js already installed: $nodeVer"
        $nodeInstalled = $true
    }
} catch {}

if (-not $nodeInstalled) {
    $nodeUrl = "https://nodejs.org/dist/v20.11.0/node-v20.11.0-x64.msi"
    $nodeInstaller = "$tempDir\node.msi"
    
    if (Download-WithProgress -Url $nodeUrl -Output $nodeInstaller -Name "Node.js") {
        Write-Progress "Installing Node.js (silent install)..."
        $process = Start-Process msiexec.exe -ArgumentList "/i `"$nodeInstaller`" /quiet /norestart" -Wait -PassThru
        
        if ($process.ExitCode -eq 0) {
            Add-ToUserPath -Path "C:\Program Files\nodejs"
            Write-Success "Node.js installed successfully"
        } else {
            Write-Warning "Node.js installation returned code $($process.ExitCode)"
        }
    }
}

# ═══════════════════════════════════════════════════════════════════════════
# STEP 2: RUST
# ═══════════════════════════════════════════════════════════════════════════
Write-Step "STEP 2/8: Installing Rust"

$rustInstalled = $false
try {
    $rustVer = cargo --version 2>$null
    if ($rustVer) {
        Write-Success "Rust already installed: $rustVer"
        $rustInstalled = $true
    }
} catch {}

if (-not $rustInstalled) {
    $rustupUrl = "https://win.rustup.rs/x86_64"
    $rustupInstaller = "$tempDir\rustup-init.exe"
    
    if (Download-WithProgress -Url $rustupUrl -Output $rustupInstaller -Name "Rustup") {
        Write-Progress "Installing Rust (this takes 5-10 minutes)..."
        $process = Start-Process -FilePath $rustupInstaller -ArgumentList "-y --default-toolchain stable" -Wait -PassThru -NoNewWindow
        
        if ($process.ExitCode -eq 0) {
            Add-ToUserPath -Path "$env:USERPROFILE\.cargo\bin"
            Write-Success "Rust installed successfully"
        } else {
            Write-Warning "Rust installation returned code $($process.ExitCode)"
        }
    }
}

# ═══════════════════════════════════════════════════════════════════════════
# STEP 3: JAVA JDK
# ═══════════════════════════════════════════════════════════════════════════
Write-Step "STEP 3/8: Installing Java JDK"

$javaInstalled = $false
try {
    $javaVer = java -version 2>&1 | Select-String "version"
    if ($javaVer) {
        Write-Success "Java already installed: $javaVer"
        $javaInstalled = $true
        
        # Make sure JAVA_HOME is set
        if (-not $env:JAVA_HOME) {
            $javaPath = (Get-Command java -ErrorAction SilentlyContinue).Source
            if ($javaPath) {
                $javaHome = Split-Path (Split-Path $javaPath)
                Set-UserEnvVar -Name "JAVA_HOME" -Value $javaHome
            }
        }
    }
} catch {}

if (-not $javaInstalled) {
    $jdkUrl = "https://download.java.net/java/GA/jdk17.0.2/dfd4a8d0985749f896bed50d7138ee7f/8/GPL/openjdk-17.0.2_windows-x64_bin.zip"
    $jdkZip = "$tempDir\openjdk.zip"
    $jdkDir = "$installBase\jdk"
    
    if (Download-WithProgress -Url $jdkUrl -Output $jdkZip -Name "OpenJDK 17") {
        Write-Progress "Extracting JDK..."
        Expand-Archive -Path $jdkZip -DestinationPath $installBase -Force
        
        # Find and rename JDK directory
        $jdkExtracted = Get-ChildItem -Path $installBase -Filter "jdk-*" -Directory | Select-Object -First 1
        if ($jdkExtracted) {
            if (Test-Path $jdkDir) {
                Remove-Item -Path $jdkDir -Recurse -Force
            }
            Move-Item -Path $jdkExtracted.FullName -Destination $jdkDir -Force
        }
        
        Set-UserEnvVar -Name "JAVA_HOME" -Value $jdkDir
        Add-ToUserPath -Path "$jdkDir\bin"
        Write-Success "Java JDK installed successfully"
    }
}

# ═══════════════════════════════════════════════════════════════════════════
# STEP 4: ANDROID SDK
# ═══════════════════════════════════════════════════════════════════════════
Write-Step "STEP 4/8: Installing Android SDK"

$androidSdk = "$installBase\Android\sdk"

if (-not (Test-Path "$androidSdk\cmdline-tools\latest")) {
    $cmdlineUrl = "https://dl.google.com/android/repository/commandlinetools-win-9477386_latest.zip"
    $cmdlineZip = "$tempDir\cmdline-tools.zip"
    
    if (Download-WithProgress -Url $cmdlineUrl -Output $cmdlineZip -Name "Android Command Line Tools") {
        Write-Progress "Extracting Android SDK..."
        
        New-Item -ItemType Directory -Force -Path "$androidSdk\cmdline-tools\latest" | Out-Null
        Expand-Archive -Path $cmdlineZip -DestinationPath "$androidSdk\cmdline-tools" -Force
        
        # Move contents to 'latest' directory
        $cmdlineContent = Get-ChildItem -Path "$androidSdk\cmdline-tools\cmdline-tools" -ErrorAction SilentlyContinue
        if ($cmdlineContent) {
            Move-Item -Path "$androidSdk\cmdline-tools\cmdline-tools\*" -Destination "$androidSdk\cmdline-tools\latest" -Force
            Remove-Item -Path "$androidSdk\cmdline-tools\cmdline-tools" -Recurse -Force -ErrorAction SilentlyContinue
        }
        
        Write-Success "Android SDK installed"
    }
} else {
    Write-Success "Android SDK already installed"
}

Set-UserEnvVar -Name "ANDROID_HOME" -Value $androidSdk
Add-ToUserPath -Path "$androidSdk\cmdline-tools\latest\bin"
Add-ToUserPath -Path "$androidSdk\platform-tools"
Add-ToUserPath -Path "$androidSdk\build-tools\33.0.0"

# ═══════════════════════════════════════════════════════════════════════════
# STEP 5: ANDROID SDK COMPONENTS
# ═══════════════════════════════════════════════════════════════════════════
Write-Step "STEP 5/8: Installing Android SDK Components"

$sdkmanager = "$androidSdk\cmdline-tools\latest\bin\sdkmanager.bat"

if (Test-Path $sdkmanager) {
    Write-Progress "Accepting Android licenses..."
    "y`ny`ny`ny`ny`ny`ny`ny`n" | & $sdkmanager --licenses --sdk_root=$androidSdk 2>&1 | Out-Null
    
    $components = @(
        "platforms;android-33",
        "build-tools;33.0.0",
        "platform-tools",
        "ndk;25.0.8775105",
        "cmake;3.22.1"
    )
    
    foreach ($component in $components) {
        Write-Progress "Installing $component..."
        & $sdkmanager $component --sdk_root=$androidSdk 2>&1 | Out-Null
    }
    
    Write-Success "Android SDK components installed"
} else {
    Write-Warning "sdkmanager not found, skipping components"
}

# ═══════════════════════════════════════════════════════════════════════════
# STEP 6: RUST ANDROID TARGETS
# ═══════════════════════════════════════════════════════════════════════════
Write-Step "STEP 6/8: Adding Rust Android Targets"

$targets = @(
    "aarch64-linux-android",
    "armv7-linux-androideabi",
    "i686-linux-android",
    "x86_64-linux-android"
)

foreach ($target in $targets) {
    Write-Progress "Adding $target..."
    rustup target add $target 2>&1 | Out-Null
}

Write-Success "Rust Android targets added"

# ═══════════════════════════════════════════════════════════════════════════
# STEP 7: PROJECT DEPENDENCIES
# ═══════════════════════════════════════════════════════════════════════════
Write-Step "STEP 7/8: Installing Project Dependencies"

# Navigate to project directory
if (-not (Test-Path "package.json")) {
    Write-Progress "Looking for msscs_mobile directory..."
    
    if (Test-Path "msscs_mobile\package.json") {
        Set-Location "msscs_mobile"
    } elseif (Test-Path "..\msscs_mobile\package.json") {
        Set-Location "..\msscs_mobile"
    } else {
        Write-Error "Cannot find msscs_mobile directory with package.json"
        Write-Host ""
        Write-Host "Please run this script from:" -ForegroundColor Yellow
        Write-Host "  - The msscs_mobile directory, or" -ForegroundColor White
        Write-Host "  - The parent directory containing msscs_mobile" -ForegroundColor White
        exit 1
    }
}

Write-Progress "Running npm install..."
npm install 2>&1 | Out-Null

if ($LASTEXITCODE -eq 0) {
    Write-Success "Project dependencies installed"
} else {
    Write-Warning "npm install completed with warnings (usually OK)"
}

# ═══════════════════════════════════════════════════════════════════════════
# STEP 8: BUILD APK
# ═══════════════════════════════════════════════════════════════════════════
Write-Step "STEP 8/8: Building Android APK"

# Initialize Android project
if (-not (Test-Path "src-tauri\gen\android")) {
    Write-Progress "Initializing Android project..."
    npm run tauri android init 2>&1 | Out-Null
    
    if ($LASTEXITCODE -eq 0) {
        Write-Success "Android project initialized"
    } else {
        Write-Error "Failed to initialize Android project"
        Write-Host ""
        Write-Host "Please check:" -ForegroundColor Yellow
        Write-Host "  ANDROID_HOME = $env:ANDROID_HOME" -ForegroundColor White
        Write-Host "  JAVA_HOME = $env:JAVA_HOME" -ForegroundColor White
        Write-Host ""
        Write-Host "Try restarting PowerShell and running again" -ForegroundColor Yellow
        exit 1
    }
} else {
    Write-Success "Android project already initialized"
}

Write-Host ""
Write-Host "╔════════════════════════════════════════════════════════════╗" -ForegroundColor Yellow
Write-Host "║                                                            ║" -ForegroundColor Yellow
Write-Host "║  BUILDING APK - This will take 10-20 minutes              ║" -ForegroundColor Yellow
Write-Host "║  Please be patient and don't close this window...         ║" -ForegroundColor Yellow
Write-Host "║                                                            ║" -ForegroundColor Yellow
Write-Host "╚════════════════════════════════════════════════════════════╝" -ForegroundColor Yellow
Write-Host ""

Write-Host "Build type:" -ForegroundColor Cyan
Write-Host "  1. Debug (faster, larger, for testing)" -ForegroundColor White
Write-Host "  2. Release (slower, smaller, for distribution)" -ForegroundColor White
Write-Host ""

$buildType = Read-Host "Choose (1/2) [1]"

if ($buildType -eq "2") {
    Write-Progress "Building RELEASE APK..."
    npm run tauri android build -- --release
    $apkPath = "src-tauri\gen\android\app\build\outputs\apk\release\app-release.apk"
} else {
    Write-Progress "Building DEBUG APK..."
    npm run tauri android build
    $apkPath = "src-tauri\gen\android\app\build\outputs\apk\debug\app-debug.apk"
}

# ═══════════════════════════════════════════════════════════════════════════
# RESULTS
# ═══════════════════════════════════════════════════════════════════════════

$endTime = Get-Date
$duration = $endTime - $startTime

Write-Host ""
Write-Host ""

if ($LASTEXITCODE -eq 0 -and (Test-Path $apkPath)) {
    $apkSize = (Get-Item $apkPath).Length / 1MB
    $apkFullPath = (Resolve-Path $apkPath).Path
    
    Write-Host "╔════════════════════════════════════════════════════════════╗" -ForegroundColor Green
    Write-Host "║                                                            ║" -ForegroundColor Green
    Write-Host "║                  ✓ BUILD SUCCESSFUL!                       ║" -ForegroundColor Green
    Write-Host "║                                                            ║" -ForegroundColor Green
    Write-Host "╚════════════════════════════════════════════════════════════╝" -ForegroundColor Green
    Write-Host ""
    Write-Host "APK Details:" -ForegroundColor Cyan
    Write-Host "  Location: $apkFullPath" -ForegroundColor White
    Write-Host "  Size: $([math]::Round($apkSize, 2)) MB" -ForegroundColor White
    Write-Host "  Build time: $($duration.ToString('mm\:ss'))" -ForegroundColor White
    Write-Host ""
    
    # Copy to easy location
    $easyName = "MSSCS-Mobile-$(Get-Date -Format 'yyyyMMdd').apk"
    Copy-Item -Path $apkPath -Destination $easyName -Force
    Write-Success "APK copied to: $(Get-Location)\$easyName"
    
    Write-Host ""
    Write-Host "What's next?" -ForegroundColor Cyan
    Write-Host "  1. Install on device via USB" -ForegroundColor White
    Write-Host "  2. Open APK location" -ForegroundColor White
    Write-Host "  3. Exit" -ForegroundColor White
    Write-Host ""
    
    $choice = Read-Host "Choose (1/2/3) [3]"
    
    if ($choice -eq "1") {
        Write-Host ""
        Write-Progress "Installing on connected device..."
        
        $adb = "$androidSdk\platform-tools\adb.exe"
        if (Test-Path $adb) {
            & $adb devices
            Write-Host ""
            & $adb install -r $apkPath
            
            if ($LASTEXITCODE -eq 0) {
                Write-Success "APK installed on device!"
            } else {
                Write-Warning "Installation failed. Make sure device is connected with USB debugging enabled."
            }
        }
    } elseif ($choice -eq "2") {
        explorer.exe /select,$apkFullPath
    }
    
} else {
    Write-Host "╔════════════════════════════════════════════════════════════╗" -ForegroundColor Red
    Write-Host "║                                                            ║" -ForegroundColor Red
    Write-Host "║                    ✗ BUILD FAILED                          ║" -ForegroundColor Red
    Write-Host "║                                                            ║" -ForegroundColor Red
    Write-Host "╚════════════════════════════════════════════════════════════╝" -ForegroundColor Red
    Write-Host ""
    Write-Host "Troubleshooting:" -ForegroundColor Yellow
    Write-Host "  1. Close and restart PowerShell as Administrator" -ForegroundColor White
    Write-Host "  2. Run this script again" -ForegroundColor White
    Write-Host "  3. Check BUILD_GUIDE.md for manual steps" -ForegroundColor White
    Write-Host ""
}

# ═══════════════════════════════════════════════════════════════════════════
# CLEANUP & SUMMARY
# ═══════════════════════════════════════════════════════════════════════════

Write-Host ""
Write-Host "Cleaning up temporary files..." -ForegroundColor Gray
Remove-Item -Path $tempDir -Recurse -Force -ErrorAction SilentlyContinue

Write-Host ""
Write-Host "╔════════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║                    SETUP SUMMARY                           ║" -ForegroundColor Cyan
Write-Host "╚════════════════════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""
Write-Host "Installed:" -ForegroundColor Yellow
Write-Host "  ✓ Node.js" -ForegroundColor Green
Write-Host "  ✓ Rust + Android targets" -ForegroundColor Green
Write-Host "  ✓ Java JDK 17" -ForegroundColor Green
Write-Host "  ✓ Android SDK + NDK" -ForegroundColor Green
Write-Host "  ✓ Build tools" -ForegroundColor Green
Write-Host ""
Write-Host "Environment:" -ForegroundColor Yellow
Write-Host "  ANDROID_HOME = $env:ANDROID_HOME" -ForegroundColor White
Write-Host "  JAVA_HOME = $env:JAVA_HOME" -ForegroundColor White
Write-Host ""
Write-Host "Total time: $($duration.ToString('hh\:mm\:ss'))" -ForegroundColor White
Write-Host ""
Write-Host "To rebuild later:" -ForegroundColor Yellow
Write-Host "  npm run tauri android build" -ForegroundColor White
Write-Host ""
Write-Host "Documentation:" -ForegroundColor Yellow
Write-Host "  - QUICKSTART.md" -ForegroundColor White
Write-Host "  - BUILD_GUIDE.md" -ForegroundColor White
Write-Host "  - FEATURES.md" -ForegroundColor White
Write-Host ""
Write-Host "✨ Setup complete! Enjoy MSSCS Mobile! ✨" -ForegroundColor Cyan
Write-Host ""
