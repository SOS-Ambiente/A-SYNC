# P2P Network Demo Script
# Demonstrates the global P2P storage network

Write-Host "🌐 MSSCS v4 - P2P Network Demo" -ForegroundColor Cyan
Write-Host "=" * 60

# Build the project
Write-Host "`n📦 Building project..." -ForegroundColor Yellow
cargo build --release --example p2p_demo

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Build failed!" -ForegroundColor Red
    exit 1
}

Write-Host "✅ Build successful!" -ForegroundColor Green

# Run the demo
Write-Host "`n🚀 Running P2P demo..." -ForegroundColor Yellow
cargo run --release --example p2p_demo

Write-Host "`n✅ Demo complete!" -ForegroundColor Green
