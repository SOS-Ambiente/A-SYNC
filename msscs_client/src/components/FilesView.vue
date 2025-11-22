<template>
  <div class="files-view">
    <div class="view-header">
      <div class="header-content">
        <h1 class="view-title gradient-text">Files</h1>
        <p class="view-subtitle">{{ filesStore.files.length }} files • {{ formatBytes(totalSize) }}</p>
      </div>
      <div class="header-actions">
        <button class="btn-primary" @click="selectAndUpload">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M17 8l-5-5-5 5M12 3v12"/>
          </svg>
          <span>Upload</span>
        </button>
        <button class="btn-secondary" @click="filesStore.loadFiles()">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <path d="M21.5 2v6h-6M2.5 22v-6h6M2 11.5a10 10 0 0 1 18.8-4.3M22 12.5a10 10 0 0 1-18.8 4.2"/>
          </svg>
          <span>Refresh</span>
        </button>
      </div>
    </div>

    <div class="files-container">
      <div v-if="filesStore.loading" class="loading-state">
        <div class="spinner-container">
          <div class="spinner"></div>
          <div class="spinner-glow"></div>
        </div>
        <p class="loading-text">Loading files...</p>
      </div>

      <div v-else-if="filesStore.files.length === 0" class="empty-state">
        <div class="empty-icon">
          <svg width="80" height="80" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z"/>
            <polyline points="13 2 13 9 20 9"/>
          </svg>
        </div>
        <h3 class="empty-title">No files yet</h3>
        <p class="empty-subtitle">Upload your first file to get started</p>
        <button class="btn-primary" @click="selectAndUpload">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M17 8l-5-5-5 5M12 3v12"/>
          </svg>
          <span>Upload File</span>
        </button>
      </div>

      <div v-else class="files-grid">
        <div v-for="file in filesStore.files" :key="file.path" class="file-card glass">
          <div class="file-icon-wrapper">
            <div class="file-icon" v-html="getFileIconSVG(file.path)"></div>
            <div v-if="file.synced" class="sync-badge">
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
                <polyline points="20 6 9 17 4 12"/>
              </svg>
            </div>
          </div>
          <div class="file-info">
            <div class="file-name" :title="file.path">{{ file.path }}</div>
            <div class="file-meta">
              <span class="file-size">{{ formatBytes(file.size) }}</span>
              <span class="meta-dot">•</span>
              <span class="file-blocks">{{ file.blocks }} blocks</span>
              <span class="file-type-badge">{{ getFileExtension(file.path) }}</span>
            </div>
          </div>
          <div class="file-actions">
            <button v-if="canPreview(file.path)" class="action-btn" @click="previewFile(file)" title="Preview">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/>
                <circle cx="12" cy="12" r="3"/>
              </svg>
            </button>
            <button class="action-btn" @click="openFile(file)" title="Open">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6M15 3h6v6M10 14L21 3"/>
              </svg>
            </button>
            <button class="action-btn" @click="downloadFile(file)" title="Download">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3"/>
              </svg>
            </button>
            <button class="action-btn danger" @click="deleteFile(file)" title="Delete">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="3 6 5 6 21 6"/>
                <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
              </svg>
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Preview Modal -->
    <div v-if="previewData" class="preview-overlay" @click="closePreview">
      <div class="preview-modal" @click.stop>
        <div class="preview-header">
          <h3>{{ previewData.name }}</h3>
          <button class="close-btn" @click="closePreview">✕</button>
        </div>
        <div class="preview-content">
          <img v-if="previewData.type === 'image'" :src="previewData.data" alt="Preview" />
          <video v-else-if="previewData.type === 'video'" :src="previewData.data" controls />
          <pre v-else-if="previewData.type === 'text'">{{ previewData.data }}</pre>
          <div v-else class="no-preview">
            <span>📄</span>
            <p>No preview available</p>
            <button class="btn-primary" @click="openFile(previewData.file)">Open with default app</button>
          </div>
        </div>
      </div>
    </div>

    <!-- Upload/Download Progress -->
    <div v-if="filesStore.uploadProgress.size > 0 || filesStore.downloadProgress.size > 0" class="upload-overlay">
      <div class="upload-card">
        <h3>File Operations</h3>
        
        <!-- Upload Progress -->
        <div v-for="[path, data] in filesStore.uploadProgress" :key="'up-' + path" class="upload-item">
          <div class="upload-header">
            <span class="upload-icon">📤</span>
            <span class="upload-name">{{ getFileName(path) }}</span>
          </div>
          <div class="upload-bar">
            <div class="upload-fill" :style="{ width: data.progress + '%' }"></div>
          </div>
          <div class="upload-info">
            <span class="upload-percent">{{ data.progress }}%</span>
            <span v-if="data.speed" class="upload-speed">{{ formatSpeed(data.speed) }}</span>
            <span v-if="data.eta" class="upload-eta">{{ formatTime(data.eta) }}</span>
          </div>
        </div>
        
        <!-- Download Progress -->
        <div v-for="[path, data] in filesStore.downloadProgress" :key="'down-' + path" class="upload-item">
          <div class="upload-header">
            <span class="upload-icon">📥</span>
            <span class="upload-name">{{ getFileName(path) }}</span>
          </div>
          <div class="upload-bar">
            <div class="upload-fill download" :style="{ width: data.progress + '%' }"></div>
          </div>
          <div class="upload-info">
            <span class="upload-percent">{{ data.progress }}%</span>
            <span v-if="data.speed" class="upload-speed">{{ formatSpeed(data.speed) }}</span>
            <span v-if="data.eta" class="upload-eta">{{ formatTime(data.eta) }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { useFilesStore } from '../stores/filesStore'
import { useNodeStore } from '../stores/nodeStore'
import { open, save } from '@tauri-apps/api/dialog'
import type { FileItem } from '../stores/filesStore'

const filesStore = useFilesStore()
const nodeStore = useNodeStore()
const previewData = ref<{ name: string; type: string; data: string; file: FileItem } | null>(null)

const totalSize = computed(() => {
  return filesStore.files.reduce((sum, file) => sum + file.size, 0)
})

const selectAndUpload = async () => {
  try {
    const selected = await open({
      multiple: false,
      title: 'Select file to upload',
    })
    
    if (selected && typeof selected === 'string') {
      await filesStore.uploadFile(selected)
      alert('File uploaded successfully!')
    }
  } catch (error) {
    console.error('Upload failed:', error)
    alert(`Upload failed: ${error}`)
  }
}

const downloadFile = async (file: FileItem) => {
  try {
    const savePath = await save({
      defaultPath: file.path,
      title: 'Save file as',
    })
    
    if (savePath) {
      await filesStore.downloadFile(file.path, savePath)
      alert('File downloaded successfully!')
    }
  } catch (error) {
    console.error('Download failed:', error)
    alert(`Download failed: ${error}`)
  }
}

const deleteFile = async (file: FileItem) => {
  if (confirm(`Delete ${file.path}?`)) {
    await filesStore.deleteFile(file.path)
  }
}

const formatBytes = (bytes: number): string => {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i]
}

const formatSpeed = (blocksPerSec: number): string => {
  return `${blocksPerSec.toFixed(1)} blocks/s`
}

const formatTime = (seconds: number): string => {
  if (seconds < 60) return `${Math.round(seconds)}s`
  const mins = Math.floor(seconds / 60)
  const secs = Math.round(seconds % 60)
  return `${mins}m ${secs}s`
}

const getFileName = (path: string): string => {
  return path.split('/').pop() || path.split('\\').pop() || path
}

const getFileExtension = (path: string): string => {
  const ext = path.split('.').pop()?.toUpperCase()
  return ext || 'FILE'
}

const getFileIconSVG = (path: string): string => {
  const ext = path.split('.').pop()?.toLowerCase()
  
  // Image files
  if (['jpg', 'jpeg', 'png', 'gif', 'webp', 'svg'].includes(ext || '')) {
    return '<svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"/><circle cx="8.5" cy="8.5" r="1.5"/><polyline points="21 15 16 10 5 21"/></svg>'
  }
  // Video files
  if (['mp4', 'avi', 'mkv', 'mov', 'webm'].includes(ext || '')) {
    return '<svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="23 7 16 12 23 17 23 7"/><rect x="1" y="5" width="15" height="14" rx="2" ry="2"/></svg>'
  }
  // Audio files
  if (['mp3', 'wav', 'flac', 'ogg'].includes(ext || '')) {
    return '<svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 18V5l12-2v13"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="16" r="3"/></svg>'
  }
  // PDF
  if (ext === 'pdf') {
    return '<svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/><polyline points="10 9 9 9 8 9"/></svg>'
  }
  // Code files
  if (['js', 'ts', 'py', 'rs', 'java', 'cpp', 'c', 'go', 'rb'].includes(ext || '')) {
    return '<svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="16 18 22 12 16 6"/><polyline points="8 6 2 12 8 18"/></svg>'
  }
  // Archive files
  if (['zip', 'rar', '7z', 'tar', 'gz'].includes(ext || '')) {
    return '<svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/></svg>'
  }
  // Default file icon
  return '<svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z"/><polyline points="13 2 13 9 20 9"/></svg>'
}

const canPreview = (path: string): boolean => {
  const ext = path.split('.').pop()?.toLowerCase()
  const previewable = ['jpg', 'jpeg', 'png', 'gif', 'webp', 'txt', 'md', 'json', 'mp4', 'webm']
  return previewable.includes(ext || '')
}

const previewFile = async (file: FileItem) => {
  try {
    const data = await filesStore.previewFile(file.path)
    if (data) {
      const ext = file.path.split('.').pop()?.toLowerCase()
      let type = 'unknown'
      
      if (['jpg', 'jpeg', 'png', 'gif', 'webp'].includes(ext || '')) {
        type = 'image'
      } else if (['mp4', 'webm'].includes(ext || '')) {
        type = 'video'
      } else if (['txt', 'md', 'json'].includes(ext || '')) {
        type = 'text'
      }
      
      previewData.value = { name: file.path, type, data, file }
    }
  } catch (error) {
    console.error('Preview failed:', error)
    alert(`Preview failed: ${error}`)
  }
}

const closePreview = () => {
  previewData.value = null
}

const openFile = async (file: FileItem) => {
  try {
    await filesStore.openWithNativeApp(file.path)
  } catch (error) {
    console.error('Open failed:', error)
    alert(`Failed to open file: ${error}`)
  }
}

// Load files when node becomes online
onMounted(() => {
  if (nodeStore.status === 'online') {
    filesStore.loadFiles()
  }
})

watch(() => nodeStore.status, (newStatus) => {
  if (newStatus === 'online') {
    filesStore.loadFiles()
  }
})
</script>

<style scoped>
.files-view {
  padding: var(--spacing-xl);
  height: 100%;
  display: flex;
  flex-direction: column;
  animation: fadeIn 0.4s ease-out;
}

.view-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: var(--spacing-xl);
  padding-bottom: var(--spacing-lg);
  border-bottom: var(--border-subtle);
}

.header-content {
  flex: 1;
}

.view-title {
  font-size: 36px;
  font-weight: 800;
  margin-bottom: var(--spacing-xs);
  letter-spacing: -0.5px;
}

.view-subtitle {
  font-size: 14px;
  color: var(--color-text-secondary);
  font-weight: 500;
}

.header-actions {
  display: flex;
  gap: var(--spacing-sm);
}

.btn-primary, .btn-secondary {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  padding: 12px 24px;
  border: none;
  border-radius: var(--radius-md);
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all var(--transition-fast);
  position: relative;
  overflow: hidden;
  /* Performance boost */
  will-change: transform;
  transform: translateZ(0);
  backface-visibility: hidden;
}

.btn-primary::before,
.btn-secondary::before {
  content: '';
  position: absolute;
  inset: 0;
  background: white;
  opacity: 0;
  transition: opacity var(--transition-fast);
  pointer-events: none;
}

.btn-primary {
  background: var(--gradient-primary);
  color: var(--color-bg-primary);
  box-shadow: var(--shadow-sm);
}

.btn-primary:hover {
  transform: translateY(-2px) translateZ(0);
  box-shadow: var(--glow-primary), var(--shadow-md);
}

.btn-primary:hover::before {
  opacity: 0.1;
}

.btn-primary:active {
  transform: translateY(0) translateZ(0);
  transition: transform 50ms;
}

.btn-secondary {
  background: rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(10px);
  color: var(--color-text-primary);
  border: var(--border-subtle);
}

.btn-secondary:hover {
  background: rgba(255, 255, 255, 0.08);
  border-color: rgba(255, 255, 255, 0.15);
  transform: translateY(-2px) translateZ(0);
}

.btn-secondary:hover::before {
  opacity: 0.05;
}

.btn-secondary:active {
  transform: translateY(0) translateZ(0);
  transition: transform 50ms;
}

.files-container {
  flex: 1;
  overflow: auto;
  padding-right: var(--spacing-sm);
}

.loading-state, .empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--color-text-tertiary);
}

.spinner-container {
  position: relative;
  width: 64px;
  height: 64px;
  margin-bottom: var(--spacing-lg);
}

.spinner {
  position: absolute;
  inset: 0;
  border: 3px solid transparent;
  border-top-color: var(--color-accent-primary);
  border-right-color: var(--color-accent-secondary);
  border-radius: 50%;
  animation: spin 1s cubic-bezier(0.68, -0.55, 0.265, 1.55) infinite;
}

.spinner-glow {
  position: absolute;
  inset: -4px;
  border: 3px solid transparent;
  border-top-color: var(--color-accent-primary);
  border-right-color: var(--color-accent-secondary);
  border-radius: 50%;
  filter: blur(8px);
  opacity: 0.5;
  animation: spin 1s cubic-bezier(0.68, -0.55, 0.265, 1.55) infinite;
}

.loading-text {
  font-size: 15px;
  font-weight: 500;
  color: var(--color-text-secondary);
}

.empty-icon {
  margin-bottom: var(--spacing-lg);
  opacity: 0.3;
  color: var(--color-text-tertiary);
}

.empty-title {
  font-size: 24px;
  font-weight: 700;
  color: var(--color-text-primary);
  margin-bottom: var(--spacing-sm);
}

.empty-subtitle {
  font-size: 15px;
  color: var(--color-text-secondary);
  margin-bottom: var(--spacing-xl);
}

.files-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(360px, 1fr));
  gap: var(--spacing-md);
  animation: fadeIn 0.5s ease-out 0.1s both;
}

.file-card {
  position: relative;
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
  padding: var(--spacing-lg);
  border-radius: var(--radius-lg);
  transition: all var(--transition-fast);
  cursor: pointer;
  overflow: hidden;
  /* Performance optimization */
  will-change: transform;
  transform: translateZ(0);
  backface-visibility: hidden;
  contain: layout style paint;
}

.file-card::before {
  content: '';
  position: absolute;
  inset: 0;
  background: var(--gradient-primary);
  opacity: 0;
  transition: opacity var(--transition-fast);
  pointer-events: none;
}

.file-card:hover {
  transform: translateY(-4px) translateZ(0);
  box-shadow: var(--shadow-lg);
}

.file-card:hover::before {
  opacity: 0.03;
}

.file-card:active {
  transform: translateY(-2px) translateZ(0);
  transition: transform 50ms;
}

.file-icon-wrapper {
  position: relative;
  flex-shrink: 0;
}

.file-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 56px;
  height: 56px;
  background: rgba(255, 255, 255, 0.03);
  border-radius: var(--radius-md);
  color: var(--color-accent-primary);
  transition: all var(--transition-base);
}

.file-card:hover .file-icon {
  background: rgba(0, 255, 136, 0.1);
  transform: scale(1.05);
}

.file-info {
  flex: 1;
  min-width: 0;
  position: relative;
  z-index: 1;
}

.file-name {
  font-size: 15px;
  font-weight: 600;
  color: var(--color-text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-bottom: 6px;
  transition: color var(--transition-fast);
}

.file-card:hover .file-name {
  color: var(--color-accent-primary);
}

.file-meta {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  font-size: 12px;
  color: var(--color-text-tertiary);
  font-weight: 500;
}

.meta-dot {
  opacity: 0.5;
}

.file-type-badge {
  padding: 2px 8px;
  background: rgba(255, 255, 255, 0.05);
  border-radius: var(--radius-sm);
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--color-accent-secondary);
}

.file-actions {
  display: flex;
  gap: var(--spacing-xs);
  position: relative;
  z-index: 1;
  opacity: 0;
  transform: translateX(-10px);
  transition: all var(--transition-base);
}

.file-card:hover .file-actions {
  opacity: 1;
  transform: translateX(0);
}

.action-btn {
  width: 36px;
  height: 36px;
  background: rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(10px);
  border: var(--border-subtle);
  border-radius: var(--radius-sm);
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all var(--transition-fast);
  display: flex;
  align-items: center;
  justify-content: center;
  /* Performance */
  will-change: transform;
  transform: translateZ(0);
  backface-visibility: hidden;
}

.action-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  border-color: var(--color-accent-primary);
  color: var(--color-accent-primary);
  transform: translateY(-2px) translateZ(0) scale(1.05);
}

.action-btn:active {
  transform: translateY(0) translateZ(0) scale(0.95);
  transition: transform 50ms;
}

.action-btn.danger:hover {
  background: rgba(255, 51, 102, 0.1);
  border-color: var(--color-accent-danger);
  color: var(--color-accent-danger);
}

.sync-badge {
  position: absolute;
  top: 12px;
  right: 12px;
  width: 24px;
  height: 24px;
  background: rgba(0, 255, 136, 0.15);
  backdrop-filter: blur(10px);
  border: 1px solid rgba(0, 255, 136, 0.3);
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-accent-primary);
  box-shadow: 0 0 12px rgba(0, 255, 136, 0.3);
  animation: fadeIn 0.3s ease-out;
}

.upload-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.8);
  backdrop-filter: blur(20px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  animation: fadeIn 0.2s ease-out;
}

.upload-card {
  background: rgba(255, 255, 255, 0.03);
  backdrop-filter: blur(40px);
  border: var(--border-medium);
  border-radius: var(--radius-xl);
  padding: var(--spacing-xl);
  min-width: 480px;
  box-shadow: var(--shadow-xl);
  animation: fadeIn 0.3s ease-out 0.1s both;
}

.upload-card h3 {
  font-size: 20px;
  font-weight: 700;
  margin-bottom: var(--spacing-lg);
  color: var(--color-text-primary);
}

.upload-item {
  margin-bottom: var(--spacing-lg);
  padding: var(--spacing-md);
  background: rgba(255, 255, 255, 0.02);
  border-radius: var(--radius-md);
}

.upload-header {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  margin-bottom: var(--spacing-sm);
}

.upload-icon {
  font-size: 18px;
}

.upload-name {
  font-size: 14px;
  font-weight: 500;
  color: var(--color-text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  flex: 1;
}

.upload-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: var(--spacing-sm);
  font-size: 12px;
  color: var(--color-text-tertiary);
}

.upload-speed, .upload-eta {
  color: var(--color-accent-primary);
}

.upload-bar {
  position: relative;
  height: 8px;
  background: rgba(255, 255, 255, 0.05);
  border-radius: var(--radius-full);
  overflow: visible;
  margin-bottom: var(--spacing-sm);
}

.upload-fill {
  height: 100%;
  background: var(--gradient-primary);
  border-radius: var(--radius-full);
  transition: width 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  box-shadow: var(--glow-primary);
}

.upload-fill.download {
  background: linear-gradient(90deg, #00ccff, #0099ff);
  box-shadow: 0 0 12px rgba(0, 204, 255, 0.4);
}

.upload-percent {
  font-size: 13px;
  font-weight: 700;
  color: var(--color-accent-primary);
  text-align: right;
}

.preview-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.95);
  backdrop-filter: blur(30px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2000;
  animation: fadeIn 0.2s ease-out;
}

.preview-modal {
  background: rgba(255, 255, 255, 0.03);
  backdrop-filter: blur(40px);
  border: var(--border-medium);
  border-radius: var(--radius-xl);
  max-width: 90vw;
  max-height: 90vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  box-shadow: var(--shadow-xl);
  animation: fadeIn 0.3s ease-out 0.1s both;
}

.preview-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--spacing-lg) var(--spacing-xl);
  border-bottom: var(--border-subtle);
  background: rgba(0, 0, 0, 0.3);
}

.preview-header h3 {
  font-size: 16px;
  font-weight: 600;
  color: var(--color-text-primary);
  margin: 0;
}

.close-btn {
  width: 40px;
  height: 40px;
  background: rgba(255, 255, 255, 0.05);
  border: var(--border-subtle);
  border-radius: var(--radius-md);
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all var(--transition-fast);
  display: flex;
  align-items: center;
  justify-content: center;
}

.close-btn:hover {
  background: rgba(255, 51, 102, 0.15);
  border-color: var(--color-accent-danger);
  color: var(--color-accent-danger);
  transform: rotate(90deg);
}

.preview-content {
  padding: var(--spacing-xl);
  overflow: auto;
  max-height: calc(90vh - 80px);
  display: flex;
  align-items: center;
  justify-content: center;
}

.preview-content img {
  max-width: 100%;
  max-height: 70vh;
  object-fit: contain;
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-lg);
}

.preview-content video {
  max-width: 100%;
  max-height: 70vh;
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-lg);
}

.preview-content pre {
  background: rgba(0, 0, 0, 0.5);
  padding: var(--spacing-lg);
  border-radius: var(--radius-lg);
  color: var(--color-accent-primary);
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', 'Monaco', monospace;
  font-size: 13px;
  line-height: 1.8;
  overflow-x: auto;
  max-height: 70vh;
  border: var(--border-subtle);
  width: 100%;
}

.no-preview {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 80px;
  color: var(--color-text-tertiary);
}

.no-preview span {
  font-size: 64px;
  margin-bottom: var(--spacing-lg);
  opacity: 0.3;
}

.no-preview p {
  margin-bottom: var(--spacing-xl);
  font-size: 16px;
  color: var(--color-text-secondary);
}
</style>
