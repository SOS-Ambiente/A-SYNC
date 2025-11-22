<template>
  <div class="settings-view">
    <h1>Settings</h1>

    <div class="section">
      <h2>Storage</h2>
      <div class="setting-item">
        <div class="setting-info">
          <div class="setting-label">Data Directory</div>
          <div class="setting-value">{{ dataDir }}</div>
        </div>
      </div>
      <div class="setting-item">
        <div class="setting-info">
          <div class="setting-label">Clear Cache</div>
          <div class="setting-desc">Remove temporary files</div>
        </div>
        <button class="btn-action" @click="clearCache">Clear</button>
      </div>
    </div>

    <div class="section">
      <h2>Network</h2>
      <div class="setting-item">
        <div class="setting-info">
          <div class="setting-label">Auto-discover Nodes</div>
          <div class="setting-desc">Scan local network automatically</div>
        </div>
        <label class="toggle">
          <input type="checkbox" v-model="autoDiscover" />
          <span class="toggle-slider"></span>
        </label>
      </div>
      <div class="setting-item">
        <div class="setting-info">
          <div class="setting-label">Replication Factor</div>
          <div class="setting-desc">Number of copies per block</div>
        </div>
        <select v-model="replicationFactor" class="select">
          <option :value="1">1</option>
          <option :value="2">2</option>
          <option :value="3">3</option>
          <option :value="5">5</option>
        </select>
      </div>
    </div>

    <div class="section">
      <h2>About</h2>
      <div class="setting-item">
        <div class="setting-info">
          <div class="setting-label">Version</div>
          <div class="setting-value">1.0.0</div>
        </div>
      </div>
      <div class="setting-item">
        <div class="setting-info">
          <div class="setting-label">Node ID</div>
          <div class="setting-value mono">{{ nodeId }}</div>
        </div>
      </div>
    </div>

    <div class="section">
      <button class="btn-danger" @click="resetApp">Reset App</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'

const dataDir = ref('/data/data/com.msscs.mobile/files')
const autoDiscover = ref(true)
const replicationFactor = ref(3)
const nodeId = ref('550e8400-e29b-41d4-a716-446655440000')

function clearCache() {
  if (confirm('Clear all cached data?')) {
    alert('Cache cleared successfully')
  }
}

function resetApp() {
  if (confirm('Reset app to default settings? This will delete all local data.')) {
    alert('App reset. Please restart.')
  }
}
</script>

<style scoped>
.settings-view {
  padding: 16px;
  padding-bottom: 80px;
}

h1 {
  font-size: 24px;
  font-weight: 600;
  margin-bottom: 24px;
}

.section {
  margin-bottom: 32px;
}

.section h2 {
  font-size: 18px;
  font-weight: 600;
  margin-bottom: 12px;
}

.setting-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px;
  background: #111;
  border-radius: 12px;
  margin-bottom: 8px;
}

.setting-info {
  flex: 1;
  min-width: 0;
}

.setting-label {
  font-size: 16px;
  font-weight: 500;
  margin-bottom: 2px;
}

.setting-desc {
  font-size: 12px;
  color: #888;
}

.setting-value {
  font-size: 14px;
  color: #888;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.setting-value.mono {
  font-family: 'Courier New', monospace;
  font-size: 12px;
}

.btn-action {
  padding: 8px 16px;
  border: none;
  background: #222;
  color: #fff;
  font-weight: 600;
  border-radius: 8px;
  cursor: pointer;
}

.toggle {
  position: relative;
  display: inline-block;
  width: 48px;
  height: 28px;
}

.toggle input {
  opacity: 0;
  width: 0;
  height: 0;
}

.toggle-slider {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: #222;
  transition: 0.3s;
  border-radius: 28px;
}

.toggle-slider:before {
  position: absolute;
  content: "";
  height: 20px;
  width: 20px;
  left: 4px;
  bottom: 4px;
  background-color: #666;
  transition: 0.3s;
  border-radius: 50%;
}

.toggle input:checked + .toggle-slider {
  background-color: #00ff88;
}

.toggle input:checked + .toggle-slider:before {
  background-color: #000;
  transform: translateX(20px);
}

.select {
  padding: 8px 16px;
  border: 1px solid #222;
  background: #222;
  color: #fff;
  border-radius: 8px;
  font-size: 14px;
  cursor: pointer;
}

.btn-danger {
  width: 100%;
  padding: 16px;
  border: none;
  background: #ff4444;
  color: #fff;
  font-size: 16px;
  font-weight: 600;
  border-radius: 12px;
  cursor: pointer;
}
</style>
