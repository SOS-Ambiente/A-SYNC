# P2P Initialization Timeout Fix

## Problem
The app was getting stuck on "Initializing P2P Network" with a timeout error after 15 seconds.

### Root Causes
1. **Backend timeout too aggressive**: 3-second timeout for P2P initialization
2. **Bootstrap takes time**: DHT bootstrap needs 10-30 seconds to connect to global network
3. **Blocking initialization**: Frontend waited for full bootstrap before showing UI
4. **Confusing UX**: Users didn't know bootstrap happens in background

## Solution

### 1. Non-Blocking P2P Initialization (main.rs)
- Changed from 3-second timeout to 2-second quick check
- P2P node creation happens in background thread
- Command sender returned immediately (within 2 seconds)
- Bootstrap continues in background without blocking UI
- Node is usable immediately, even while bootstrap is ongoing

### 2. Faster Frontend Initialization (InitializationOverlay.vue)
- Removed 15-second timeout on start_node
- Reduced verification checks from 8 attempts to 3 attempts
- Reduced wait time from 500ms to 300ms per attempt
- Total initialization time: ~5 seconds instead of 15+ seconds
- Clear messaging that P2P bootstrap continues in background

### 3. Better User Communication
- Success message: "Node ready! P2P network connecting in background"
- Log message: "P2P bootstrap will complete in 10-30 seconds"
- Users can start using the app immediately
- Bootstrap completes transparently in background

## Technical Details

### Before
```rust
// Blocked for 3 seconds, then failed
match tokio::time::timeout(
    std::time::Duration::from_secs(3),
    P2PNode::new(p2p_config.clone())
).await {
    // ...
}
```

### After
```rust
// Non-blocking: returns command sender within 2 seconds
tokio::spawn(async move {
    // P2P initialization happens here
    // Bootstrap continues in background
});

// Quick check (2 seconds max)
match tokio::time::timeout(
    std::time::Duration::from_secs(2), 
    init_rx.recv()
).await {
    // Returns immediately with command sender
}
```

## Benefits
1. **Fast startup**: UI ready in ~5 seconds
2. **No timeouts**: Bootstrap happens in background
3. **Better UX**: Clear communication about what's happening
4. **Functional immediately**: Users can start using app while P2P connects
5. **Resilient**: Works even if bootstrap takes longer than expected

## Testing
1. Start the app
2. Initialization should complete in ~5 seconds
3. UI should be accessible immediately
4. Check logs for "DHT bootstrap complete" message (appears after 10-30 seconds)
5. Verify peer connections appear in dashboard

## Notes
- The node is fully functional even before bootstrap completes
- Local file operations work immediately
- P2P features (global discovery) become available after bootstrap
- Bootstrap happens automatically in background
- No user action required
