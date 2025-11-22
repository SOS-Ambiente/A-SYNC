# MSSCS Client Setup Script for Windows
# Run this script to set up the development environment

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  MSSCS Desktop Client Setup" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Check Node.js
Write-Host "Checking Node.js..." -ForegroundColor Yellow
try {
    $nodeVersion = node --version
    Write-Host "✓ Node.js found: $nodeVersion" -ForegroundColor Green
} catch {
    Write-Host "✗ Node.js not found!" -ForegroundColor Red
    Write-Host "  Please install from: https://nodejs.org/" -ForegroundColor Yellow
    exit 1
}

# Check npm
Write-Host "Checking npm..." -ForegroundColor Yellow
try {
    $npmVersion = npm --version
    Write-Host "✓ npm found: v$npmVersion" -ForegroundColor Green
} catch {
    Write-Host "✗ npm not found!" -ForegroundColor Red
    exit 1
}

# Check Rust
Write-Host "Checking Rust..." -ForegroundColor Yellow
try {
    $rustVersion = rustc --version
    Write-Host "✓ Rust found: $rustVersion" -ForegroundColor Green
} catch {
    Write-Host "✗ Rust not found!" -ForegroundColor Red
    Write-Host "  Please install from: https://rustup.rs/" -ForegroundColor Yellow
    exit 1
}

# Check Cargo
Write-Host "Checking Cargo..." -ForegroundColor Yellow
try {
    $cargoVersion = cargo --version
    Write-Host "✓ Cargo found: $cargoVersion" -ForegroundColor Green
} catch {
    Write-Host "✗ Cargo not found!" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "All prerequisites found!" -ForegroundColor Green
Write-Host ""

# Install npm dependencies
Write-Host "Installing npm dependencies..." -ForegroundColor Yellow
npm install
if ($LASTEXITCODE -ne 0) {
    Write-Host "✗ npm install failed!" -ForegroundColor Red
    exit 1
}
Write-Host "✓ npm dependencies installed" -ForegroundColor Green
Write-Host ""

# Create data directory
Write-Host "Creating data directory..." -ForegroundColor Yellow
if (!(Test-Path "msscs_data")) {
    New-Item -ItemType Directory -Path "msscs_data" | Out-Null
    Write-Host "✓ Data directory created" -ForegroundColor Green
} else {
    Write-Host "✓ Data directory already exists" -ForegroundColor Green
}
Write-Host ""

# Create default config if not exists
Write-Host "Checking configuration..." -ForegroundColor Yellow
if (!(Test-Path "config.toml")) {
    Copy-Item "../msscs_v4/config.toml.example" "config.toml"
    Write-Host "✓ Default config created" -ForegroundColor Green
} else {
    Write-Host "✓ Config already exists" -ForegroundColor Green
}
Write-Host ""

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Setup Complete!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Yellow
Write-Host "  1. Run 'npm run tauri dev' to start development" -ForegroundColor White
Write-Host "  2. Or run 'npm run tauri build' to create installer" -ForegroundColor White
Write-Host ""
Write-Host "Documentation:" -ForegroundColor Yellow
Write-Host "  - README.md for full docs" -ForegroundColor White
Write-Host "  - QUICKSTART.md for quick start" -ForegroundColor White
Write-Host "  - UI_DESIGN.md for design specs" -ForegroundColor White
Write-Host ""
Write-Host "Happy coding! 🚀" -ForegroundColor Cyan
