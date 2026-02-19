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
  background: transparent;
  color: #1e293b;
  user-select: none;
  border-radius: 12px 12px 0 0;
  flex-shrink: 0;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
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
  background: rgba(255, 255, 255, 0.08);
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
  background: rgba(255, 255, 255, 0.15);
  color: rgba(255, 255, 255, 0.85);
  transform: scale(1.05);
}
.close-btn:hover {
  background: rgba(239, 68, 68, 0.15);
  color: #f87171;
}
</style>
