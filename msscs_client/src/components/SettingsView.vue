<template>
  <div class="settings-view">
    <div class="view-header">
      <div class="header-content">
        <h1 class="view-title gradient-text">Settings</h1>
        <p class="view-subtitle">Configure your MSSCS node</p>
      </div>
    </div>

    <div class="settings-container">
      <div class="settings-section glass">
        <div class="section-header">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/>
          </svg>
          <h2 class="section-title">Node Configuration</h2>
        </div>
        <div class="setting-item">
          <label>Port</label>
          <input v-model="settings.port" type="number" />
        </div>
        <div class="setting-item">
          <label>Data Directory</label>
          <input v-model="settings.dataDir" type="text" />
        </div>
        <div class="setting-item">
          <label>Replication Factor</label>
          <input v-model="settings.replicationFactor" type="number" min="1" max="10" />
        </div>
        <div class="setting-item">
          <label>Chunk Size (bytes)</label>
          <input v-model="settings.chunkSize" type="number" />
        </div>
      </div>

      <div class="settings-section glass">
        <div class="section-header">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="3" y="11" width="18" height="11" rx="2" ry="2"/>
            <path d="M7 11V7a5 5 0 0 1 10 0v4"/>
          </svg>
          <h2 class="section-title">Security</h2>
        </div>
        <div class="setting-item">
          <div class="toggle-wrapper">
            <label>Enable API Authentication</label>
            <div class="toggle" :class="{ active: settings.enableAuth }" @click="settings.enableAuth = !settings.enableAuth">
              <div class="toggle-slider"></div>
            </div>
          </div>
        </div>
        <div v-if="settings.enableAuth" class="setting-item">
          <label>API Keys</label>
          <textarea v-model="settings.apiKeys" rows="3" placeholder="One key per line"></textarea>
        </div>
      </div>

      <div class="settings-section glass">
        <div class="section-header">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="5"/>
            <line x1="12" y1="1" x2="12" y2="3"/>
            <line x1="12" y1="21" x2="12" y2="23"/>
            <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/>
            <line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/>
            <line x1="1" y1="12" x2="3" y2="12"/>
            <line x1="21" y1="12" x2="23" y2="12"/>
            <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/>
            <line x1="18.36" y1="5.64" x2="19.78" y2="4.22"/>
          </svg>
          <h2 class="section-title">Appearance</h2>
        </div>
        <div class="setting-item">
          <label>Theme</label>
          <select v-model="settings.theme">
            <option value="dark">Dark (AMOLED)</option>
            <option value="light">Light</option>
          </select>
        </div>
      </div>
    </div>

    <div class="settings-actions">
      <button class="btn-secondary" @click="resetSettings">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
          <polyline points="1 4 1 10 7 10"/>
          <path d="M3.51 15a9 9 0 1 0 2.13-9.36L1 10"/>
        </svg>
        <span>Reset to Defaults</span>
      </button>
      <button class="btn-primary" @click="saveSettings">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
          <polyline points="20 6 9 17 4 12"/>
        </svg>
        <span>Save Changes</span>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'

const settings = ref({
  port: 8080,
  dataDir: './msscs_data',
  replicationFactor: 3,
  chunkSize: 1024,
  enableAuth: false,
  apiKeys: '',
  theme: 'dark',
})

const saveSettings = () => {
  // TODO: Save settings via Tauri command
  console.log('Saving settings:', settings.value)
}

const resetSettings = () => {
  settings.value = {
    port: 8080,
    dataDir: './msscs_data',
    replicationFactor: 3,
    chunkSize: 1024,
    enableAuth: false,
    apiKeys: '',
    theme: 'dark',
  }
}
</script>

<style scoped>
.settings-view {
  padding: var(--spacing-xl);
  animation: fadeIn 0.4s ease-out;
}

.view-header {
  margin-bottom: var(--spacing-xl);
  padding-bottom: var(--spacing-lg);
  border-bottom: var(--border-subtle);
}

.header-content {
  max-width: 800px;
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

.settings-container {
  max-width: 800px;
  display: flex;
  flex-direction: column;
  gap: var(--spacing-lg);
  margin-bottom: var(--spacing-xl);
}

.settings-section {
  padding: var(--spacing-xl);
  border-radius: var(--radius-lg);
  animation: fadeIn 0.5s ease-out both;
}

.settings-section:nth-child(1) { animation-delay: 0.1s; }
.settings-section:nth-child(2) { animation-delay: 0.2s; }
.settings-section:nth-child(3) { animation-delay: 0.3s; }

.section-header {
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
  margin-bottom: var(--spacing-lg);
  padding-bottom: var(--spacing-md);
  border-bottom: var(--border-subtle);
}

.section-header svg {
  color: var(--color-accent-primary);
}

.section-title {
  font-size: 18px;
  font-weight: 700;
  color: var(--color-text-primary);
}

.setting-item {
  margin-bottom: var(--spacing-lg);
}

.setting-item:last-child {
  margin-bottom: 0;
}

.setting-item label {
  display: block;
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-secondary);
  text-transform: uppercase;
  letter-spacing: 1px;
  margin-bottom: var(--spacing-sm);
}

.setting-item input[type="text"],
.setting-item input[type="number"],
.setting-item textarea,
.setting-item select {
  width: 100%;
}

.toggle-wrapper {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.toggle {
  position: relative;
  width: 52px;
  height: 28px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: var(--radius-full);
  cursor: pointer;
  transition: all var(--transition-base);
  border: var(--border-subtle);
}

.toggle:hover {
  background: rgba(255, 255, 255, 0.15);
}

.toggle.active {
  background: var(--gradient-primary);
  border-color: transparent;
  box-shadow: var(--glow-primary);
}

.toggle-slider {
  position: absolute;
  top: 3px;
  left: 3px;
  width: 22px;
  height: 22px;
  background: white;
  border-radius: 50%;
  transition: all var(--transition-base);
  box-shadow: var(--shadow-sm);
}

.toggle.active .toggle-slider {
  transform: translateX(24px);
  background: var(--color-bg-primary);
}

.settings-actions {
  display: flex;
  gap: var(--spacing-sm);
  justify-content: flex-end;
  max-width: 800px;
  padding-top: var(--spacing-lg);
  border-top: var(--border-subtle);
}

.btn-primary, .btn-secondary {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  padding: 12px 24px;
  border-radius: var(--radius-md);
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all var(--transition-base);
}

.btn-primary {
  background: var(--gradient-primary);
  border: none;
  color: var(--color-bg-primary);
  box-shadow: var(--shadow-sm);
}

.btn-primary:hover {
  transform: translateY(-2px);
  box-shadow: var(--glow-primary), var(--shadow-md);
}

.btn-secondary {
  background: rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(10px);
  border: var(--border-subtle);
  color: var(--color-text-primary);
}

.btn-secondary:hover {
  background: rgba(255, 255, 255, 0.08);
  border-color: rgba(255, 255, 255, 0.15);
  transform: translateY(-2px);
}
</style>
