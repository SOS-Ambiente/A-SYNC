# Tauri Desktop Client - Comprehensive Improvements

## Current Issues Identified

### Backend (Rust)
1. ❌ VFS methods don't match what commands expect
2. ❌ Missing proper file metadata retrieval
3. ❌ Progress callbacks not properly threaded through
4. ❌ Peer list returns empty data
5. ❌ Storage stats calculation incomplete

### Frontend (Vue.js)
1. ❌ Event listeners not properly set up
2. ❌ Progress tracking not working
3. ❌ File list not refreshing after operations
4. ❌ Peer list not displaying
5. ❌ Error handling insufficient

## Implementation Plan

### Phase 1: Fix Backend Commands ✅
- [x] Fix list_files to work with VFS structure
- [x] Fix upload_file with proper progress
- [x] Fix download_file with proper parameters
- [x] Add comprehensive logging
- [x] Fix peer listing

### Phase 2: Improve Frontend Stores (IN PROGRESS)
- [ ] Add proper event listeners for progress
- [ ] Implement auto-refresh after operations
- [ ] Better error messages
- [ ] Loading states
- [ ] Retry logic

### Phase 3: Add Missing Features
- [ ] File search/filter
- [ ] Bulk operations
- [ ] Peer connection status
- [ ] Network statistics
- [ ] Settings persistence

## Technical Details

### VFS Structure
```rust
pub struct VirtualFileSystem {
    pub local_blocks: HashMap<String, DataBlock>,  // block_id -> block
    pub file_manifest: HashMap<String, Uuid>,      // path -> first_block_uuid
    pub node: Option<Arc<Node>>,
    pub persistence: Arc<PersistenceManager>,
    pub config: Arc<Config>,
}
```

### Tauri Event Flow
```
Frontend (Vue) -> invoke('command') -> Backend (Rust)
Backend -> window.emit('event') -> Frontend (listen)
```

### Progress Tracking
```rust
// Backend emits
window.emit("upload-progress", { progress: 50, ... })

// Frontend listens
listen('upload-progress', (event) => { ... })
```

## Next Steps
1. Implement event listeners in filesStore
2. Add auto-refresh mechanism
3. Improve error handling
4. Add loading indicators
5. Test all operations end-to-end
