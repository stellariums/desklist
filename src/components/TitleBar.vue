<script setup lang="ts">
import { getCurrentWindow } from '@tauri-apps/api/window';

const appWindow = getCurrentWindow();

async function minimize() {
  await appWindow.hide();
}

async function close() {
  await appWindow.hide();
}
</script>

<template>
  <div class="title-bar" data-tauri-drag-region>
    <span class="title" data-tauri-drag-region>Desklist</span>
    <div class="title-bar-buttons">
      <button class="title-btn" @click="minimize" title="最小化到托盘" aria-label="最小化到托盘">
        <svg width="12" height="12" viewBox="0 0 12 12"><rect y="5" width="12" height="2" fill="currentColor"/></svg>
      </button>
      <button class="title-btn close-btn" @click="close" title="关闭到托盘" aria-label="关闭到托盘">
        <svg width="12" height="12" viewBox="0 0 12 12">
          <line x1="1" y1="1" x2="11" y2="11" stroke="currentColor" stroke-width="2"/>
          <line x1="11" y1="1" x2="1" y2="11" stroke="currentColor" stroke-width="2"/>
        </svg>
      </button>
    </div>
  </div>
</template>

<style scoped>
.title-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 40px;
  padding: 0 16px;
  background: rgba(255, 255, 255, 0.95);
  color: #1e293b;
  user-select: none;
  border-radius: 16px 16px 0 0;
  flex-shrink: 0;
  backdrop-filter: blur(10px);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
}
.title {
  font-size: 14px;
  font-weight: 700;
  letter-spacing: 0.5px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}
.title-bar-buttons {
  display: flex;
  gap: 6px;
}
.title-btn {
  width: 28px;
  height: 28px;
  border: none;
  background: rgba(100, 116, 139, 0.1);
  color: #64748b;
  border-radius: 8px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  transition: all 0.2s;
}
.title-btn:hover {
  background: rgba(100, 116, 139, 0.15);
  color: #334155;
  transform: scale(1.05);
}
.close-btn:hover {
  background: rgba(239, 68, 68, 0.1);
  color: #EF4444;
}
</style>
