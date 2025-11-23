# Quick Start P2P Network - Test Internet Connectivity
# This script starts multiple MSSCS nodes that can discover each other

Write-Host "🌐 MSSCS P2P Network Quick Start" -ForegroundColor Cyan
Write-Host "=================================" -ForegroundColor Cyan
Write-Host ""

# Check if cargo is available
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Error: Cargo not found. Please install Rust." -ForegroundColor Red
    exit 1
}

Write-Host "Building MSSCS node..." -ForegroundColor Yellow
cargo build --release --bin msscs_node
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Build failed!" -ForegroundColor Red
    exit 1
}

Write-Host "✅ Build successful!" -ForegroundColor Green
Write-Host ""

# Create data directories
$node1Dir = ".\msscs_data\node1"
$node2Dir = ".\msscs_data\node2"
$node3Dir = ".\msscs_data\node3"

New-Item -ItemType Directory -Force -Path $node1Dir | Out-Null
New-Item -ItemType Directory -Force -Path $node2Dir | Out-Null
New-Item -ItemType Directory -Force -Path $node3Dir | Out-Null

Write-Host "📁 Data directories created" -ForegroundColor Green
Write-Host ""

Write-Host "🚀 Starting P2P Network..." -ForegroundColor Cyan
Write-Host ""
Write-Host "This will start 3 nodes that will discover each other via mDNS" -ForegroundColor Yellow
Write-Host "Press Ctrl+C in any window to stop that node" -ForegroundColor Yellow
Write-Host ""

# Start Node 1 (Bootstrap node)
Write-Host "Starting Node 1 (Bootstrap) on ports 8080/4001..." -ForegroundColor Green
Start-Process powershell -ArgumentList "-NoExit", "-Command", @"
Set-Location '$PWD'
Write-Host '🟢 Node 1 - Bootstrap Node' -ForegroundColor Green
Write-Host 'HTTP API: http://localhost:8080' -ForegroundColor Cyan
Write-Host 'P2P Port: 4001' -ForegroundColor Cyan
Write-Host ''
.\target\release\msscs_node.exe --port 8080 --p2p-port 4001 --mdns true --passphrase 'node1-secure-pass'
"@

Start-Sleep -Seconds 3

# Start Node 2
Write-Host "Starting Node 2 on ports 8081/4002..." -ForegroundColor Green
Start-Process powershell -ArgumentList "-NoExit", "-Command", @"
Set-Location '$PWD'
Write-Host '🟡 Node 2 - Peer Node' -ForegroundColor Yellow
Write-Host 'HTTP API: http://localhost:8081' -ForegroundColor Cyan
Write-Host 'P2P Port: 4002' -ForegroundColor Cyan
Write-Host ''
.\target\release\msscs_node.exe --port 8081 --p2p-port 4002 --mdns true --passphrase 'node2-secure-pass'
"@

Start-Sleep -Seconds 3

# Start Node 3
Write-Host "Starting Node 3 on ports 8082/4003..." -ForegroundColor Green
Start-Process powershell -ArgumentList "-NoExit", "-Command", @"
Set-Location '$PWD'
Write-Host '🔵 Node 3 - Peer Node' -ForegroundColor Blue
Write-Host 'HTTP API: http://localhost:8082' -ForegroundColor Cyan
Write-Host 'P2P Port: 4003' -ForegroundColor Cyan
Write-Host ''
.\target\release\msscs_node.exe --port 8082 --p2p-port 4003 --mdns true --passphrase 'node3-secure-pass'
"@

Write-Host ""
Write-Host "✅ All nodes started!" -ForegroundColor Green
Write-Host ""
Write-Host "📡 Network Status:" -ForegroundColor Cyan
Write-Host "   Node 1: http://localhost:8080 (P2P: 4001)" -ForegroundColor White
Write-Host "   Node 2: http://localhost:8081 (P2P: 4002)" -ForegroundColor White
Write-Host "   Node 3: http://localhost:8082 (P2P: 4003)" -ForegroundColor White
Write-Host ""
Write-Host "🔍 The nodes will discover each other automatically via mDNS" -ForegroundColor Yellow
Write-Host ""
Write-Host "📝 Test the network:" -ForegroundColor Cyan
Write-Host "   1. Upload a file to Node 1:" -ForegroundColor White
Write-Host "      .\target\release\p2p_client.exe --server http://localhost:8080 upload test.txt --path /test.txt" -ForegroundColor Gray
Write-Host ""
Write-Host "   2. Download from Node 2 (should replicate):" -ForegroundColor White
Write-Host "      .\target\release\p2p_client.exe --server http://localhost:8081 download /test.txt --output downloaded.txt" -ForegroundColor Gray
Write-Host ""
Write-Host "   3. Check stats on any node:" -ForegroundColor White
Write-Host "      .\target\release\p2p_client.exe --server http://localhost:8080 stats" -ForegroundColor Gray
Write-Host ""
Write-Host "Press any key to exit (nodes will continue running)..." -ForegroundColor Yellow
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
