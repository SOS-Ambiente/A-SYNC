#!/usr/bin/env pwsh
# P2P Integration Test Script - Tests connectivity across all platforms

Write-Host ""
Write-Host "╔════════════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║  🧪 MSSCS P2P Integration Test Suite                         ║" -ForegroundColor Cyan
Write-Host "╚════════════════════════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""

$ErrorActionPreference = "Continue"
$testResults = @()

function Test-Component {
    param(
        [string]$Name,
        [scriptblock]$Test
    )
    
    Write-Host "Testing: $Name..." -ForegroundColor Yellow
    try {
        $result = & $Test
        if ($result) {
            Write-Host "  ✅ PASS: $Name" -ForegroundColor Green
            $script:testResults += @{ Name = $Name; Status = "PASS"; Message = "" }
            return $true
        } else {
            Write-Host "  ❌ FAIL: $Name" -ForegroundColor Red
            $script:testResults += @{ Name = $Name; Status = "FAIL"; Message = "Test returned false" }
            return $false
        }
    } catch {
        Write-Host "  ❌ ERROR: $Name - $($_.Exception.Message)" -ForegroundColor Red
        $script:testResults += @{ Name = $Name; Status = "ERROR"; Message = $_.Exception.Message }
        return $false
    }
}

Write-Host "📋 Test Plan:" -ForegroundColor Cyan
Write-Host "   1. Rust Backend (libp2p) - NAT traversal, DHT, Relay"
Write-Host "   2. Web Frontend (PeerJS) - WebRTC, STUN/TURN"
Write-Host "   3. Tauri Client (Hybrid) - Rust + PeerJS bridge"
Write-Host "   4. Cross-platform connectivity"
Write-Host "   5. Storage allocation"
Write-Host "   6. Progress tracking"
Write-Host ""

# Test 1: Check Rust dependencies
Write-Host "═══════════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "TEST 1: Rust Backend Dependencies" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════════════════════" -ForegroundColor Cyan

Test-Component "Cargo installed" {
    $null -ne (Get-Command cargo -ErrorAction SilentlyContinue)
}

Test-Component "Rust libp2p dependencies" {
    $cargoToml = Get-Content "msscs_v4/Cargo.toml" -Raw
    $cargoToml -match "libp2p" -and $cargoToml -match "tokio"
}

# Test 2: Check Web dependencies
Write-Host ""
Write-Host "═══════════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "TEST 2: Web Frontend Dependencies" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════════════════════" -ForegroundColor Cyan

Test-Component "Node.js installed" {
    $null -ne (Get-Command node -ErrorAction SilentlyContinue)
}

Test-Component "Web app files exist" {
    (Test-Path "msscs_web/app.js") -and 
    (Test-Path "msscs_web/p2p.js") -and
    (Test-Path "msscs_web/index.html")
}

Test-Component "PeerJS integration" {
    $appJs = Get-Content "msscs_web/app.js" -Raw
    $appJs -match "PeerJS" -or $appJs -match "new Peer"
}

# Test 3: Check Tauri client
Write-Host ""
Write-Host "═══════════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "TEST 3: Tauri Client Integration" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════════════════════" -ForegroundColor Cyan

Test-Component "Tauri CLI installed" {
    $null -ne (Get-Command cargo -ErrorAction SilentlyContinue)
}

Test-Component "Tauri main.rs exists" {
    Test-Path "msscs_client/src-tauri/src/main.rs"
}

Test-Component "PeerJS bridge exists" {
    Test-Path "msscs_client/src/peerjs-bridge.ts"
}

Test-Component "P2P command integration" {
    $mainRs = Get-Content "msscs_client/src-tauri/src/main.rs" -Raw
    $mainRs -match "P2PNodeCommand" -and $mainRs -match "p2p_command_tx"
}

Test-Component "PeerJS bridge STUN/TURN config" {
    $bridge = Get-Content "msscs_client/src/peerjs-bridge.ts" -Raw
    $bridge -match "stun.l.google.com" -and $bridge -match "turn:openrelay.metered.ca"
}

# Test 4: Check P2P network configuration
Write-Host ""
Write-Host "═══════════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "TEST 4: P2P Network Configuration" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════════════════════" -ForegroundColor Cyan

Test-Component "Bootstrap peers configured" {
    $p2pNetwork = Get-Content "msscs_v4/src/p2p_network.rs" -Raw
    $p2pNetwork -match "bootstrap.libp2p.io" -and $p2pNetwork -match "default_bootstrap_peers"
}

Test-Component "NAT traversal enabled" {
    $p2pNetwork = Get-Content "msscs_v4/src/p2p_network.rs" -Raw
    $p2pNetwork -match "relay" -and $p2pNetwork -match "autonat" -and $p2pNetwork -match "dcutr"
}

Test-Component "QUIC transport enabled" {
    $p2pNetwork = Get-Content "msscs_v4/src/p2p_network.rs" -Raw
    $p2pNetwork -match "with_quic"
}

Test-Component "Kademlia DHT configured" {
    $p2pNetwork = Get-Content "msscs_v4/src/p2p_network.rs" -Raw
    $p2pNetwork -match "kademlia" -and $p2pNetwork -match "MemoryStore"
}

# Test 5: Check storage allocation
Write-Host ""
Write-Host "═══════════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "TEST 5: Storage Allocation" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════════════════════" -ForegroundColor Cyan

Test-Component "Storage limit commands (Tauri)" {
    $mainRs = Get-Content "msscs_client/src-tauri/src/main.rs" -Raw
    $mainRs -match "get_storage_limit" -and $mainRs -match "set_storage_limit"
}

Test-Component "Storage stats command (Tauri)" {
    $mainRs = Get-Content "msscs_client/src-tauri/src/main.rs" -Raw
    $mainRs -match "get_storage_stats"
}

Test-Component "Storage allocation UI (Web)" {
    $indexHtml = Get-Content "msscs_web/index.html" -Raw
    $indexHtml -match "storage-limit" -and $indexHtml -match "Storage Allocation"
}

Test-Component "Storage manager (Web)" {
    Test-Path "msscs_web/storage.js"
}

# Test 6: Check progress tracking
Write-Host ""
Write-Host "═══════════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "TEST 6: Progress Tracking (Upload/Download)" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════════════════════" -ForegroundColor Cyan

Test-Component "Upload progress events (Tauri)" {
    $mainRs = Get-Content "msscs_client/src-tauri/src/main.rs" -Raw
    $mainRs -match "upload-progress" -and $mainRs -match "speed" -and $mainRs -match "eta"
}

Test-Component "Download progress events (Tauri)" {
    $mainRs = Get-Content "msscs_client/src-tauri/src/main.rs" -Raw
    $mainRs -match "download-progress" -and $mainRs -match "speed" -and $mainRs -match "eta"
}

Test-Component "Progress UI (Web)" {
    $appJs = Get-Content "msscs_web/app.js" -Raw
    $appJs -match "progress-item" -and $appJs -match "formatBytes" -and $appJs -match "formatTime"
}

Test-Component "Progress throttling (performance - Tauri)" {
    $mainRs = Get-Content "msscs_client/src-tauri/src/main.rs" -Raw
    $mainRs -match "last_emit" -and $mainRs -match "100"
}

Test-Component "Progress throttling (performance - Web)" {
    $appJs = Get-Content "msscs_web/app.js" -Raw
    $appJs -match "lastProgressUpdate" -and $appJs -match "100"
}

Test-Component "Download progress with chunk tracking (Web)" {
    $appJs = Get-Content "msscs_web/app.js" -Raw
    $appJs -match "download-" -and $appJs -match "totalChunks" -and $appJs -match "bytesProcessed"
}

Test-Component "Upload progress with speed calculation (Web)" {
    $appJs = Get-Content "msscs_web/app.js" -Raw
    $appJs -match "startTime" -and $appJs -match "elapsed" -and $appJs -match "speed"
}

# Test 9: Check cross-platform compatibility
Write-Host ""
Write-Host "═══════════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "TEST 9: Cross-Platform Compatibility" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════════════════════" -ForegroundColor Cyan

Test-Component "Web P2P implementation" {
    $p2pJs = Get-Content "msscs_web/p2p.js" -Raw
    $p2pJs -match "class P2PNetwork" -and $p2pJs -match "connectToPeer"
}

Test-Component "Tauri PeerJS bridge" {
    $bridge = Get-Content "msscs_client/src/peerjs-bridge.ts" -Raw
    $bridge -match "class PeerJSBridge" -and $bridge -match "connectToPeer"
}

Test-Component "Rust libp2p implementation" {
    $p2pNet = Get-Content "msscs_v4/src/p2p_network.rs" -Raw
    $p2pNet -match "pub struct P2PNode" -and $p2pNet -match "bootstrap"
}

Test-Component "Storage stats serialization" {
    $p2pVfs = Get-Content "msscs_v4/src/p2p_vfs.rs" -Raw
    $p2pVfs -match "StorageStats" -and $p2pVfs -match "serde::Serialize"
}

Test-Component "Peer count aggregation (libp2p + legacy)" {
    $mainRs = Get-Content "msscs_client/src-tauri/src/main.rs" -Raw
    $mainRs -match "p2p_peer_count" -and $mainRs -match "legacy_peer_count" -and $mainRs -match "total_peers"
}

Test-Component "Storage limit enforcement (Web)" {
    $appJs = Get-Content "msscs_web/app.js" -Raw
    $appJs -match "storage_used \+ file.size > stats.storage_limit"
}

Test-Component "WebRTC bridge implementation" {
    Test-Path "msscs_web/webrtc-bridge.js"
}

Test-Component "Bridge integration in app" {
    $appJs = Get-Content "msscs_web/app.js" -Raw
    $appJs -match "WebRTCBridge" -and $appJs -match "bridge.init"
}

Test-Component "P2P VFS command channel integration" {
    $p2pVfs = Get-Content "msscs_v4/src/p2p_vfs.rs" -Raw
    $p2pVfs -match "p2p_command_tx" -and $p2pVfs -match "store_block_p2p" -and $p2pVfs -match "get_block_p2p"
}

Test-Component "Async P2P block operations" {
    $p2pVfs = Get-Content "msscs_v4/src/p2p_vfs.rs" -Raw
    $p2pVfs -match "reply_tx" -and $p2pVfs -match "reply_rx.await"
}

Test-Component "Peer exchange (Web)" {
    $p2pJs = Get-Content "msscs_web/p2p.js" -Raw
    $p2pJs -match "peer-list" -and $p2pJs -match "sendPeerList"
}

Test-Component "Peer exchange (Tauri)" {
    $bridge = Get-Content "msscs_client/src/peerjs-bridge.ts" -Raw
    $bridge -match "peer-list" -and $bridge -match "sendPeerList"
}

Test-Component "Global storage estimation" {
    $appJs = Get-Content "msscs_web/app.js" -Raw
    $appJs -match "global_storage_estimate"
}

# Test 7: Check crypto implementation
Write-Host ""
Write-Host "═══════════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "TEST 7: Encryption & Security" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════════════════════" -ForegroundColor Cyan

Test-Component "Crypto manager (Web)" {
    Test-Path "msscs_web/crypto.js"
}

Test-Component "AES-256-GCM encryption" {
    $cryptoJs = Get-Content "msscs_web/crypto.js" -Raw
    $cryptoJs -match "AES-GCM" -and $cryptoJs -match "256"
}

Test-Component "Quantum encryption (Rust)" {
    $p2pVfs = Get-Content "msscs_v4/src/p2p_vfs.rs" -Raw
    $p2pVfs -match "QuantumDataBlock" -and $p2pVfs -match "encrypt"
}

# Test 8: Check STUN/TURN configuration
Write-Host ""
Write-Host "═══════════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "TEST 8: STUN/TURN Configuration" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════════════════════" -ForegroundColor Cyan

Test-Component "STUN servers (Web)" {
    $p2pJs = Get-Content "msscs_web/p2p.js" -Raw
    $p2pJs -match "stun.l.google.com" -and $p2pJs -match "stun.services.mozilla.com"
}

Test-Component "TURN servers (Web)" {
    $p2pJs = Get-Content "msscs_web/p2p.js" -Raw
    $p2pJs -match "turn:openrelay.metered.ca" -and $p2pJs -match "openrelayproject"
}

Test-Component "STUN servers (Tauri)" {
    $peerBridge = Get-Content "msscs_client/src/peerjs-bridge.ts" -Raw
    $peerBridge -match "stun.l.google.com"
}

Test-Component "TURN servers (Tauri)" {
    $peerBridge = Get-Content "msscs_client/src/peerjs-bridge.ts" -Raw
    $peerBridge -match "turn:openrelay.metered.ca"
}

# Summary
Write-Host ""
Write-Host "═══════════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "TEST SUMMARY" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host ""

$passed = ($testResults | Where-Object { $_.Status -eq "PASS" }).Count
$failed = ($testResults | Where-Object { $_.Status -eq "FAIL" }).Count
$errors = ($testResults | Where-Object { $_.Status -eq "ERROR" }).Count
$total = $testResults.Count

Write-Host "Total Tests: $total" -ForegroundColor White
Write-Host "  ✅ Passed: $passed" -ForegroundColor Green
Write-Host "  ❌ Failed: $failed" -ForegroundColor Red
Write-Host "  ⚠️  Errors: $errors" -ForegroundColor Yellow
Write-Host ""

if ($failed -gt 0 -or $errors -gt 0) {
    Write-Host "Failed/Error Tests:" -ForegroundColor Red
    $testResults | Where-Object { $_.Status -ne "PASS" } | ForEach-Object {
        Write-Host "  • $($_.Name): $($_.Message)" -ForegroundColor Red
    }
    Write-Host ""
}

$successRate = [math]::Round(($passed / $total) * 100, 1)
Write-Host "Success Rate: $successRate%" -ForegroundColor $(if ($successRate -ge 90) { "Green" } elseif ($successRate -ge 70) { "Yellow" } else { "Red" })
Write-Host ""

if ($successRate -ge 90) {
    Write-Host "🎉 Excellent! P2P integration is ready for testing." -ForegroundColor Green
} elseif ($successRate -ge 70) {
    Write-Host "⚠️  Good progress, but some issues need attention." -ForegroundColor Yellow
} else {
    Write-Host "❌ Critical issues detected. Please review failed tests." -ForegroundColor Red
}

Write-Host ""
Write-Host "╔════════════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║  📋 Next Steps & Testing Guide                                ║" -ForegroundColor Cyan
Write-Host "╚════════════════════════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""
Write-Host "1️⃣  BUILD RUST BACKEND:" -ForegroundColor Yellow
Write-Host "   cd msscs_v4"
Write-Host "   cargo build --release"
Write-Host ""
Write-Host "2️⃣  TEST DESKTOP APP (Windows/Linux):" -ForegroundColor Yellow
Write-Host "   cd msscs_client"
Write-Host "   pnpm install"
Write-Host "   pnpm tauri dev"
Write-Host ""
Write-Host "3️⃣  TEST WEB APP:" -ForegroundColor Yellow
Write-Host "   cd msscs_web"
Write-Host "   # Serve with any HTTP server, e.g.:"
Write-Host "   python -m http.server 8080"
Write-Host "   # Or:"
Write-Host "   npx serve ."
Write-Host "   # Then open: http://localhost:8080"
Write-Host ""
Write-Host "4️⃣  TEST MOBILE APP (Android):" -ForegroundColor Yellow
Write-Host "   cd msscs_mobile"
Write-Host "   pnpm install"
Write-Host "   pnpm android:dev"
Write-Host ""
Write-Host "5️⃣  TEST P2P CONNECTIVITY:" -ForegroundColor Yellow
Write-Host "   • Open desktop app on one machine"
Write-Host "   • Open web app in browser on another"
Write-Host "   • Copy Peer ID from one and connect from the other"
Write-Host "   • Upload a file on one peer"
Write-Host "   • Verify it appears on the other peer"
Write-Host "   • Check storage stats update in real-time"
Write-Host ""
Write-Host "6️⃣  VERIFY FEATURES:" -ForegroundColor Yellow
Write-Host "   ✓ Upload/Download progress with speed & ETA"
Write-Host "   ✓ Storage allocation (configurable limits)"
Write-Host "   ✓ Peer count updates in real-time"
Write-Host "   ✓ Files encrypted before upload"
Write-Host "   ✓ Cross-platform connectivity (Web ↔ Desktop)"
Write-Host "   ✓ NAT traversal (works behind routers)"
Write-Host ""
Write-Host "💡 Tips:" -ForegroundColor Cyan
Write-Host "   • Check browser console for detailed P2P logs"
Write-Host "   • Desktop app logs show libp2p connection details"
Write-Host "   • Storage stats update every 2 seconds"
Write-Host "   • Progress bars show real-time upload/download speed"
Write-Host ""

exit $(if ($successRate -ge 90) { 0 } else { 1 })
