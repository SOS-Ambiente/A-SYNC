# MSSCS v4.0 Demo Script
# Demonstrates the complete functionality of the distributed storage system

Write-Host "=== MSSCS v4.0 Demo ===" -ForegroundColor Cyan
Write-Host ""

# Check if binary exists
if (-not (Test-Path "target/release/msscs_v4.exe")) {
    Write-Host "Building MSSCS v4.0..." -ForegroundColor Yellow
    cargo build --release
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Build failed!" -ForegroundColor Red
        exit 1
    }
}

# Clean up any existing data
Write-Host "Cleaning up previous data..." -ForegroundColor Yellow
Remove-Item -Path "msscs_data_node1" -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item -Path "msscs_data_node2" -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item -Path "msscs_data_node3" -Recurse -Force -ErrorAction SilentlyContinue

# Create config files for 3 nodes
Write-Host "Creating configuration files..." -ForegroundColor Yellow

$config1 = @"
port = 8080
data_dir = "./msscs_data_node1"
replication_factor = 2
chunk_size = 1024
log_level = "info"
bootstrap_peers = []
"@

$config2 = @"
port = 8081
data_dir = "./msscs_data_node2"
replication_factor = 2
chunk_size = 1024
log_level = "info"
bootstrap_peers = ["127.0.0.1:8080"]
"@

$config3 = @"
port = 8082
data_dir = "./msscs_data_node3"
replication_factor = 2
chunk_size = 1024
log_level = "info"
bootstrap_peers = ["127.0.0.1:8080", "127.0.0.1:8081"]
"@

$config1 | Out-File -FilePath "config_node1.toml" -Encoding UTF8
$config2 | Out-File -FilePath "config_node2.toml" -Encoding UTF8
$config3 | Out-File -FilePath "config_node3.toml" -Encoding UTF8

# Start nodes in background
Write-Host ""
Write-Host "Starting Node 1 (port 8080)..." -ForegroundColor Green
$node1 = Start-Process -FilePath "target/release/msscs_v4.exe" -ArgumentList "--config config_node1.toml" -PassThru -WindowStyle Hidden

Start-Sleep -Seconds 2

Write-Host "Starting Node 2 (port 8081)..." -ForegroundColor Green
$node2 = Start-Process -FilePath "target/release/msscs_v4.exe" -ArgumentList "--config config_node2.toml" -PassThru -WindowStyle Hidden

Start-Sleep -Seconds 2

Write-Host "Starting Node 3 (port 8082)..." -ForegroundColor Green
$node3 = Start-Process -FilePath "target/release/msscs_v4.exe" -ArgumentList "--config config_node3.toml" -PassThru -WindowStyle Hidden

Start-Sleep -Seconds 3

Write-Host ""
Write-Host "All nodes started!" -ForegroundColor Green
Write-Host ""

# Function to make API calls
function Invoke-MSSCS {
    param(
        [string]$Method = "GET",
        [string]$Port = "8080",
        [string]$Path,
        [string]$Body
    )
    
    $uri = "http://127.0.0.1:$Port$Path"
    
    try {
        if ($Body) {
            $response = Invoke-RestMethod -Uri $uri -Method $Method -Body $Body -ContentType "application/json"
        } else {
            $response = Invoke-RestMethod -Uri $uri -Method $Method
        }
        return $response
    } catch {
        Write-Host "Error: $_" -ForegroundColor Red
        return $null
    }
}

# Test 1: Health Check
Write-Host "=== Test 1: Health Check ===" -ForegroundColor Cyan
$health1 = Invoke-MSSCS -Port "8080" -Path "/health"
$health2 = Invoke-MSSCS -Port "8081" -Path "/health"
$health3 = Invoke-MSSCS -Port "8082" -Path "/health"

Write-Host "Node 1: $($health1.status) - Peers: $($health1.peers)" -ForegroundColor White
Write-Host "Node 2: $($health2.status) - Peers: $($health2.peers)" -ForegroundColor White
Write-Host "Node 3: $($health3.status) - Peers: $($health3.peers)" -ForegroundColor White
Write-Host ""

# Test 2: Write a file to Node 1
Write-Host "=== Test 2: Write File to Node 1 ===" -ForegroundColor Cyan
$testData = "Hello, MSSCS! This is a distributed storage system with compression and encryption."
$base64Data = [Convert]::ToBase64String([System.Text.Encoding]::UTF8.GetBytes($testData))

$writeBody = @{
    path = "test.txt"
    content = $base64Data
} | ConvertTo-Json

$writeResult = Invoke-MSSCS -Method "POST" -Port "8080" -Path "/files" -Body $writeBody
Write-Host "File written: UUID=$($writeResult.uuid), Blocks=$($writeResult.blocks)" -ForegroundColor White
Write-Host ""

Start-Sleep -Seconds 2

# Test 3: Read file from Node 2 (should fetch from network)
Write-Host "=== Test 3: Read File from Node 2 ===" -ForegroundColor Cyan
$readResult = Invoke-MSSCS -Port "8081" -Path "/files/test.txt"
if ($readResult) {
    $decodedData = [System.Text.Encoding]::UTF8.GetString([Convert]::FromBase64String($readResult.content))
    Write-Host "File read successfully: $decodedData" -ForegroundColor White
}
Write-Host ""

# Test 4: Write a larger file
Write-Host "=== Test 4: Write Larger File ===" -ForegroundColor Cyan
$largeData = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. " * 100
$base64Large = [Convert]::ToBase64String([System.Text.Encoding]::UTF8.GetBytes($largeData))

$writeLargeBody = @{
    path = "large.txt"
    content = $base64Large
} | ConvertTo-Json

$writeLargeResult = Invoke-MSSCS -Method "POST" -Port "8082" -Path "/files" -Body $writeLargeBody
Write-Host "Large file written: UUID=$($writeLargeResult.uuid), Blocks=$($writeLargeResult.blocks)" -ForegroundColor White
Write-Host ""

Start-Sleep -Seconds 2

# Test 5: List files from all nodes
Write-Host "=== Test 5: List Files ===" -ForegroundColor Cyan
$files1 = Invoke-MSSCS -Port "8080" -Path "/files"
$files2 = Invoke-MSSCS -Port "8081" -Path "/files"
$files3 = Invoke-MSSCS -Port "8082" -Path "/files"

Write-Host "Node 1 files: $($files1.files -join ', ')" -ForegroundColor White
Write-Host "Node 2 files: $($files2.files -join ', ')" -ForegroundColor White
Write-Host "Node 3 files: $($files3.files -join ', ')" -ForegroundColor White
Write-Host ""

# Test 6: View metrics
Write-Host "=== Test 6: System Metrics ===" -ForegroundColor Cyan
$metrics1 = Invoke-MSSCS -Port "8080" -Path "/metrics"
Write-Host "Node 1 Metrics:" -ForegroundColor Yellow
Write-Host "  Blocks: $($metrics1.block_count)" -ForegroundColor White
Write-Host "  Storage: $($metrics1.storage_bytes) bytes" -ForegroundColor White
Write-Host "  Peers: $($metrics1.peer_count)" -ForegroundColor White
Write-Host "  Uptime: $($metrics1.uptime_seconds) seconds" -ForegroundColor White
Write-Host "  Requests: $($metrics1.requests_total) (Success rate: $($metrics1.success_rate)%)" -ForegroundColor White
Write-Host ""

# Test 7: Delete a file
Write-Host "=== Test 7: Delete File ===" -ForegroundColor Cyan
$deleteResult = Invoke-MSSCS -Method "DELETE" -Port "8080" -Path "/files/test.txt"
Write-Host "File deleted: $($deleteResult.status)" -ForegroundColor White
Write-Host ""

# Test 8: Verify deletion
Write-Host "=== Test 8: Verify Deletion ===" -ForegroundColor Cyan
$filesAfterDelete = Invoke-MSSCS -Port "8080" -Path "/files"
Write-Host "Remaining files: $($filesAfterDelete.files -join ', ')" -ForegroundColor White
Write-Host ""

# Cleanup
Write-Host "=== Demo Complete ===" -ForegroundColor Green
Write-Host ""
Write-Host "Press any key to stop nodes and cleanup..." -ForegroundColor Yellow
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")

Write-Host ""
Write-Host "Stopping nodes..." -ForegroundColor Yellow
Stop-Process -Id $node1.Id -Force -ErrorAction SilentlyContinue
Stop-Process -Id $node2.Id -Force -ErrorAction SilentlyContinue
Stop-Process -Id $node3.Id -Force -ErrorAction SilentlyContinue

Write-Host "Cleaning up..." -ForegroundColor Yellow
Remove-Item -Path "config_node1.toml" -Force -ErrorAction SilentlyContinue
Remove-Item -Path "config_node2.toml" -Force -ErrorAction SilentlyContinue
Remove-Item -Path "config_node3.toml" -Force -ErrorAction SilentlyContinue
Remove-Item -Path "msscs_data_node1" -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item -Path "msscs_data_node2" -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item -Path "msscs_data_node3" -Recurse -Force -ErrorAction SilentlyContinue

Write-Host ""
Write-Host "Demo cleanup complete!" -ForegroundColor Green
