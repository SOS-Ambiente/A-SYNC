# Quick Fixes - Immediate Implementation

## 1. Fix TypeScript Warnings (5 minutes)

### filesStore.ts
```typescript
// Remove unused generateFileId function (line 59)
// Remove unused onProgress parameters (lines 131, 153)

const uploadFile = async (filePath: string) => {
  // Remove onProgress parameter
  try {
    console.log('📤 Starting upload:', filePath)
    const result = await tauri.invoke<{ uuid: string; blocks: number }>('upload_file', {
      path: filePath
    })
    console.log('✅ Upload complete:', result)
    await loadFiles()
    return result
  } catch (error) {
    console.error('❌ Upload failed:', error)
    const errorMsg = error instanceof Error ? error.message : String(error)
    throw new Error(`Upload failed: ${errorMsg}`)
  }
}

const downloadFile = async (path: string, savePath: string) => {
  // Remove onProgress parameter
  try {
    console.log('📥 Starting download:', path, '→', savePath)
    await tauri.invoke('download_file', {
      path,
      outputPath: savePath
    })
    console.log('✅ Download complete')
  } catch (error) {
    console.error('❌ Download failed:', error)
    const errorMsg = error instanceof Error ? error.message : String(error)
    throw new Error(`Download failed: ${errorMsg}`)
  }
}
```

## 2. Add Input Validation (15 minutes)

### Create src-tauri/src/validation.rs
```rust
use uuid::Uuid;

pub fn validate_file_path(path: &str) -> Result<(), String> {
    // Prevent path traversal
    if path.contains("..") {
        return Err("Path traversal not allowed".to_string());
    }
    
    // Check for null bytes
    if path.contains('\0') {
        return Err("Invalid path: null byte detected".to_string());
    }
    
    // Validate path length
    if path.len() > 4096 {
        return Err("Path too long (max 4096 characters)".to_string());
    }
    
    // Check for empty path
    if path.trim().is_empty() {
        return Err("Path cannot be empty".to_string());
    }
    
    Ok(())
}

pub fn validate_uuid(uuid_str: &str) -> Result<Uuid, String> {
    Uuid::parse_str(uuid_str)
        .map_err(|_| "Invalid UUID format".to_string())
}

pub fn validate_storage_limit(limit_mb: u64) -> Result<(), String> {
    if limit_mb < 100 {
        return Err("Storage limit must be at least 100 MB".to_string());
    }
    
    if limit_mb > 1_000_000 {
        return Err("Storage limit cannot exceed 1 TB".to_string());
    }
    
    Ok(())
}
```

### Update main.rs
```rust
mod validation;
use validation::*;

#[tauri::command]
async fn upload_file(
    path: String,
    window: tauri::Window,
    state: State<'_, Arc<RwLock<Option<AppStateWrapper>>>>,
) -> Result<FileUploadResult, String> {
    // Validate input
    validate_file_path(&path)?;
    
    tracing::info!("📤 upload_file command called: {}", path);
    
    // ... rest of implementation
}

#[tauri::command]
async fn download_file(
    path: String,
    output_path: String,
    window: tauri::Window,
    state: State<'_, Arc<RwLock<Option<AppStateWrapper>>>>,
) -> Result<(), String> {
    // Validate inputs
    validate_file_path(&path)?;
    validate_file_path(&output_path)?;
    
    tracing::info!("📥 download_file command called: {} -> {}", path, output_path);
    
    // ... rest of implementation
}

#[tauri::command]
async fn set_storage_limit(limit_mb: u64) -> Result<(), String> {
    // Validate input
    validate_storage_limit(limit_mb)?;
    
    let app_data_dir = get_app_data_dir();
    std::fs::create_dir_all(&app_data_dir).map_err(|e| e.to_string())?;
    
    let config_path = app_data_dir.join("storage_limit.txt");
    std::fs::write(&config_path, limit_mb.to_string()).map_err(|e| e.to_string())?;
    
    tracing::info!("📊 Storage limit set to {} MB", limit_mb);
    Ok(())
}
```

## 3. Fix Map Reactivity in Vue (10 minutes)

### Update FilesView.vue
```vue
<script setup lang="ts">
// Convert Map to Array for better reactivity
const uploadProgressArray = computed(() => 
  Array.from(filesStore.uploadProgress.entries()).map(([path, data]) => ({ 
    path, 
    ...data 
  }))
)

const downloadProgressArray = computed(() => 
  Array.from(filesStore.downloadProgress.entries()).map(([path, data]) => ({ 
    path, 
    ...data 
  }))
)
</script>

<template>
  <!-- Replace Map iteration with Array -->
  <div v-if="uploadProgressArray.length > 0 || downloadProgressArray.length > 0" class="upload-overlay">
    <div class="upload-card">
      <h3>File Operations</h3>
      
      <!-- Upload Progress -->
      <div v-for="item in uploadProgressArray" :key="'up-' + item.path" class="upload-item">
        <div class="upload-header">
          <span class="upload-icon">📤</span>
          <span class="upload-name">{{ getFileName(item.path) }}</span>
        </div>
        <div class="upload-bar">
          <div class="upload-fill" :style="{ width: item.progress + '%' }"></div>
        </div>
        <div class="upload-info">
          <span class="upload-percent">{{ item.progress }}%</span>
          <span v-if="item.speed" class="upload-speed">{{ formatSpeed(item.speed) }}</span>
          <span v-if="item.eta" class="upload-eta">{{ formatTime(item.eta) }}</span>
        </div>
      </div>
      
      <!-- Download Progress -->
      <div v-for="item in downloadProgressArray" :key="'down-' + item.path" class="upload-item">
        <div class="upload-header">
          <span class="upload-icon">📥</span>
          <span class="upload-name">{{ getFileName(item.path) }}</span>
        </div>
        <div class="upload-bar">
          <div class="upload-fill download" :style="{ width: item.progress + '%' }"></div>
        </div>
        <div class="upload-info">
          <span class="upload-percent">{{ item.progress }}%</span>
          <span v-if="item.speed" class="upload-speed">{{ formatSpeed(item.speed) }}</span>
          <span v-if="item.eta" class="upload-eta">{{ formatTime(item.eta) }}</span>
        </div>
      </div>
    </div>
  </div>
</template>
```

## 4. Add CSP to tauri.conf.json (5 minutes)

```json
{
  "tauri": {
    "security": {
      "csp": {
        "default-src": "'self'",
        "script-src": "'self' 'wasm-unsafe-eval'",
        "style-src": "'self' 'unsafe-inline'",
        "img-src": "'self' data: https:",
        "connect-src": "'self' ws://localhost:* wss://* https://*",
        "font-src": "'self' data:",
        "media-src": "'self' data:",
        "object-src": "'none'",
        "base-uri": "'self'",
        "form-action": "'self'",
        "frame-ancestors": "'none'"
      }
    }
  }
}
```

## 5. Add Error Boundary (10 minutes)

### Create src/components/ErrorBoundary.vue
```vue
<template>
  <div v-if="error" class="error-boundary">
    <div class="error-card">
      <div class="error-icon">⚠️</div>
      <h2>Something went wrong</h2>
      <p class="error-message">{{ error.message }}</p>
      <button class="btn-primary" @click="reset">Try Again</button>
    </div>
  </div>
  <slot v-else />
</template>

<script setup lang="ts">
import { ref, onErrorCaptured } from 'vue'

const error = ref<Error | null>(null)

onErrorCaptured((err) => {
  error.value = err
  console.error('Error caught by boundary:', err)
  return false
})

const reset = () => {
  error.value = null
}
</script>

<style scoped>
.error-boundary {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100vh;
  background: var(--color-bg-primary);
}

.error-card {
  text-align: center;
  padding: var(--spacing-xl);
  max-width: 500px;
}

.error-icon {
  font-size: 64px;
  margin-bottom: var(--spacing-lg);
}

.error-message {
  color: var(--color-text-secondary);
  margin: var(--spacing-lg) 0;
  font-family: monospace;
  background: rgba(255, 51, 102, 0.1);
  padding: var(--spacing-md);
  border-radius: var(--radius-md);
}
</style>
```

### Update App.vue
```vue
<template>
  <ErrorBoundary>
    <div class="app">
      <!-- existing content -->
    </div>
  </ErrorBoundary>
</template>

<script setup lang="ts">
import ErrorBoundary from './components/ErrorBoundary.vue'
// ... rest of imports
</script>
```

## Total Time: ~45 minutes

These quick fixes address the most critical issues:
- ✅ TypeScript warnings removed
- 🔒 Input validation added
- ⚡ Vue reactivity fixed
- 🔒 CSP security added
- 🐛 Error boundary added

Run these commands after implementing:
```bash
# Check for TypeScript errors
cd msscs_client
pnpm run type-check

# Check for Rust errors
cd src-tauri
cargo check

# Build and test
pnpm tauri build
```
