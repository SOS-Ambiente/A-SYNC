import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

// Compatibility layer for Tauri API versions
const tauri = {
  invoke: invoke,
  listen: listen
};

export interface FileItem {
  path: string
  size: number
  blocks: number
  uuid: string
  synced: boolean
  mime_type: string
  extension: string
}

export interface UploadProgress {
  file: string
  progress: number
  current: number
  total: number
  complete?: boolean
  cancelled?: boolean
  operationId?: string
  speed?: number
  eta?: number
}

// Enhanced progress interface for uploads (matching desktop)
export interface EnhancedUploadProgress {
  fileId: string;
  progress: number; // 0-100
  bytesUploaded: number;
  totalBytes: number;
  speed: number; // bytes per second
  eta: number; // estimated time remaining in seconds
  stage: 'uploading' | 'compressing' | 'encrypting' | 'replicating' | 'complete' | 'error';
  error?: string;
}

// Enhanced progress interface for downloads (matching desktop)
export interface EnhancedDownloadProgress {
  fileId: string;
  progress: number; // 0-100
  bytesDownloaded: number;
  totalBytes: number;
  speed: number; // bytes per second
  eta: number; // estimated time remaining in seconds
  stage: 'downloading' | 'decompressing' | 'verifying' | 'complete' | 'error';
  chunksFound: number;
  totalChunks: number;
  peersConnected: number;
  error?: string;
}

// Generate unique file ID
function generateFileId(): string {
  return Date.now().toString(36) + Math.random().toString(36).substr(2);
}

export const useFilesStore = defineStore('files', () => {
  const files = ref<FileItem[]>([])
  const loading = ref(false)
  const uploadProgress = ref<Map<string, UploadProgress>>(new Map())

  const loadFiles = async () => {
    loading.value = true
    try {
      const fileList = await tauri.invoke<FileItem[]>('list_files')
      files.value = fileList
    } catch (error) {
      console.error('Failed to load files:', error)
      files.value = []
    } finally {
      loading.value = false
    }
  }

  const pickAndUploadFile = async () => {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog')
      const selected = await open({
        multiple: false,
        directory: false,
      })
      
      if (selected && typeof selected === 'string') {
        return await uploadFile(selected)
      }
      return null
    } catch (error) {
      console.error('Failed to pick file:', error)
      throw error
    }
  }

  const uploadFile = async (filePath: string) => {
    const operationId = `upload-${Date.now()}-${Math.random()}`
    
    try {
      uploadProgress.value.set(operationId, {
        file: filePath,
        progress: 0,
        current: 0,
        total: 100,
        operationId
      })

      // Listen for progress events
      const unlisten = await tauri.listen<UploadProgress>('upload-progress', (event) => {
        if (event.payload.operationId === operationId) {
          uploadProgress.value.set(operationId, event.payload)
          
          if (event.payload.complete || event.payload.cancelled) {
            setTimeout(() => {
              uploadProgress.value.delete(operationId)
              unlisten()
            }, 1500)
          }
        }
      })

      const result = await tauri.invoke<{ uuid: string; blocks: number }>('upload_file', {
        filePath,
        operationId
      })
      
      await loadFiles()
      return result
    } catch (error) {
      uploadProgress.value.delete(operationId)
      throw error
    }
  }

  const cancelOperation = async (operationId: string) => {
    try {
      await tauri.invoke('cancel_operation', { operationId })
    } catch (error) {
      console.error('Failed to cancel operation:', error)
    }
  }

  const downloadFile = async (path: string, savePath: string) => {
    const operationId = `download-${Date.now()}-${Math.random()}`
    
    try {
      uploadProgress.value.set(operationId, {
        file: path,
        progress: 0,
        current: 0,
        total: 100
      })

      // Listen for progress events
      const { listen } = await import('@tauri-apps/api/event')
      const unlisten = await listen<UploadProgress>('download-progress', (event) => {
        if (event.payload.operationId === operationId) {
          uploadProgress.value.set(operationId, event.payload)
          
          if (event.payload.complete || event.payload.cancelled) {
            setTimeout(() => {
              uploadProgress.value.delete(operationId)
              unlisten()
            }, 1500)
          }
        }
      })

      await invoke('download_file', { path, savePath, operationId })
    } catch (error) {
      uploadProgress.value.delete(operationId)
      console.error('Failed to download file:', error)
      throw error
    }
  }

  const previewFile = async (path: string): Promise<string | null> => {
    try {
      const data = await invoke<string>('preview_file', { path })
      return data
    } catch (error) {
      console.error('Failed to preview file:', error)
      return null
    }
  }

  const openWithNativeApp = async (path: string) => {
    try {
      await invoke('open_with_system', { path })
    } catch (error) {
      console.error('Failed to open file:', error)
      throw error
    }
  }

  const deleteFile = async (path: string) => {
    try {
      await invoke('delete_file', { path })
      await loadFiles()
    } catch (error) {
      console.error('Failed to delete file:', error)
      throw error
    }
  }

  const shareFile = async (path: string) => {
    try {
      // Download file to temp location first
      const tempPath = `/sdcard/Download/${path.split('/').pop()}`
      await downloadFile(path, tempPath)

      // Share using system share sheet (would need native plugin)
      console.log('Share file:', tempPath)
      return tempPath
    } catch (error) {
      console.error('Failed to share file:', error)
      throw error
    }
  }

  // Enhanced upload function matching desktop client
  const uploadFileEnhanced = async (filePath: string, onProgress?: (progress: EnhancedUploadProgress) => void) => {
    try {
      const fileId = generateFileId();

      // Setup progress listener
      if (onProgress) {
        const unlisten = await tauri.listen('upload-progress', (event) => {
          if (event.payload.fileId === fileId) {
            onProgress(event.payload as EnhancedUploadProgress);
          }
        });
      }

      // Start upload with enhanced parameters
      const result = await tauri.invoke('upload_file', {
        fileId,
        fileName: filePath.split('/').pop() || filePath,
        fileSize: 0, // Will be read by backend
        filePath: filePath,
        chunkSize: 1024 * 1024, // 1MB chunks (matching backend)
        compressionLevel: 'high', // Enable Huffman compression
        replicationFactor: 3, // Replicate to 3 peers
        encrypt: true // Enable encryption
      });

      await loadFiles();
      return { fileId, ...result };
    } catch (error) {
      console.error('Upload failed:', error);
      throw new Error(`Upload failed: ${error}`);
    }
  }

  // Enhanced download function matching desktop client
  const downloadFileEnhanced = async (fileId: string, savePath: string, onProgress?: (progress: EnhancedDownloadProgress) => void) => {
    try {
      // Setup progress listener
      if (onProgress) {
        const unlisten = await tauri.listen('download-progress', (event) => {
          if (event.payload.fileId === fileId) {
            onProgress(event.payload as EnhancedDownloadProgress);
          }
        });
      }

      // Start download
      const result = await tauri.invoke('download_file', {
        fileId,
        outputPath: savePath,
        decompress: true, // Decompress using Huffman
        verifyIntegrity: true, // Check checksums
        preferLocal: true // Use local cache if available
      });

      return { fileId, ...result };
    } catch (error) {
      console.error('Download failed:', error);
      throw new Error(`Download failed: ${error}`);
    }
  }

  const getFileSize = (file: FileItem): number => {
    return file.size
  }

  const getFileCount = (): number => {
    return files.value.length
  }

  const getTotalSize = (): number => {
    return files.value.reduce((sum, file) => sum + file.size, 0)
  }

  return {
    files,
    loading,
    uploadProgress,
    loadFiles,
    uploadFile,
    uploadFileEnhanced,
    pickAndUploadFile,
    downloadFile,
    downloadFileEnhanced,
    deleteFile,
    previewFile,
    openWithNativeApp,
    cancelOperation,
    shareFile,
    getFileSize,
    getFileCount,
    getTotalSize,
  }
})
