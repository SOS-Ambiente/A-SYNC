import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'

export interface FileItem {
  path: string
  size: number
  blocks: number
  uuid: string
  synced: boolean
  extension?: string
  mimeType?: string
}

export const useFilesStore = defineStore('files', () => {
  const files = ref<FileItem[]>([])
  const loading = ref(false)
  const uploadProgress = ref<Map<string, number>>(new Map())

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
      uploadProgress.value.set(filePath, 0)
      const result = await invoke<{ uuid: string; blocks: number }>('upload_file', { filePath })
      uploadProgress.value.delete(filePath)
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
    loadFiles,
    uploadFile,
    downloadFile,
    deleteFile,
    previewFile,
    openWithNativeApp,
  }
})
