# Test P2P Network - Start multiple nodes and test file sharing

Write-Host "🌐 MSSCS P2P Network Test" -ForegroundColor Cyan
Write-Host "=" * 60

# Build
Write-Host "`n📦 Building..." -ForegroundColor Yellow
cargo build --release --bin p2p_server --bin p2p_client

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Build failed!" -ForegroundColor Red
    exit 1
}

Write-Host "✅ Build successful!" -ForegroundColor Green

# Start Node 1
Write-Host "`n🚀 Starting Node 1 (port 8080)..." -ForegroundColor Yellow
Start-Process powershell -ArgumentList "-NoExit", "-Command", "cargo run --release --bin p2p_server -- --port 8080 --p2p-port 9000"

Start-Sleep -Seconds 3

# Start Node 2
Write-Host "🚀 Starting Node 2 (port 8081)..." -ForegroundColor Yellow
Start-Process powershell -ArgumentList "-NoExit", "-Command", "cargo run --release --bin p2p_server -- --port 8081 --p2p-port 9001"

Start-Sleep -Seconds 3

# Start Node 3
Write-Host "🚀 Starting Node 3 (port 8082)..." -ForegroundColor Yellow
Start-Process powershell -ArgumentList "-NoExit", "-Command", "cargo run --release --bin p2p_server -- --port 8082 --p2p-port 9002"

Start-Sleep -Seconds 3

Write-Host "`n✅ All nodes started!" -ForegroundColor Green
Write-Host "`n📡 Test the network with:" -ForegroundColor Cyan
Write-Host "   cargo run --bin p2p_client -- upload test.txt --path /test.txt"
Write-Host "   cargo run --bin p2p_client -- list"
Write-Host "   cargo run --bin p2p_client -- download /test.txt --output downloaded.txt"
Write-Host "   cargo run --bin p2p_client -- stats"
