<template>
  <div class="files-view">
    <div class="view-header">
      <div class="header-content">
        <h1 class="view-title gradient-text">Files</h1>
        <p class="view-subtitle">
          {{ filteredFiles.length }} items • {{ formatBytes(totalSize) }}
          <span v-if="filesStore.selectedFiles.size > 0" class="selection-info">
            • {{ filesStore.selectedFiles.size }} selected
          </span>
        </p>
      </div>
      <div class="header-actions" v-if="filesStore.selectedFiles.size > 0">
        <button class="btn-secondary" @click="filesStore.clearSelection()">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <line x1="18" y1="6" x2="6" y2="18"/>
            <line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
          <span>Clear</span>
        </button>
        <button class="btn-secondary danger" @click="handleDeleteSelected">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <polyline points="3 6 5 6 21 6"/>
            <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
          </svg>
          <span>Delete ({{ filesStore.selectedFiles.size }})</span>
        </button>
      </div>
    </div>

    <FileToolbar
      :current-path="filesStore.currentPath"
      :search-query="filesStore.searchQuery"
      :view-mode="filesStore.viewMode"
      @back="navigateBack"
      @navigate="navigateTo"
      @search="(q) => filesStore.searchQuery = q"
      @view-mode="(m) => filesStore.viewMode = m"
      @new-folder="showFolderDialog = true"
      @upload="selectAndUpload"
    />

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

      <div v-else :class="filesStore.viewMode === 'grid' ? 'files-grid' : 'files-list'">
        <FileCard
          v-for="file in filteredFiles"
          :key="file.path"
          :file="file"
          :is-selected="filesStore.selectedFiles.has(file.path)"
          @click="handleFileClick(file, $event)"
          @dblclick="handleFileDoubleClick(file)"
          @contextmenu="handleContextMenu(file, $event)"
        >
          <template #actions>
            <FileActions 
              :file="file"
              @preview="previewFile"
              @open="openFile"
              @download="downloadFile"
              @delete="deleteFile"
            />
          </template>
        </FileCard>
      </div>
    </div>

    <FolderDialog
      :visible="showFolderDialog"
      title="Create New Folder"
      @close="showFolderDialog = false"
      @submit="handleCreateFolder"
    />

    <ContextMenu
      :visible="contextMenu.visible"
      :position="{ x: contextMenu.x, y: contextMenu.y }"
      :items="contextMenuItems"
      @close="contextMenu.visible = false"
      @select="handleContextMenuAction"
    />

    <FilePreview
      :visible="!!previewData"
      :file="previewData?.file || null"
      :data="previewData?.data || null"
      :type="previewData?.type || 'unknown'"
      @close="closePreview"
      @open="openFile"
    />

    <UploadProgress
      :upload-progress="filesStore.uploadProgress"
      :download-progress="filesStore.downloadProgress"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { useFilesStore } from '../stores/filesStore'
import { useNodeStore } from '../stores/nodeStore'
import { open, save } from '@tauri-apps/api/dialog'
import type { FileItem } from '../stores/filesStore'
import FileToolbar from './FileToolbar.vue'
import FileCard from './FileCard.vue'
import FolderDialog from './FolderDialog.vue'
import ContextMenu from './ContextMenu.vue'
import LoadingState from './LoadingState.vue'
import EmptyState from './EmptyState.vue'
import FileActions from './FileActions.vue'
import FilePreview from './FilePreview.vue'
import UploadProgress from './UploadProgress.vue'
import type { MenuItem } from './ContextMenu.vue'

const filesStore = useFilesStore()
const nodeStore = useNodeStore()
const previewData = ref<{ name: string; type: string; data: string; file: FileItem } | null>(null)
const showFolderDialog = ref(false)
const contextMenu = ref({ visible: false, x: 0, y: 0, file: null as FileItem | null })
const contextMenuItems = computed<MenuItem[]>(() => [
  {
    id: 'open',
    label: 'Open',
    icon: '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6M15 3h6v6M10 14L21 3"/></svg>',
    shortcut: 'Enter'
  },
  {
    id: 'download',
    label: 'Download',
    icon: '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3"/></svg>',
    shortcut: 'Ctrl+D'
  },
  {
    id: 'rename',
    label: 'Rename',
    icon: '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>',
    shortcut: 'F2'
  },
  {
    id: 'share',
    label: 'Share',
    icon: '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 12v8a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-8"/><polyline points="16 6 12 2 8 6"/><line x1="12" y1="2" x2="12" y2="15"/></svg>'
  },
  {
    id: 'delete',
    label: 'Delete',
    icon: '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>',
    shortcut: 'Del',
    danger: true
  }
])

const filteredFiles = computed(() => {
  let files = filesStore.files.filter(f => {
    if (filesStore.currentPath === '/') return !f.parentPath || f.parentPath === '/'
    return f.parentPath === filesStore.currentPath
  })

  if (filesStore.searchQuery) {
    const query = filesStore.searchQuery.toLowerCase()
    files = files.filter(f => f.path.toLowerCase().includes(query))
  }

  return files.sort((a, b) => {
    if (a.isFolder && !b.isFolder) return -1
    if (!a.isFolder && b.isFolder) return 1
    
    const aVal = filesStore.sortBy === 'name' ? a.path : filesStore.sortBy === 'size' ? a.size : a.modifiedAt || 0
    const bVal = filesStore.sortBy === 'name' ? b.path : filesStore.sortBy === 'size' ? b.size : b.modifiedAt || 0
    
    const comparison = aVal > bVal ? 1 : -1
    return filesStore.sortOrder === 'asc' ? comparison : -comparison
  })
})

const totalSize = computed(() => {
  return filesStore.files.reduce((sum, file) => sum + file.size, 0)
})

const navigateBack = () => {
  const parts = filesStore.currentPath.split('/').filter(Boolean)
  parts.pop()
  filesStore.currentPath = '/' + parts.join('/')
}

const navigateTo = (path: string) => {
  filesStore.currentPath = path
}

const handleFileClick = (file: FileItem, event: MouseEvent) => {
  if (event.ctrlKey || event.metaKey) {
    filesStore.toggleFileSelection(file.path)
  } else if (event.shiftKey) {
    // TODO: Implement range selection
    filesStore.toggleFileSelection(file.path)
  } else {
    filesStore.clearSelection()
    filesStore.toggleFileSelection(file.path)
  }
}

const handleFileDoubleClick = (file: FileItem) => {
  if (file.isFolder) {
    filesStore.currentPath = file.path
  } else {
    openFile(file)
  }
}

const handleContextMenu = (file: FileItem, event: MouseEvent) => {
  contextMenu.value = {
    visible: true,
    x: event.clientX,
    y: event.clientY,
    file
  }
}

const handleContextMenuAction = (actionId: string) => {
  const file = contextMenu.value.file
  if (!file) return

  switch (actionId) {
    case 'open':
      openFile(file)
      break
    case 'download':
      downloadFile(file)
      break
    case 'rename':
      // TODO: Implement rename
      break
    case 'share':
      // TODO: Implement share
      break
    case 'delete':
      deleteFile(file)
      break
  }
  
  contextMenu.value.visible = false
}

const handleCreateFolder = async (name: string) => {
  try {
    const folderPath = filesStore.currentPath === '/' 
      ? `/${name}` 
      : `${filesStore.currentPath}/${name}`
    await filesStore.createFolder(folderPath)
    alert('Folder created successfully!')
  } catch (error) {
    console.error('Failed to create folder:', error)
    alert(`Failed to create folder: ${error}`)
  }
}

const handleDeleteSelected = async () => {
  if (confirm(`Delete ${filesStore.selectedFiles.size} items?`)) {
    await filesStore.deleteSelected()
  }
}

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

.files-list {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
  animation: fadeIn 0.5s ease-out 0.1s both;
}

.selection-info {
  color: var(--color-accent-primary);
  font-weight: 600;
}

.btn-secondary.danger {
  border-color: var(--color-accent-danger);
  color: var(--color-accent-danger);
}

.btn-secondary.danger:hover {
  background: rgba(255, 51, 102, 0.15);
  border-color: var(--color-accent-danger);
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
