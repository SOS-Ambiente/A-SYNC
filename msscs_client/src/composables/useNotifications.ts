import { ref } from 'vue'

export interface Notification {
  id: number
  type: 'success' | 'error' | 'warning' | 'info'
  title: string
  message: string
  duration?: number
}

const notifications = ref<Notification[]>([])
let nextId = 1

export function useNotifications() {
  const addNotification = (notification: Omit<Notification, 'id'>) => {
    const id = nextId++
    const newNotification: Notification = {
      id,
      duration: 5000,
      ...notification
    }
    
    notifications.value.push(newNotification)
    
    if (newNotification.duration) {
      setTimeout(() => {
        removeNotification(id)
      }, newNotification.duration)
    }
    
    return id
  }
  
  const removeNotification = (id: number) => {
    const index = notifications.value.findIndex(n => n.id === id)
    if (index !== -1) {
      notifications.value.splice(index, 1)
    }
  }
  
  const success = (title: string, message: string) => {
    return addNotification({ type: 'success', title, message })
  }
  
  const error = (title: string, message: string) => {
    return addNotification({ type: 'error', title, message, duration: 7000 })
  }
  
  const warning = (title: string, message: string) => {
    return addNotification({ type: 'warning', title, message })
  }
  
  const info = (title: string, message: string) => {
    return addNotification({ type: 'info', title, message })
  }
  
  return {
    notifications,
    addNotification,
    removeNotification,
    success,
    error,
    warning,
    info
  }
}
