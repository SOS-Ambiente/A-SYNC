# Node Sync Status Fix - Event-Based Approach

## Problem
The node status was stuck on "Syncing" and never transitioned to "Online" in the Tauri Windows app.

## Root Cause
The issue was in the `nodeStore.ts` initialization flow:

1. **Race Condition**: The frontend was polling for node readiness, but the backend initialization is async
2. **Timing Issues**: Metrics polling started before the node was fully initialized
3. **No Synchronization**: Frontend had no reliable way to know when backend was truly ready

## Solution Applied - Event-Based Architecture

### Backend Changes (`msscs_client/src-tauri/src/main.rs`)

1. **New Command `wait_for_node_ready`**: 
   - Polls backend state until node is fully initialized
   - Emits `node-ready` event to frontend when ready
   - Has 30-second timeout to prevent infinite waiting
   - Runs asynchronously without blocking

2. **Event Emission**: Backend emits `node-ready` event when initialization completes

### Frontend Changes (`msscs_client/src/stores/nodeStore.ts`)

1. **Event Listener**: Listens for `node-ready` event from backend
2. **Non-Blocking Start**: Calls `start_node` without waiting for completion
3. **Async Wait**: Calls `wait_for_node_ready` which resolves when backend is ready
4. **Automatic Transition**: Status changes to 'online' when event is received
5. **Cleanup**: Properly unlistens from events after initialization

## Benefits of This Approach

- **No Race Conditions**: Backend explicitly signals when ready
- **No Polling Overhead**: Single wait command instead of repeated metric checks
- **Reliable**: Event-driven architecture ensures proper synchronization
- **Clean**: Follows Tauri best practices for async initialization
- **Timeout Protection**: Won't hang forever if initialization fails

## Testing
To verify the fix works:

1. Build and run the Tauri app: `cd msscs_client && pnpm tauri dev`
2. Watch the console for initialization logs:
   - Should see "🚀 Initializing MSSCS Tauri node..."
   - Should see "✅ Received node-ready event from backend"
   - Status should transition from "syncing" to "online" immediately
3. Check the Dashboard - Status should show "online" not "syncing"

## Files Modified
- `msscs_client/src/stores/nodeStore.ts` - Event-based initialization
- `msscs_client/src-tauri/src/main.rs` - Added wait_for_node_ready command and event emission
