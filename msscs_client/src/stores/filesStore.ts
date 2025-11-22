import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { listen } from '@tauri-apps/api/event'

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

export const useFilesStore = defineStore('files', () => {
  const files = ref<FileItem[]>([])
  const loading = ref(false)
  const uploadProgress = ref<Map<string, ProgressData>>(new Map())
  const downloadProgress = ref<Map<string, ProgressData>>(new Map())

  // Listen for upload progress events
  listen<ProgressData>('upload-progress', (event) => {
    const data = event.payload
    uploadProgress.value.set(data.file, data)
    
    if (data.complete) {
      setTimeout(() => {
        uploadProgress.value.delete(data.file)
      }, 2000)
    }
  })

  // Listen for download progress events
  listen<ProgressData>('download-progress', (event) => {
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
      const fileList = await invoke<FileItem[]>('list_files')
      files.value = fileList
    } catch (error) {
      console.error('Failed to load files:', error)
      files.value = []
    } finally {
      loading.value = false
    }
  }

  const uploadFile = async (filePath: string) => {
    try {
      const result = await invoke<{ uuid: string; blocks: number }>('upload_file', { filePath })
      await loadFiles()
      return result
    } catch (error) {
      uploadProgress.value.delete(filePath)
      throw error
    }
  }

  const downloadFile = async (path: string, savePath: string) => {
    try {
      await invoke('download_file', { path, savePath })
    } catch (error) {
      console.error('Failed to download file:', error)
      downloadProgress.value.delete(path)
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
      await invoke('open_with_native', { path })
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
