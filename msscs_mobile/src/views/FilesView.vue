<template>
  <div class="files-view">
    <div class="header">
      <h1>Files</h1>
      <div class="header-actions">
        <button class="btn-refresh" @click="handleRefresh" :disabled="refreshing">
          <span :class="{ spinning: refreshing }">🔄</span>
        </button>
        <button class="btn-upload" @click="handleUpload">
          <span>📤</span>
        </button>
      </div>
    </div>

    <!-- Progress Bars -->
    <div v-if="activeOperations.length > 0" class="progress-container">
      <div
        v-for="op in activeOperations"
        :key="op.id"
        class="progress-item"
      >
        <div class="progress-header">
          <span class="progress-file">{{ getFileName(op.data.file) }}</span>
          <button class="btn-cancel" @click="cancelOperation(op.id)">✕</button>
        </div>
        <div class="progress-bar-container">
          <div class="progress-bar" :style="{ width: op.data.progress + '%' }">
            <span class="progress-text">{{ op.data.progress }}%</span>
          </div>
        </div>
        <div class="progress-info">
          <span>{{ op.data.current }} / {{ op.data.total }} blocks</span>
          <span v-if="op.data.speed">{{ formatSpeed(op.data.speed) }}</span>
          <span v-if="op.data.eta">ETA: {{ formatTime(op.data.eta) }}</span>
        </div>
      </div>
    </div>

    <div v-if="loading" class="loading">
      <div class="spinner"></div>
      <p>Loading files...</p>
    </div>

    <div v-else-if="files.length === 0" class="empty-state">
      <span class="empty-icon">📁</span>
      <h2>No files yet</h2>
      <p>Upload your first file to get started</p>
    </div>

    <div v-else class="file-list">
      <div
        v-for="file in files"
        :key="file.path"
        class="file-item"
        @click="handleFileClick(file)"
      >
        <div class="file-icon">{{ getFileIcon(file.extension) }}</div>
        <div class="file-info">
          <div class="file-name">{{ file.path }}</div>
          <div class="file-meta">
            <span>{{ formatSize(file.size) }}</span>
            <span v-if="file.synced" class="synced">✓ Synced</span>
          </div>
        </div>
        <button class="btn-more" @click.stop="showFileMenu(file)">⋮</button>
      </div>
    </div>

    <!-- File Menu Modal -->
    <div v-if="selectedFile" class="modal" @click="selectedFile = null">
      <div class="modal-content" @click.stop>
        <h3>{{ selectedFile.path }}</h3>
        <div class="menu-options">
          <button @click="viewFile(selectedFile)">
            <span>👁️</span> View
          </button>
          <button @click="downloadFile(selectedFile)">
            <span>💾</span> Download
          </button>
          <button @click="shareFile(selectedFile)">
            <span>📤</span> Share
          </button>
          <button @click="openWithSystem(selectedFile)">
            <span>📱</span> Open With
          </button>
          <button @click="deleteFile(selectedFile)" class="danger">
            <span>🗑️</span> Delete
          </button>
        </div>
        <button class="btn-cancel" @click="selectedFile = null">Cancel</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useFilesStore, type FileItem } from '../stores/filesStore'
import { useToast } from '../composables/useToast'

const router = useRouter()
const filesStore = useFilesStore()
const toast = useToast()

const loading = ref(false)
const refreshing = ref(false)
const files = ref<FileItem[]>([])
const selectedFile = ref<FileItem | null>(null)
const isOnline = ref(true)

const activeOperations = computed(() => {
  return Array.from(filesStore.uploadProgress.entries()).map(([id, data]) => ({
    id,
    data
  }))
})

onMounted(async () => {
  await loadFiles()
  
  // Check online status
  checkOnlineStatus()
  window.addEventListener('online', checkOnlineStatus)
  window.addEventListener('offline', checkOnlineStatus)
})

function checkOnlineStatus() {
  isOnline.value = navigator.onLine
}

async function handleRefresh() {
  if (refreshing.value) return
  
  refreshing.value = true
  try {
    await loadFiles()
  } finally {
    setTimeout(() => {
      refreshing.value = false
    }, 500)
  }
}

async function loadFiles() {
  loading.value = true
  try {
    await filesStore.loadFiles()
    files.value = filesStore.files
  } finally {
    loading.value = false
  }
}

async function handleUpload() {
  try {
    const result = await filesStore.pickAndUploadFile()
    if (result) {
      await loadFiles()
      toast.success('File uploaded successfully')
    }
  } catch (error) {
    console.error('Upload failed:', error)
    toast.error('Failed to upload file')
  }
}

function handleFileClick(file: FileItem) {
  if (canPreview(file.mime_type)) {
    viewFile(file)
  } else {
    showFileMenu(file)
  }
}

function showFileMenu(file: FileItem) {
  selectedFile.value = file
}

function viewFile(file: FileItem) {
  router.push(`/viewer/${encodeURIComponent(file.path)}`)
  selectedFile.value = null
}

async function downloadFile(file: FileItem) {
  try {
    await filesStore.downloadFile(file.path, `/sdcard/Download/${file.path}`)
    toast.success('File downloaded to Downloads')
  } catch (error) {
    toast.error('Failed to download file')
  }
  selectedFile.value = null
}

async function openWithSystem(file: FileItem) {
  try {
    await filesStore.openWithNativeApp(file.path)
    toast.info('Opening file...')
  } catch (error) {
    toast.error('Failed to open file')
  }
  selectedFile.value = null
}

async function shareFile(file: FileItem) {
  try {
    await filesStore.shareFile(file.path)
    toast.success('File ready to share')
  } catch (error) {
    toast.error('Failed to share file')
  }
  selectedFile.value = null
}

async function deleteFile(file: FileItem) {
  if (confirm(`Delete ${file.path}?`)) {
    try {
      await filesStore.deleteFile(file.path)
      await loadFiles()
      toast.success('File deleted')
    } catch (error) {
      toast.error('Failed to delete file')
    }
  }
  selectedFile.value = null
}

function canPreview(mimeType: string): boolean {
  return mimeType.startsWith('image/') || 
         mimeType.startsWith('video/') ||
         mimeType === 'text/plain'
}

function getFileIcon(extension: string): string {
  const icons: Record<string, string> = {
    jpg: '🖼️', jpeg: '🖼️', png: '🖼️', gif: '🖼️', webp: '🖼️',
    mp4: '🎥', avi: '🎥', mkv: '🎥', mov: '🎥',
    mp3: '🎵', wav: '🎵', flac: '🎵',
    pdf: '📄',
    doc: '📝', docx: '📝',
    txt: '📃', md: '📃',
    zip: '📦', rar: '📦',
  }
  return icons[extension.toLowerCase()] || '📁'
}

function formatSize(bytes: number): string {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i]
}

function formatSpeed(blocksPerSec: number): string {
  return `${blocksPerSec.toFixed(1)} blocks/s`
}

function formatTime(seconds: number): string {
  if (seconds < 60) return `${Math.round(seconds)}s`
  const mins = Math.floor(seconds / 60)
  const secs = Math.round(seconds % 60)
  return `${mins}m ${secs}s`
}

function getFileName(path: string): string {
  return path.split('/').pop() || path.split('\\').pop() || path
}

async function cancelOperation(operationId: string) {
  await filesStore.cancelOperation(operationId)
}
</script>

<style scoped>
.files-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #000;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px;
  border-bottom: 1px solid #222;
}

h1 {
  font-size: 24px;
  font-weight: 600;
}

.header-actions {
  display: flex;
  gap: 12px;
  align-items: center;
}

.btn-refresh {
  width: 40px;
  height: 40px;
  border: none;
  background: #222;
  border-radius: 50%;
  font-size: 20px;
  cursor: pointer;
  transition: all 0.15s cubic-bezier(0.4, 0, 0.2, 1);
  display: flex;
  align-items: center;
  justify-content: center;
}

.btn-refresh:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-refresh:active:not(:disabled) {
  transform: scale(0.9);
  background: #333;
}

.spinning {
  animation: spin 1s linear infinite;
}

.btn-upload {
  width: 48px;
  height: 48px;
  border: none;
  background: #00ff88;
  border-radius: 50%;
  font-size: 24px;
  cursor: pointer;
  transition: transform 0.15s cubic-bezier(0.4, 0, 0.2, 1);
  /* Performance */
  will-change: transform;
  transform: translateZ(0);
  backface-visibility: hidden;
  -webkit-tap-highlight-color: transparent;
  box-shadow: 0 4px 12px rgba(0, 255, 136, 0.3);
}

.btn-upload:active {
  transform: scale(0.9) translateZ(0);
  box-shadow: 0 2px 8px rgba(0, 255, 136, 0.4);
  transition: all 0.1s;
}

.loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 64px 16px;
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

@keyframes spin {
  to { transform: rotate(360deg); }
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 64px 16px;
  gap: 16px;
  text-align: center;
}

.empty-icon {
  font-size: 64px;
  opacity: 0.5;
}

.empty-state h2 {
  font-size: 20px;
  font-weight: 600;
}

.empty-state p {
  color: #888;
}

.file-list {
  flex: 1;
  overflow-y: auto;
}

.file-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px;
  border-bottom: 1px solid #222;
  cursor: pointer;
  transition: background 0.15s cubic-bezier(0.4, 0, 0.2, 1);
  /* Performance */
  will-change: background;
  transform: translateZ(0);
  backface-visibility: hidden;
  -webkit-tap-highlight-color: transparent;
  contain: layout style paint;
}

.file-item:active {
  background: #111;
  transform: scale(0.98) translateZ(0);
  transition: all 0.1s;
}

.file-icon {
  font-size: 32px;
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #111;
  border-radius: 12px;
}

.file-info {
  flex: 1;
  min-width: 0;
}

.file-name {
  font-size: 16px;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.file-meta {
  display: flex;
  gap: 12px;
  margin-top: 4px;
  font-size: 12px;
  color: #888;
}

.synced {
  color: #00ff88;
}

.btn-more {
  width: 32px;
  height: 32px;
  border: none;
  background: transparent;
  color: #888;
  font-size: 20px;
  cursor: pointer;
}

.modal {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.8);
  backdrop-filter: blur(10px);
  display: flex;
  align-items: flex-end;
  z-index: 1000;
  animation: fadeIn 0.2s ease-out;
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

@keyframes slideUp {
  from { 
    opacity: 0;
    transform: translateY(100%);
  }
  to { 
    opacity: 1;
    transform: translateY(0);
  }
}

.modal-content {
  width: 100%;
  background: #111;
  border-radius: 16px 16px 0 0;
  padding: 24px;
  padding-bottom: max(24px, env(safe-area-inset-bottom));
  animation: slideUp 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  /* Performance */
  will-change: transform;
  transform: translateZ(0);
  backface-visibility: hidden;
}

.modal-content h3 {
  font-size: 18px;
  margin-bottom: 16px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
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
  transition: all 0.15s cubic-bezier(0.4, 0, 0.2, 1);
  /* Performance */
  will-change: transform, background;
  transform: translateZ(0);
  backface-visibility: hidden;
  -webkit-tap-highlight-color: transparent;
}

.menu-options button:active {
  background: #333;
  transform: scale(0.97) translateZ(0);
  transition: all 0.1s;
}

.menu-options button.danger {
  color: #ff4444;
}

.menu-options .btn-cancel {
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

.progress-container {
  padding: 16px;
  background: #0a0a0a;
  border-bottom: 1px solid #222;
}

.progress-item {
  background: #111;
  border-radius: 12px;
  padding: 16px;
  margin-bottom: 12px;
}

.progress-item:last-child {
  margin-bottom: 0;
}

.progress-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.progress-file {
  font-size: 14px;
  font-weight: 500;
  color: #fff;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  flex: 1;
}

.btn-cancel {
  width: 28px;
  height: 28px;
  border: none;
  background: #ff4444;
  color: #fff;
  border-radius: 50%;
  font-size: 16px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  margin-left: 12px;
  transition: all 0.15s cubic-bezier(0.4, 0, 0.2, 1);
}

.btn-cancel:active {
  transform: scale(0.9);
  background: #cc0000;
}

.progress-bar-container {
  width: 100%;
  height: 32px;
  background: #222;
  border-radius: 8px;
  overflow: hidden;
  position: relative;
  margin-bottom: 8px;
}

.progress-bar {
  height: 100%;
  background: linear-gradient(90deg, #00ff88, #00cc6a);
  transition: width 0.3s ease;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
}

.progress-text {
  font-size: 12px;
  font-weight: 600;
  color: #000;
  position: absolute;
  left: 50%;
  transform: translateX(-50%);
  z-index: 1;
  text-shadow: 0 1px 2px rgba(255, 255, 255, 0.3);
}

.progress-info {
  display: flex;
  justify-content: space-between;
  font-size: 11px;
  color: #888;
  gap: 8px;
}

.progress-info span {
  white-space: nowrap;
}
</style>
