import { defineStore } from 'pinia'
import { ref } from 'vue'
// Compatibility layer for Tauri API versions
const tauri = {
  invoke: (command: string, args?: any) => {
    // Desktop client using v1.5 API
    return window.__TAURI__.invoke(command, args);
  },
  listen: (event: string, callback: (event: any) => void) => {
    return window.__TAURI__.event.listen(event, callback);
  }
};

export interface FileItem {
  path: string
  size: number
  blocks: number
  uuid: string
  synced: boolean
  extension?: string
  mimeType?: string
}

export interface ProgressData {
  file: string
  progress: number
  current: number
  total: number
  speed?: number
  eta?: number
  complete?: boolean
}

// Enhanced progress interface for uploads
export interface UploadProgress {
  fileId: string;
  progress: number; // 0-100
  bytesUploaded: number;
  totalBytes: number;
  speed: number; // bytes per second
  eta: number; // estimated time remaining in seconds
  stage: 'uploading' | 'compressing' | 'encrypting' | 'replicating' | 'complete' | 'error';
  error?: string;
}

// Enhanced progress interface for downloads
export interface DownloadProgress {
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
  const uploadProgress = ref<Map<string, ProgressData>>(new Map())
  const downloadProgress = ref<Map<string, ProgressData>>(new Map())

  // Listen for upload progress events
  tauri.listen<ProgressData>('upload-progress', (event) => {
    const data = event.payload
    uploadProgress.value.set(data.file, data)

    if (data.complete) {
      setTimeout(() => {
        uploadProgress.value.delete(data.file)
      }, 2000)
    }
  })

  // Listen for download progress events
  tauri.listen<ProgressData>('download-progress', (event) => {
    const data = event.payload
    downloadProgress.value.set(data.file, data)

    if (data.complete) {
      setTimeout(() => {
        downloadProgress.value.delete(data.file)
      }, 2000)
    }
  })

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

  const uploadFile = async (filePath: string, onProgress?: (progress: UploadProgress) => void) => {
    try {
      const fileId = generateFileId();

      // Setup progress listener
      if (onProgress) {
        const unlisten = await tauri.listen('upload-progress', (event) => {
          if (event.payload.fileId === fileId) {
            onProgress(event.payload as UploadProgress);
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

  const downloadFile = async (fileId: string, savePath: string, onProgress?: (progress: DownloadProgress) => void) => {
    try {
      // Setup progress listener
      if (onProgress) {
        const unlisten = await tauri.listen('download-progress', (event) => {
          if (event.payload.fileId === fileId) {
            onProgress(event.payload as DownloadProgress);
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

  const previewFile = async (path: string): Promise<string | null> => {
    try {
      const data = await tauri.invoke<string>('preview_file', { path })
      return data
    } catch (error) {
      console.error('Failed to preview file:', error)
      return null
    }
  }

  const openWithNativeApp = async (path: string) => {
    try {
      await tauri.invoke('open_with_native', { path })
    } catch (error) {
      console.error('Failed to open file:', error)
      throw error
    }
  }

  const deleteFile = async (path: string) => {
    try {
      await tauri.invoke('delete_file', { path })
      await loadFiles()
    } catch (error) {
      console.error('Failed to delete file:', error)
      throw error
    }
  }

  return {
    files,
    loading,
    uploadProgress,
    downloadProgress,
    loadFiles,
    uploadFile,
    downloadFile,
    deleteFile,
    previewFile,
    openWithNativeApp,
  }
})
