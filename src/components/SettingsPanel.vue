<script setup lang="ts">
import { useTheme } from '../composables/useTheme';

defineProps<{ visible: boolean }>();
const emit = defineEmits<{ close: [] }>();
const { settings, resetDefaults } = useTheme();
</script>

<template>
  <Transition name="slide">
    <div v-if="visible" class="form-overlay" @click.self="emit('close')">
      <div class="form-panel">
        <div class="form-header">
          <span>外观设置</span>
          <button class="form-close" @click="emit('close')" aria-label="关闭">
            <svg width="14" height="14" viewBox="0 0 14 14">
              <line x1="3" y1="3" x2="11" y2="11" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
              <line x1="11" y1="3" x2="3" y2="11" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
            </svg>
          </button>
        </div>
        <div class="form-body">
          <div class="form-group">
            <label>窗口透明度</label>
            <div class="range-row">
              <input type="range" :value="settings.windowOpacity * 100" @input="settings.windowOpacity = Number(($event.target as HTMLInputElement).value) / 100" min="10" max="100" step="5" class="range-input" />
              <span class="range-value">{{ Math.round(settings.windowOpacity * 100) }}%</span>
            </div>
          </div>
          <div class="form-group">
            <label>主题色</label>
            <input type="color" v-model="settings.accentColor" class="color-input" />
          </div>
        </div>
        <div class="form-footer">
          <button class="btn btn-reset" @click="resetDefaults">重置默认</button>
          <button class="btn btn-done" @click="emit('close')">完成</button>
        </div>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.form-overlay {
  position: absolute;
  inset: 0;
  background: var(--dl-overlay);
  backdrop-filter: blur(4px);
  z-index: 100;
  display: flex;
  justify-content: flex-end;
}
.form-panel {
  width: 100%;
  max-width: 320px;
  background: var(--dl-panel-bg);
  backdrop-filter: blur(20px);
  display: flex;
  flex-direction: column;
  height: 100%;
  box-shadow: -4px 0 20px rgba(0, 0, 0, 0.15);
  border: 1px solid rgba(255, 255, 255, 0.1);
}
.form-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  color: rgba(255, 255, 255, 0.9);
  font-size: 16px;
  font-weight: 600;
  border-bottom: 1px solid var(--dl-border-dim);
}
.form-close {
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
.form-close:hover {
  background: var(--dl-surface-stronger);
  color: rgba(255, 255, 255, 0.9);
}
.form-body {
  flex: 1;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}
.form-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.form-group label {
  font-size: 12px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.6);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}
.range-row {
  display: flex;
  align-items: center;
  gap: 10px;
}
.range-input {
  flex: 1;
  accent-color: var(--dl-accent);
  height: 4px;
}
.range-value {
  font-size: 13px;
  color: rgba(255, 255, 255, 0.7);
  min-width: 40px;
  text-align: right;
}
.color-input {
  width: 48px;
  height: 32px;
  border: 1px solid var(--dl-border-subtle);
  border-radius: 8px;
  background: transparent;
  cursor: pointer;
  padding: 2px;
}
.form-footer {
  display: flex;
  gap: 8px;
  padding: 12px 16px;
  border-top: 1px solid var(--dl-border-dim);
}
.btn {
  flex: 1;
  padding: 10px 0;
  border: none;
  border-radius: 10px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}
.btn-reset {
  background: var(--dl-surface-active);
  color: rgba(255, 255, 255, 0.7);
}
.btn-reset:hover {
  background: var(--dl-surface-stronger);
  color: rgba(255, 255, 255, 0.9);
}
.btn-done {
  background: var(--dl-accent-gradient);
  color: white;
  box-shadow: 0 2px 8px var(--dl-accent-shadow);
}
.btn-done:hover {
  box-shadow: 0 4px 12px var(--dl-accent-shadow-strong);
  transform: translateY(-1px);
}
.slide-enter-active, .slide-leave-active {
  transition: opacity 0.2s ease;
}
.slide-enter-active .form-panel, .slide-leave-active .form-panel {
  transition: transform 0.25s ease;
}
.slide-enter-from { opacity: 0; }
.slide-enter-from .form-panel { transform: translateX(100%); }
.slide-leave-to { opacity: 0; }
.slide-leave-to .form-panel { transform: translateX(100%); }
</style>
