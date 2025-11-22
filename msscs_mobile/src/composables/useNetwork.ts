import { ref, onMounted, onUnmounted } from 'vue'

export function useNetwork() {
  const isOnline = ref(navigator.onLine)
  const connectionType = ref<string>('unknown')

  const updateOnlineStatus = () => {
    isOnline.value = navigator.onLine
    
    // Get connection type if available
    const connection = (navigator as any).connection || (navigator as any).mozConnection || (navigator as any).webkitConnection
    if (connection) {
      connectionType.value = connection.effectiveType || connection.type || 'unknown'
    }
  }

  onMounted(() => {
    updateOnlineStatus()
    window.addEventListener('online', updateOnlineStatus)
    window.addEventListener('offline', updateOnlineStatus)
  })

  onUnmounted(() => {
    window.removeEventListener('online', updateOnlineStatus)
    window.removeEventListener('offline', updateOnlineStatus)
  })

  return {
    isOnline,
    connectionType,
  }
}
