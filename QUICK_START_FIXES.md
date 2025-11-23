# Quick Start: Critical Fixes Implementation

This document provides ready-to-use code fixes for the most critical issues.

---

## Fix 1: Add Desktop Progress Tracking

### File: `msscs_client/src-tauri/src/main.rs`

Replace the `upload_file` command with:

```rust
#[tauri::command]
async fn upload_file(
    file_path: String,
    window: tauri::Window,  // ADD THIS
    state: State<'_, Arc<RwLock<Option<AppStateWrapper>>>>,
) -> Result<FileUploadResult, String> {
    let state_guard = state.read().await;
    let app_state = state_guard.as_ref().ok_or("Node not started")?;

    let data = std::fs::read(&file_path).map_err(|e| e.to_string())?;
    let file_name = PathBuf::from(&file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("Invalid file path")?
        .to_string();

    let total_size = data.len();
    let chunk_size = app_state.config.chunk_size;
    let total_blocks = (total_size + chunk_size - 1) / chunk_size;

    // Emit initial progress
    let _ = window.emit("upload-progress", serde_json::json!({
        "file": file_name,
        "progress": 0,
        "current": 0,
        "total": total_blocks
    }));

    let start_time = std::time::Instant::now();
    let mut vfs = app_state.vfs.write().await;
    
    // Upload with progress
    let window_clone = window.clone();
    let file_name_clone = file_name.clone();
    let uuid = vfs
        .write_file_with_progress(&PathBuf::from(&file_name), &data, |current, total| {
            let progress = (current as f64 / total as f64 * 100.0) as u32;
            let elapsed = start_time.elapsed().as_secs_f64();
            let speed = if elapsed > 0.0 { (current as f64 / elapsed) as u64 } else { 0 };
            let eta = if speed > 0 { ((total - current) as f64 / speed as f64) as u64 } else { 0 };
            
            let _ = window_clone.emit("upload-progress", serde_json::json!({
                "file": file_name_clone,
                "progress": progress,
                "current": current,
                "total": total,
                "speed": speed,
                "eta": eta
            }));
        })
        .await
        .map_err(|e| e.to_string())?;

    // Emit completion
    let _ = window.emit("upload-progress", serde_json::json!({
        "file": file_name,
        "progress": 100,
        "current": total_blocks,
        "total": total_blocks,
        "complete": true
    }));

    Ok(FileUploadResult {
        uuid: uuid.to_string(),
        blocks: total_blocks,
    })
}
```

### File: `msscs_client/src/stores/filesStore.ts`

Add progress tracking:

```typescript
export const useFilesStore = defineStore('files', () => {
  const files = ref<FileItem[]>([])
  const loading = ref(false)
  const uploadProgress = ref<Map<string, UploadProgress>>(new Map())

  const uploadFile = async (filePath: string) => {
    try {
      // Listen for progress events
      const { listen } = await import('@tauri-apps/api/event')
      const unlisten = await listen<UploadProgress>('upload-progress', (event) => {
        const data = event.payload
        uploadProgress.value.set(data.file, data)
        
        if (data.complete) {
          setTimeout(() => {
            uploadProgress.value.delete(data.file)
            unlisten()
          }, 1500)
        }
      })

      const result = await invoke<{ uuid: string; blocks: number }>('upload_file', { filePath })
      await loadFiles()
      return result
    } catch (error) {
      throw error
    }
  }

  return {
    files,
    loading,
    uploadProgress,
    loadFiles,
    uploadFile,
    // ... other methods
  }
})
```

---

## Fix 2: Return Actual File Metadata

### File: `msscs_client/src-tauri/src/main.rs`

Replace `list_files` command:

```rust
#[tauri::command]
async fn list_files(state: State<'_, Arc<RwLock<Option<AppStateWrapper>>>>) -> Result<Vec<FileInfo>, String> {
    let state_guard = state.read().await;
    let app_state = state_guard.as_ref().ok_or("Node not started")?;
    
    let vfs = app_state.vfs.read().await;
    let file_paths = vfs.list_files();
    
    let mut file_infos = Vec::new();
    
    for path in file_paths {
        // Try to get actual file info
        let path_buf = PathBuf::from(&path);
        
        // Get extension and MIME type
        let extension = path_buf
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_string();
        
        // Calculate size and blocks by reading the file
        let (size, blocks, uuid) = match vfs.read_file(&path_buf).await {
            Ok(data) => {
                let size = data.len() as u64;
                let blocks = (size + app_state.config.chunk_size as u64 - 1) / app_state.config.chunk_size as u64;
                // Get UUID from manifest
                let uuid = app_state.vfs.read().await.file_manifest
                    .get(&path)
                    .map(|u| u.to_string())
                    .unwrap_or_default();
                (size, blocks as usize, uuid)
            }
            Err(_) => (0, 0, String::new())
        };
        
        file_infos.push(FileInfo {
            path,
            size,
            blocks,
            uuid,
            synced: true,
        });
    }
    
    Ok(file_infos)
}
```

---

## Fix 3: Add Real Peer List Command

### File: `msscs_client/src-tauri/src/main.rs`

Add new command:

```rust
#[derive(Debug, Serialize, Deserialize)]
struct PeerInfo {
    id: String,
    address: String,
    status: String,
    blocks: usize,
    latency: u64,
}

#[tauri::command]
async fn list_peers(state: State<'_, Arc<RwLock<Option<AppStateWrapper>>>>) -> Result<Vec<PeerInfo>, String> {
    let state_guard = state.read().await;
    let app_state = state_guard.as_ref().ok_or("Node not started")?;
    
    let mut peer_infos = Vec::new();
    
    // Get legacy TCP peers
    let peers = app_state.node.peers.read().await;
    for peer_addr in peers.iter() {
        peer_infos.push(PeerInfo {
            id: peer_addr.clone(),
            address: peer_addr.clone(),
            status: "online".to_string(),
            blocks: 0, // Would need to query peer
            latency: 0, // Would need to ping peer
        });
    }
    
    // Get P2P peers if available
    if let Some(ref p2p_node) = app_state.p2p_node {
        let p2p = p2p_node.read().await;
        let connected_peers = p2p.get_connected_peers().await;
        
        for peer_id in connected_peers {
            peer_infos.push(PeerInfo {
                id: peer_id.to_string(),
                address: peer_id.to_string(),
                status: "online".to_string(),
                blocks: 0,
                latency: 0,
            });
        }
    }
    
    Ok(peer_infos)
}

// Don't forget to add to invoke_handler:
.invoke_handler(tauri::generate_handler![
    start_node,
    is_node_running,
    list_files,
    list_peers,  // ADD THIS
    upload_file,
    download_file,
    delete_file,
    get_metrics,
    preview_file,
    open_with_native,
])
```

### File: `msscs_client/src/components/PeersView.vue`

Replace hardcoded peers:

```typescript
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'

interface Peer {
  id: string
  address: string
  status: 'online' | 'offline'
  blocks: number
  latency: number
}

const peers = ref<Peer[]>([])
const loading = ref(false)

const loadPeers = async () => {
  loading.value = true
  try {
    const peerList = await invoke<Peer[]>('list_peers')
    peers.value = peerList
  } catch (error) {
    console.error('Failed to load peers:', error)
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  loadPeers()
  // Refresh every 10 seconds
  setInterval(loadPeers, 10000)
})
```

---

## Fix 4: Enable P2P Node in Desktop Client

The desktop client already initializes P2P node, but we need to ensure it's properly started.

### File: `msscs_client/src-tauri/src/main.rs`

The P2P node initialization is already there, just ensure it's being used:

```rust
// In start_node command, after P2P node creation:
if let Some(p2p_node) = &p2p_node {
    let p2p = p2p_node.read().await;
    tracing::info!("P2P Node Peer ID: {}", p2p.peer_id());
    
    // Start listening
    drop(p2p); // Release read lock
    let p2p_mut = p2p_node.write().await;
    // P2P node is already started in the background
}
```

---

## Fix 5: Add Error Toast Notifications

### File: `msscs_client/src/App.vue`

Add toast container:

```vue
<template>
  <div class="app">
    <!-- ... existing content ... -->
    
    <!-- Toast Notifications -->
    <div class="toast-container">
      <div v-for="toast in toasts" :key="toast.id" 
           :class="['toast', toast.type]"
           @click="removeToast(toast.id)">
        <span class="toast-icon">{{ getToastIcon(toast.type) }}</span>
        <span class="toast-message">{{ toast.message }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'

interface Toast {
  id: number
  type: 'success' | 'error' | 'warning' | 'info'
  message: string
}

const toasts = ref<Toast[]>([])
let toastId = 0

const showToast = (type: Toast['type'], message: string) => {
  const id = toastId++
  toasts.value.push({ id, type, message })
  
  setTimeout(() => {
    removeToast(id)
  }, 5000)
}

const removeToast = (id: number) => {
  const index = toasts.value.findIndex(t => t.id === id)
  if (index !== -1) {
    toasts.value.splice(index, 1)
  }
}

const getToastIcon = (type: Toast['type']) => {
  switch (type) {
    case 'success': return '✓'
    case 'error': return '✕'
    case 'warning': return '⚠'
    case 'info': return 'ℹ'
  }
}

// Expose to window for global access
;(window as any).showToast = showToast
</script>

<style scoped>
.toast-container {
  position: fixed;
  top: 80px;
  right: 20px;
  z-index: 10000;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.toast {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px 20px;
  background: rgba(0, 0, 0, 0.9);
  backdrop-filter: blur(20px);
  border-radius: 12px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  color: white;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  animation: slideIn 0.3s ease-out;
  min-width: 300px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
}

.toast.success {
  border-color: rgba(0, 255, 136, 0.3);
  background: rgba(0, 255, 136, 0.1);
}

.toast.error {
  border-color: rgba(255, 51, 102, 0.3);
  background: rgba(255, 51, 102, 0.1);
}

.toast.warning {
  border-color: rgba(255, 170, 0, 0.3);
  background: rgba(255, 170, 0, 0.1);
}

.toast.info {
  border-color: rgba(0, 204, 255, 0.3);
  background: rgba(0, 204, 255, 0.1);
}

.toast-icon {
  font-size: 18px;
  font-weight: 700;
}

@keyframes slideIn {
  from {
    opacity: 0;
    transform: translateX(100px);
  }
  to {
    opacity: 1;
    transform: translateX(0);
  }
}
</style>
```

Then use in components:

```typescript
// In any component
try {
  await filesStore.uploadFile(filePath)
  ;(window as any).showToast('success', 'File uploaded successfully!')
} catch (error) {
  ;(window as any).showToast('error', `Upload failed: ${error}`)
}
```

---

## Testing the Fixes

### 1. Test Desktop Progress
```bash
cd msscs_client
npm run tauri dev
# Upload a file and watch the progress bar
```

### 2. Test Peer List
```bash
# In the app, go to Peers view
# Should see actual connected peers instead of hardcoded ones
```

### 3. Test File Metadata
```bash
# Upload files and check if sizes are displayed correctly
# Should show actual file sizes, not "0 B"
```

---

## Next Steps

After implementing these fixes:

1. Test thoroughly on desktop
2. Apply similar fixes to mobile client
3. Implement P2P VFS integration (see FIXES_ROADMAP.md)
4. Add comprehensive error handling
5. Write integration tests

---

**Estimated Time to Implement All Fixes:** 2-3 hours  
**Impact:** Immediate improvement in user experience and functionality
