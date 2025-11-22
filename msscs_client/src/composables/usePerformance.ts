import { ref, onMounted, onUnmounted } from 'vue'

/**
 * Debounce function calls for better performance
 */
export function debounce<T extends (...args: any[]) => any>(
  fn: T,
  delay: number = 300
): (...args: Parameters<T>) => void {
  let timeoutId: ReturnType<typeof setTimeout> | null = null
  
  return function (this: any, ...args: Parameters<T>) {
    if (timeoutId) clearTimeout(timeoutId)
    timeoutId = setTimeout(() => fn.apply(this, args), delay)
  }
}

/**
 * Throttle function calls for better performance
 */
export function throttle<T extends (...args: any[]) => any>(
  fn: T,
  limit: number = 100
): (...args: Parameters<T>) => void {
  let inThrottle: boolean = false
  
  return function (this: any, ...args: Parameters<T>) {
    if (!inThrottle) {
      fn.apply(this, args)
      inThrottle = true
      setTimeout(() => (inThrottle = false), limit)
    }
  }
}

/**
 * Request animation frame wrapper for smooth animations
 */
export function useRAF(callback: (time: number) => void) {
  let rafId: number | null = null
  let isRunning = false

  const start = () => {
    if (isRunning) return
    isRunning = true
    
    const animate = (time: number) => {
      callback(time)
      if (isRunning) {
        rafId = requestAnimationFrame(animate)
      }
    }
    
    rafId = requestAnimationFrame(animate)
  }

  const stop = () => {
    isRunning = false
    if (rafId !== null) {
      cancelAnimationFrame(rafId)
      rafId = null
    }
  }

  onUnmounted(stop)

  return { start, stop }
}

/**
 * Intersection Observer for lazy loading
 */
export function useIntersectionObserver(
  callback: (entry: IntersectionObserverEntry) => void,
  options?: IntersectionObserverInit
) {
  const observer = ref<IntersectionObserver | null>(null)

  onMounted(() => {
    observer.value = new IntersectionObserver((entries) => {
      entries.forEach(callback)
    }, options)
  })

  onUnmounted(() => {
    observer.value?.disconnect()
  })

  const observe = (element: Element) => {
    observer.value?.observe(element)
  }

  const unobserve = (element: Element) => {
    observer.value?.unobserve(element)
  }

  return { observe, unobserve }
}

/**
 * Optimized click handler with debounce
 */
export function useOptimizedClick(
  handler: () => void | Promise<void>,
  delay: number = 300
) {
  const isProcessing = ref(false)
  
  const handleClick = async () => {
    if (isProcessing.value) return
    
    isProcessing.value = true
    try {
      await handler()
    } finally {
      setTimeout(() => {
        isProcessing.value = false
      }, delay)
    }
  }
  
  return { handleClick, isProcessing }
}

/**
 * Virtual scroll helper for large lists
 */
export function useVirtualScroll<T>(
  items: T[],
  itemHeight: number,
  containerHeight: number
) {
  const scrollTop = ref(0)
  
  const visibleStart = ref(0)
  const visibleEnd = ref(0)
  const bufferSize = 5
  
  const updateVisibleRange = throttle(() => {
    const start = Math.max(0, Math.floor(scrollTop.value / itemHeight) - bufferSize)
    const end = Math.min(
      items.length,
      Math.ceil((scrollTop.value + containerHeight) / itemHeight) + bufferSize
    )
    
    visibleStart.value = start
    visibleEnd.value = end
  }, 16) // ~60fps
  
  const handleScroll = (event: Event) => {
    const target = event.target as HTMLElement
    scrollTop.value = target.scrollTop
    updateVisibleRange()
  }
  
  const visibleItems = () => {
    return items.slice(visibleStart.value, visibleEnd.value)
  }
  
  const offsetY = () => {
    return visibleStart.value * itemHeight
  }
  
  const totalHeight = () => {
    return items.length * itemHeight
  }
  
  return {
    handleScroll,
    visibleItems,
    offsetY,
    totalHeight,
    visibleStart,
    visibleEnd
  }
}
