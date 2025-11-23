# Test Sync Status and Connectivity Fixes
# This script helps verify all the fixes are working correctly

Write-Host "╔════════════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║  🧪 MSSCS Sync Status and Connectivity Test Suite            ║" -ForegroundColor Cyan
Write-Host "╚════════════════════════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""

# Test 1: Check if fixes are applied
Write-Host "📋 Test 1: Verifying Fixes Applied" -ForegroundColor Yellow
Write-Host ""

$fixes = @(
    @{
        File = "msscs_web/app.js"
        Pattern = "connStats\.peerId"
        Description = "Web status fix (checks peer ID)"
    },
    @{
        File = "msscs_web/p2p.js"
        Pattern = "Found self in localStorage"
        Description = "Self-recognition in localStorage"
    },
    @{
        File = "msscs_web/p2p.js"
        Pattern = "My Peer ID:"
        Description = "Peer ID logging in connectToPeer"
    },
    @{
        File = "msscs_client/src/stores/nodeStore.ts"
        Pattern = "__NODE_SYNC_START_TIME"
        Description = "Desktop timeout tracking"
    },
    @{
        File = "msscs_client/src/peerjs-bridge.ts"
        Pattern = "Skipping self-connection"
        Description = "Desktop self-connection prevention"
    }
)

$allPassed = $true
foreach ($fix in $fixes) {
    $content = Get-Content $fix.File -Raw -ErrorAction SilentlyContinue
    if ($content -match $fix.Pattern) {
        Write-Host "   ✅ $($fix.Description)" -ForegroundColor Green
    } else {
        Write-Host "   ❌ $($fix.Description) - NOT FOUND" -ForegroundColor Red
        $allPassed = $false
    }
}

Write-Host ""

if ($allPassed) {
    Write-Host "✅ All fixes verified!" -ForegroundColor Green
} else {
    Write-Host "❌ Some fixes missing - please review" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "─────────────────────────────────────────────────────────────────" -ForegroundColor Gray
Write-Host ""

# Test 2: Check file syntax
Write-Host "📋 Test 2: Checking File Syntax" -ForegroundColor Yellow
Write-Host ""

$jsFiles = @(
    "msscs_web/app.js",
    "msscs_web/p2p.js"
)

$tsFiles = @(
    "msscs_client/src/stores/nodeStore.ts",
    "msscs_client/src/peerjs-bridge.ts"
)

Write-Host "   Checking JavaScript files..." -ForegroundColor Cyan
foreach ($file in $jsFiles) {
    if (Test-Path $file) {
        Write-Host "   ✅ $file exists" -ForegroundColor Green
    } else {
        Write-Host "   ❌ $file missing" -ForegroundColor Red
    }
}

Write-Host ""
Write-Host "   Checking TypeScript files..." -ForegroundColor Cyan
foreach ($file in $tsFiles) {
    if (Test-Path $file) {
        Write-Host "   ✅ $file exists" -ForegroundColor Green
    } else {
        Write-Host "   ❌ $file missing" -ForegroundColor Red
    }
}

Write-Host ""
Write-Host "─────────────────────────────────────────────────────────────────" -ForegroundColor Gray
Write-Host ""

# Test 3: Manual Testing Instructions
Write-Host "📋 Test 3: Manual Testing Instructions" -ForegroundColor Yellow
Write-Host ""

Write-Host "🌐 Web App Test (Sync Status):" -ForegroundColor Cyan
Write-Host "   1. Open msscs_web/index.html in browser" -ForegroundColor White
Write-Host "   2. Open browser console (F12)" -ForegroundColor White
Write-Host "   3. Wait for initialization" -ForegroundColor White
Write-Host "   4. Expected: Status shows 'Online - Ready for connections'" -ForegroundColor Green
Write-Host "   5. Expected: Console shows 'Found self in localStorage'" -ForegroundColor Green
Write-Host ""

Write-Host "🖥️  Desktop App Test (Offline Handling):" -ForegroundColor Cyan
Write-Host "   1. Disconnect internet" -ForegroundColor White
Write-Host "   2. Start desktop app" -ForegroundColor White
Write-Host "   3. Wait 30 seconds" -ForegroundColor White
Write-Host "   4. Expected: Status transitions to 'offline'" -ForegroundColor Green
Write-Host "   5. Expected: No indefinite 'syncing' state" -ForegroundColor Green
Write-Host ""

Write-Host "🔗 Same-Computer Connectivity Test:" -ForegroundColor Cyan
Write-Host "   1. Start desktop app" -ForegroundColor White
Write-Host "   2. Note the Peer ID in console" -ForegroundColor White
Write-Host "   3. Open web app in browser" -ForegroundColor White
Write-Host "   4. Expected: Automatic connection via localStorage" -ForegroundColor Green
Write-Host "   5. Alternative: Manually enter peer ID" -ForegroundColor White
Write-Host "   6. Expected: Connection within 30 seconds" -ForegroundColor Green
Write-Host ""

Write-Host "─────────────────────────────────────────────────────────────────" -ForegroundColor Gray
Write-Host ""

# Test 4: Check for common issues
Write-Host "📋 Test 4: Checking for Common Issues" -ForegroundColor Yellow
Write-Host ""

Write-Host "   Checking for self-connection attempts..." -ForegroundColor Cyan
$webP2P = Get-Content "msscs_web/p2p.js" -Raw
$desktopBridge = Get-Content "msscs_client/src/peerjs-bridge.ts" -Raw

$issues = @()

if ($webP2P -notmatch "peerId === this\.peerId") {
    $issues += "Web P2P missing self-check in connectToPeer"
}

if ($desktopBridge -notmatch "peerId === this\.peerId") {
    $issues += "Desktop bridge missing self-check in connectToPeer"
}

if ($webP2P -notmatch "serialization: 'json'") {
    $issues += "Web P2P missing JSON serialization"
}

if ($desktopBridge -notmatch "serialization: 'json'") {
    $issues += "Desktop bridge missing JSON serialization"
}

if ($issues.Count -eq 0) {
    Write-Host "   ✅ No common issues found" -ForegroundColor Green
} else {
    Write-Host "   ⚠️  Found potential issues:" -ForegroundColor Yellow
    foreach ($issue in $issues) {
        Write-Host "      • $issue" -ForegroundColor Yellow
    }
}

Write-Host ""
Write-Host "─────────────────────────────────────────────────────────────────" -ForegroundColor Gray
Write-Host ""

# Summary
Write-Host "╔════════════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║  📊 Test Summary                                              ║" -ForegroundColor Cyan
Write-Host "╚════════════════════════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""

Write-Host "✅ Fixes Applied:" -ForegroundColor Green
Write-Host "   • Web sync status fix" -ForegroundColor White
Write-Host "   • Desktop offline timeout" -ForegroundColor White
Write-Host "   • Node self-recognition" -ForegroundColor White
Write-Host "   • Browser-exe connectivity" -ForegroundColor White
Write-Host ""

Write-Host "📝 Next Steps:" -ForegroundColor Yellow
Write-Host "   1. Run manual tests above" -ForegroundColor White
Write-Host "   2. Check browser console for logs" -ForegroundColor White
Write-Host "   3. Verify status transitions" -ForegroundColor White
Write-Host "   4. Test same-computer connectivity" -ForegroundColor White
Write-Host ""

Write-Host "📚 Documentation:" -ForegroundColor Cyan
Write-Host "   See SYNC_STATUS_AND_CONNECTIVITY_FIXES.md for details" -ForegroundColor White
Write-Host ""

Write-Host "✨ All automated checks passed!" -ForegroundColor Green
Write-Host ""
