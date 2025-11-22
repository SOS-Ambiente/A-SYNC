import { ref, computed } from 'vue'
import { useFilesStore } from '../stores/filesStore'

export function useFileSync() {
  const filesStore = useFilesStore()
  const syncStatus = ref<'idle' | 'syncing' | 'error' | 'success'>('idle')
  const lastSyncTime = ref<Date | null>(null)
  const syncError = ref<string | null>(null)

  const isSyncing = computed(() => syncStatus.value === 'syncing')
  
  const activeUploads = computed(() => {
    return Array.from(filesStore.uploadProgress.values())
  })

  const totalProgress = computed(() => {
    const uploads = activeUploads.value
    if (uploads.length === 0) return 0
    
    const total = uploads.reduce((sum, upload) => sum + upload.progress, 0)
    return Math.round(total / uploads.length)
  })

  const syncFiles = async () => {
    if (isSyncing.value) return
    
    syncStatus.value = 'syncing'
    syncError.value = null
    
    try {
      await filesStore.loadFiles()
      syncStatus.value = 'success'
      lastSyncTime.value = new Date()
      
      // Reset to idle after 2 seconds
      setTimeout(() => {
        if (syncStatus.value === 'success') {
          syncStatus.value = 'idle'
        }
      }, 2000)
    } catch (error) {
      syncStatus.value = 'error'
      syncError.value = error instanceof Error ? error.message : 'Sync failed'
      
      // Reset to idle after 5 seconds
      setTimeout(() => {
        if (syncStatus.value === 'error') {
          syncStatus.value = 'idle'
        }
      }, 5000)
    }
  }

  const getTimeSinceSync = (): string => {
    if (!lastSyncTime.value) return 'Never'
    
    const now = new Date()
    const diff = now.getTime() - lastSyncTime.value.getTime()
    const seconds = Math.floor(diff / 1000)
    const minutes = Math.floor(seconds / 60)
    const hours = Math.floor(minutes / 60)
    
    if (hours > 0) return `${hours}h ago`
    if (minutes > 0) return `${minutes}m ago`
    return `${seconds}s ago`
  }

  return {
    syncStatus,
    isSyncing,
    syncError,
    lastSyncTime,
    activeUploads,
    totalProgress,
    syncFiles,
    getTimeSinceSync,
  }
}
