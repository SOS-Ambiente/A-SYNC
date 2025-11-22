<template>
  <div class="file-viewer">
    <div class="viewer-header">
      <button class="btn-back" @click="goBack">←</button>
      <h1>{{ fileName }}</h1>
      <button class="btn-menu" @click="showMenu = true">⋮</button>
    </div>

    <div v-if="loading" class="loading">
      <div class="spinner"></div>
      <p>Loading file...</p>
    </div>

    <div v-else-if="error" class="error">
      <span class="error-icon">⚠️</span>
      <p>{{ error }}</p>
      <button @click="loadFile">Retry</button>
    </div>

    <div v-else class="viewer-content">
      <!-- Image Viewer -->
      <div v-if="isImage" class="image-viewer">
        <img :src="dataUrl" :alt="fileName" />
      </div>

      <!-- Video Viewer -->
      <div v-else-if="isVideo" class="video-viewer">
        <video :src="dataUrl" controls />
      </div>

      <!-- Text Viewer -->
      <div v-else-if="isText" class="text-viewer">
        <pre>{{ textContent }}</pre>
      </div>

      <!-- Unsupported -->
      <div v-else class="unsupported">
        <span class="file-icon">{{ fileIcon }}</span>
        <h2>{{ fileName }}</h2>
        <p>{{ formatSize(fileSize) }}</p>
        <button @click="openWithSystem">Open with System App</button>
      </div>
    </div>

    <!-- Menu Modal -->
    <div v-if="showMenu" class="modal" @click="showMenu = false">
      <div class="modal-content" @click.stop>
        <h3>File Options</h3>
        <div class="menu-options">
          <button @click="downloadFile">
            <span>💾</span> Download
          </button>
          <button @click="openWithSystem">
            <span>📱</span> Open With
          </button>
          <button @click="shareFile">
            <span>📤</span> Share
          </button>
        </div>
        <button class="btn-cancel" @click="showMenu = false">Cancel</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'

interface Props {
  path: string
}

const props = defineProps<Props>()
const router = useRouter()

const loading = ref(true)
const error = ref('')
const showMenu = ref(false)
const fileData = ref<string>('')
const mimeType = ref('')
const fileSize = ref(0)

const fileName = computed(() => decodeURIComponent(props.path))

const isImage = computed(() => mimeType.value.startsWith('image/'))
const isVideo = computed(() => mimeType.value.startsWith('video/'))
const isText = computed(() => mimeType.value.startsWith('text/'))

const dataUrl = computed(() => {
  if (!fileData.value) return ''
  return `data:${mimeType.value};base64,${fileData.value}`
})

const textContent = computed(() => {
  if (!fileData.value) return ''
  try {
    return atob(fileData.value)
  } catch {
    return 'Unable to decode text'
  }
})

const fileIcon = computed(() => {
  const ext = fileName.value.split('.').pop()?.toLowerCase() || ''
  const icons: Record<string, string> = {
    pdf: '📄',
    doc: '📝', docx: '📝',
    xls: '📊', xlsx: '📊',
    zip: '📦', rar: '📦',
  }
  return icons[ext] || '📁'
})

onMounted(async () => {
  await loadFile()
})

async function loadFile() {
  loading.value = true
  error.value = ''
  
  try {
    const result = await invoke<{
      mime_type: string
      data: string
      size: number
    }>('preview_file', { path: props.path })
    
    fileData.value = result.data
    mimeType.value = result.mime_type
    fileSize.value = result.size
  } catch (err) {
    error.value = 'Failed to load file'
    console.error(err)
  } finally {
    loading.value = false
  }
}

function goBack() {
  router.back()
}

async function downloadFile() {
  try {
    await invoke('download_file', {
      path: props.path,
      savePath: `/sdcard/Download/${fileName.value}`
    })
    alert('File downloaded to Downloads folder')
  } catch (err) {
    alert('Failed to download file')
  }
  showMenu.value = false
}

async function openWithSystem() {
  try {
    await invoke('open_with_system', { path: props.path })
  } catch (err) {
    alert('Failed to open file')
  }
  showMenu.value = false
}

function shareFile() {
  alert('Share functionality coming soon')
  showMenu.value = false
}

function formatSize(bytes: number): string {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i]
}
</script>

<style scoped>
.file-viewer {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #000;
}

.viewer-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px;
  border-bottom: 1px solid #222;
}

.btn-back,
.btn-menu {
  width: 40px;
  height: 40px;
  border: none;
  background: #222;
  color: #fff;
  font-size: 20px;
  border-radius: 8px;
  cursor: pointer;
}

.viewer-header h1 {
  flex: 1;
  font-size: 16px;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.loading,
.error {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  flex: 1;
  gap: 16px;
}

.spinner {
  width: 48px;
  height: 48px;
  border: 4px solid #222;
  border-top-color: #00ff88;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

.error-icon {
  font-size: 64px;
}

.error button {
  padding: 12px 24px;
  border: none;
  background: #00ff88;
  color: #000;
  font-weight: 600;
  border-radius: 8px;
  cursor: pointer;
}

.viewer-content {
  flex: 1;
  overflow: auto;
}

.image-viewer {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 100%;
  padding: 16px;
}

.image-viewer img {
  max-width: 100%;
  max-height: 100%;
  object-fit: contain;
}

.video-viewer {
  padding: 16px;
}

.video-viewer video {
  width: 100%;
  border-radius: 12px;
}

.text-viewer {
  padding: 16px;
}

.text-viewer pre {
  font-family: 'Courier New', monospace;
  font-size: 14px;
  line-height: 1.5;
  white-space: pre-wrap;
  word-wrap: break-word;
}

.unsupported {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 100%;
  padding: 32px;
  gap: 16px;
  text-align: center;
}

.file-icon {
  font-size: 64px;
}

.unsupported h2 {
  font-size: 18px;
  font-weight: 600;
}

.unsupported p {
  color: #888;
}

.unsupported button {
  padding: 12px 24px;
  border: none;
  background: #00ff88;
  color: #000;
  font-weight: 600;
  border-radius: 8px;
  cursor: pointer;
}

.modal {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.8);
  display: flex;
  align-items: flex-end;
  z-index: 1000;
}

.modal-content {
  width: 100%;
  background: #111;
  border-radius: 16px 16px 0 0;
  padding: 24px;
}

.modal-content h3 {
  font-size: 18px;
  margin-bottom: 16px;
}

.menu-options {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 16px;
}

.menu-options button {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px;
  border: none;
  background: #222;
  color: #fff;
  font-size: 16px;
  border-radius: 12px;
  cursor: pointer;
}

.btn-cancel {
  width: 100%;
  padding: 16px;
  border: none;
  background: #222;
  color: #fff;
  font-size: 16px;
  font-weight: 600;
  border-radius: 12px;
  cursor: pointer;
}
</style>
