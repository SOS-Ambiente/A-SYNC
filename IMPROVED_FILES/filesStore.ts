import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

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
  status?: 'starting' | 'uploading' | 'downloading' | 'complete' | 'error'
}

export const useFilesStore = defineStore('files', () => {
  const files = ref<FileItem[]>([])
  const loading = ref(false)
  const uploadProgress = ref<Map<string, ProgressData>>(new Map())
  const downloadProgress = ref<Map<string, ProgressData>>(new Map())
  
  // Track event listeners for cleanup
  const unlistenFns = ref<UnlistenFn[]>([])

  // Setup event listeners with proper cleanup
  const setupEventListeners = async () => {
    try {
      // Upload progress listener
      const unlistenUpload = await listen<ProgressData>('upload-progress', (event) => {
        const data = event.payload
        console.log('📤 Upload progress:', data)
        uploadProgress.value.set(data.file, data)

        if (data.complete) {
          console.log('✅ Upload complete:', data.file)
          setTimeout(() => {
            uploadProgress.value.delete(data.file)
            loadFiles()
          }, 2000)
        }
      })
      
      // Download progress listener
      const unlistenDownload = await listen<ProgressData>('download-progress', (event) => {
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
      
      unlistenFns.value.push(unlistenUpload, unlistenDownload)
      console.log('✅ Event listeners setup complete')
    } catch (error) {
      console.error('❌ Failed to setup event listeners:', error)
    }
  }
  
  // Cleanup function
  const cleanup = () => {
    unlistenFns.value.forEach(unlisten => unlisten())
    unlistenFns.value = []
  }
  
  // Initialize listeners
  setupEventListeners()

  const loadFiles = async () => {
    loading.value = true
    try {
      console.log('📋 Loading files...')
      const fileList = await invoke<FileItem[]>('list_files')
      files.value = fileList
      console.log(`✅ Loaded ${fileList.length} files`)
    } catch (error) {
      console.error('❌ Failed to load files:', error)
      const errorMsg = error instanceof Error ? error.message : String(error)
      
      if (errorMsg.includes('Node not started')) {
        console.warn('⚠️  Node not ready yet, will retry...')
        setTimeout(loadFiles, 2000)
      } else {
        files.value = []
        throw error
      }
    } finally {
      loading.value = false
    }
  }

  const uploadFile = async (filePath: string) => {
    try {
      console.log('📤 Starting upload:', filePath)
      
      const result = await invoke<{ uuid: string; blocks: number }>('upload_file', {
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
    try {
      console.log('📥 Starting download:', path, '→', savePath)
      
      await invoke('download_file', {
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
      await invoke('open_with_native', { path })
    } catch (error) {
      console.error('Failed to open file:', error)
      throw error
    }
  }

  const deleteFile = async (path: string) => {
    try {
      console.log('🗑️  Deleting file:', path)
      await invoke('delete_file', { path })
      console.log('✅ File deleted:', path)
      await loadFiles()
    } catch (error) {
      console.error('❌ Failed to delete file:', error)
      const errorMsg = error instanceof Error ? error.message : String(error)
      throw new Error(`Delete failed: ${errorMsg}`)
    }
  }
  
  // Computed properties
  const totalSize = computed(() => 
    files.value.reduce((sum, file) => sum + file.size, 0)
  )
  
  const fileCount = computed(() => files.value.length)

  return {
    // State
    files,
    loading,
    uploadProgress,
    downloadProgress,
    
    // Computed
    totalSize,
    fileCount,
    
    // Actions
    loadFiles,
    uploadFile,
    downloadFile,
    deleteFile,
    previewFile,
    openWithNativeApp,
    cleanup,
  }
})
