import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

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

export const useFilesStore = defineStore('files', () => {
  const files = ref<FileItem[]>([])
  const loading = ref(false)
  const uploadProgress = ref<Map<string, UploadProgress>>(new Map())

  const loadFiles = async () => {
    loading.value = true
    try {
      const fileList = await invoke<FileItem[]>('list_files')
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
      const { listen } = await import('@tauri-apps/api/event')
      const unlisten = await listen<UploadProgress>('upload-progress', (event) => {
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

      const result = await invoke<{ uuid: string; blocks: number }>('upload_file', { 
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
      await invoke('cancel_operation', { operationId })
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
    pickAndUploadFile,
    downloadFile,
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
