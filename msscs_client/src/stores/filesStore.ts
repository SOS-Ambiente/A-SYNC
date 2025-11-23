import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { listen } from '@tauri-apps/api/event'

// Compatibility layer for Tauri API
const tauri = {
  invoke,
  listen
};

export interface FileItem {
  path: string
  size: number
  blocks: number
  uuid: string
  synced: boolean
  extension?: string
  mimeType?: string
  isFolder?: boolean
  parentPath?: string
  modifiedAt?: number
  createdAt?: number
  sharedWith?: string[]
  permissions?: 'read' | 'write' | 'admin'
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



export const useFilesStore = defineStore('files', () => {
  const files = ref<FileItem[]>([])
  const loading = ref(false)
  const uploadProgress = ref<Map<string, ProgressData>>(new Map())
  const downloadProgress = ref<Map<string, ProgressData>>(new Map())
  const currentPath = ref<string>('/')
  const viewMode = ref<'grid' | 'list'>('grid')
  const sortBy = ref<'name' | 'size' | 'date'>('name')
  const sortOrder = ref<'asc' | 'desc'>('asc')
  const selectedFiles = ref<Set<string>>(new Set())
  const searchQuery = ref<string>('')

  // Setup event listeners on store creation
  const setupEventListeners = () => {
    // Listen for upload progress events
    tauri.listen<ProgressData>('upload-progress', (event) => {
      const data = event.payload
      console.log('📤 Upload progress:', data)
      uploadProgress.value.set(data.file, data)

      if (data.complete) {
        console.log('✅ Upload complete:', data.file)
        setTimeout(() => {
          uploadProgress.value.delete(data.file)
          // Auto-refresh file list after upload
          loadFiles()
        }, 2000)
      }
    })

    // Listen for download progress events
    tauri.listen<ProgressData>('download-progress', (event) => {
      const data = event.payload
      console.log('📥 Download progress:', data)
      downloadProgress.value.set(data.file, data)

      if (data.complete) {
        console.log('✅ Download complete:', data.file)
        setTimeout(() => {
          downloadProgress.value.delete(data.file)
        }, 2000)
      }
    })
    
    console.log('✅ Event listeners setup complete')
  }
  
  // Initialize listeners
  setupEventListeners()

  const loadFiles = async () => {
    loading.value = true
    try {
      console.log('📋 Loading files...')
      const fileList = await tauri.invoke<FileItem[]>('list_files')
      files.value = fileList
      console.log(`✅ Loaded ${fileList.length} files`)
    } catch (error) {
      console.error('❌ Failed to load files:', error)
      const errorMsg = error instanceof Error ? error.message : String(error)
      
      // Show user-friendly error
      if (errorMsg.includes('Node not started')) {
        console.warn('⚠️  Node not ready yet, will retry...')
        // Retry after a delay
        setTimeout(loadFiles, 2000)
      } else {
        files.value = []
      }
    } finally {
      loading.value = false
    }
  }

  const uploadFile = async (filePath: string) => {
    try {
      console.log('📤 Starting upload:', filePath)
      
      // Start upload - backend will emit progress events
      const result = await tauri.invoke('upload_file', {
        path: filePath
      });

      console.log('✅ Upload complete:', result)
      
      // Reload files to show the new file
      await loadFiles();
      
      return result;
    } catch (error) {
      console.error('❌ Upload failed:', error);
      const errorMsg = error instanceof Error ? error.message : String(error)
      throw new Error(`Upload failed: ${errorMsg}`);
    }
  }

  const downloadFile = async (path: string, savePath: string) => {
    try {
      console.log('📥 Starting download:', path, '→', savePath)
      
      // Start download - backend will emit progress events
      const result = await tauri.invoke('download_file', {
        path,
        outputPath: savePath
      });

      console.log('✅ Download complete:', result)
      return result;
    } catch (error) {
      console.error('❌ Download failed:', error);
      const errorMsg = error instanceof Error ? error.message : String(error)
      throw new Error(`Download failed: ${errorMsg}`);
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
      console.log('🗑️  Deleting file:', path)
      await tauri.invoke('delete_file', { path })
      console.log('✅ File deleted:', path)
      
      // Reload files to update the list
      await loadFiles()
    } catch (error) {
      console.error('❌ Failed to delete file:', error)
      const errorMsg = error instanceof Error ? error.message : String(error)
      throw new Error(`Delete failed: ${errorMsg}`)
    }
  }

  const createFolder = async (folderPath: string) => {
    try {
      console.log('📁 Creating folder:', folderPath)
      await tauri.invoke('create_folder', { path: folderPath })
      console.log('✅ Folder created:', folderPath)
      await loadFiles()
    } catch (error) {
      console.error('❌ Failed to create folder:', error)
      throw error
    }
  }

  const moveFile = async (sourcePath: string, destPath: string) => {
    try {
      console.log('🔄 Moving file:', sourcePath, '→', destPath)
      await tauri.invoke('move_file', { sourcePath, destPath })
      console.log('✅ File moved')
      await loadFiles()
    } catch (error) {
      console.error('❌ Failed to move file:', error)
      throw error
    }
  }

  const renameFile = async (oldPath: string, newPath: string) => {
    try {
      console.log('✏️  Renaming file:', oldPath, '→', newPath)
      await tauri.invoke('rename_file', { oldPath, newPath })
      console.log('✅ File renamed')
      await loadFiles()
    } catch (error) {
      console.error('❌ Failed to rename file:', error)
      throw error
    }
  }

  const shareFile = async (filePath: string, userIds: string[], permission: string) => {
    try {
      console.log('🤝 Sharing file:', filePath)
      await tauri.invoke('share_file', { path: filePath, userIds, permission })
      console.log('✅ File shared')
      await loadFiles()
    } catch (error) {
      console.error('❌ Failed to share file:', error)
      throw error
    }
  }

  const toggleFileSelection = (path: string) => {
    if (selectedFiles.value.has(path)) {
      selectedFiles.value.delete(path)
    } else {
      selectedFiles.value.add(path)
    }
  }

  const clearSelection = () => {
    selectedFiles.value.clear()
  }

  const selectAll = () => {
    files.value.forEach(file => selectedFiles.value.add(file.path))
  }

  const deleteSelected = async () => {
    const paths = Array.from(selectedFiles.value)
    for (const path of paths) {
      await deleteFile(path)
    }
    clearSelection()
  }

  return {
    files,
    loading,
    uploadProgress,
    downloadProgress,
    currentPath,
    viewMode,
    sortBy,
    sortOrder,
    selectedFiles,
    searchQuery,
    loadFiles,
    uploadFile,
    downloadFile,
    deleteFile,
    previewFile,
    openWithNativeApp,
    createFolder,
    moveFile,
    renameFile,
    shareFile,
    toggleFileSelection,
    clearSelection,
    selectAll,
    deleteSelected,
  }
})
