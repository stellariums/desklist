<script setup lang="ts">
import { getCurrentWindow } from '@tauri-apps/api/window';

const appWindow = getCurrentWindow();
const emit = defineEmits<{ settings: [] }>();

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
      <button class="title-btn" @click="emit('settings')" title="设置" aria-label="设置">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/>
        </svg>
      </button>
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
  background: transparent;
  user-select: none;
  border-radius: 12px 12px 0 0;
  flex-shrink: 0;
  border-bottom: 1px solid var(--dl-border-dim);
}
.title {
  font-size: 14px;
  font-weight: 700;
  letter-spacing: 0.5px;
  color: rgba(255, 255, 255, 0.9);
}
.title-bar-buttons {
  display: flex;
  gap: 6px;
}
.title-btn {
  width: 28px;
  height: 28px;
  border: none;
  background: var(--dl-surface);
  color: rgba(255, 255, 255, 0.6);
  border-radius: 8px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  transition: all 0.2s;
}
.title-btn:hover {
  background: var(--dl-surface-strong);
  color: rgba(255, 255, 255, 0.85);
  transform: scale(1.05);
}
.close-btn:hover {
  background: rgba(239, 68, 68, 0.15);
  color: #f87171;
}
</style>
