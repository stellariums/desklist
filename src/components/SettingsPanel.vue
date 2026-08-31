<script setup lang="ts">
import { ref, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useTheme } from '../composables/useTheme';
import { useAppSettings } from '../composables/useAppSettings';
import { useLocale } from '../composables/useLocale';
import type { AgentAccessStatus, DataStatus } from '../types';

const props = defineProps<{ visible: boolean }>();
const emit = defineEmits<{ close: [] }>();
const { settings, resetDefaults } = useTheme();
const appSettingsState = useAppSettings();
const { settings: appSettings, resetDefaults: resetAppDefaults } = appSettingsState;
const { t, locale, setLocale } = useLocale();
const dataPath = ref('');
const dataPathError = ref('');
const agentAccess = ref<AgentAccessStatus | null>(null);
const agentAccessError = ref('');
const tokenVisible = ref(false);
const tokenCopied = ref(false);

watch(() => props.visible, async (visible) => {
  if (!visible) return;
  dataPathError.value = '';
  agentAccessError.value = '';
  tokenCopied.value = false;
  try {
    const status = await invoke<DataStatus>('get_data_status');
    dataPath.value = status.dataDir || '';
  } catch (error) {
    dataPathError.value = String(error);
  }
  try {
    agentAccess.value = await invoke<AgentAccessStatus>('get_agent_access');
  } catch (error) {
    agentAccessError.value = String(error);
  }
});

function handleResetDefaults() {
  resetDefaults();
  resetAppDefaults();
}

async function openDataFolder() {
  dataPathError.value = '';
  try {
    await invoke('open_data_directory');
  } catch (error) {
    dataPathError.value = String(error);
  }
}

async function copyAgentToken() {
  if (!agentAccess.value) return;
  agentAccessError.value = '';
  try {
    let copied = false;
    if (navigator.clipboard) {
      try {
        await navigator.clipboard.writeText(agentAccess.value.token);
        copied = true;
      } catch {
        copied = false;
      }
    }
    if (!copied) {
      const temporary = document.createElement('textarea');
      temporary.value = agentAccess.value.token;
      temporary.style.position = 'fixed';
      temporary.style.opacity = '0';
      document.body.appendChild(temporary);
      temporary.select();
      copied = document.execCommand('copy');
      temporary.remove();
      if (!copied) throw new Error('Clipboard is unavailable');
    }
    tokenCopied.value = true;
    window.setTimeout(() => {
      tokenCopied.value = false;
    }, 1500);
  } catch (error) {
    agentAccessError.value = String(error);
  }
}

async function regenerateAgentToken() {
  if (!window.confirm(t.value.regenerateTokenConfirm)) return;
  agentAccessError.value = '';
  try {
    agentAccess.value = await invoke<AgentAccessStatus>('regenerate_agent_token');
    tokenVisible.value = true;
    tokenCopied.value = false;
  } catch (error) {
    agentAccessError.value = String(error);
  }
}
</script>

<template>
  <Transition name="slide">
    <div v-if="visible" class="form-overlay" @click.self="emit('close')">
      <div class="form-panel">
        <div class="form-header">
          <span>{{ t.settingsTitle }}</span>
          <button class="form-close" @click="emit('close')" :aria-label="t.close">
            <svg width="14" height="14" viewBox="0 0 14 14">
              <line x1="3" y1="3" x2="11" y2="11" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
              <line x1="11" y1="3" x2="3" y2="11" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
            </svg>
          </button>
        </div>
        <div class="form-body">
          <div class="form-group">
            <label>{{ t.language }}</label>
            <div class="lang-toggle">
              <button
                class="lang-btn"
                :class="{ active: locale.locale === 'zh-CN' }"
                @click="setLocale('zh-CN')"
              >中文</button>
              <button
                class="lang-btn"
                :class="{ active: locale.locale === 'en' }"
                @click="setLocale('en')"
              >English</button>
            </div>
          </div>
          <div class="form-group">
            <label>{{ t.windowOpacity }}</label>
            <div class="range-row">
              <input type="range" :value="settings.windowOpacity * 100" @input="settings.windowOpacity = Number(($event.target as HTMLInputElement).value) / 100" min="10" max="100" step="5" class="range-input" />
              <span class="range-value">{{ Math.round(settings.windowOpacity * 100) }}%</span>
            </div>
          </div>
          <div class="form-group">
            <label>{{ t.themeColor }}</label>
            <input type="color" v-model="settings.accentColor" class="color-input" />
          </div>
          <div class="form-group">
            <label>{{ t.defaultReminderLabel }}</label>
            <label class="switch-row">
              <input
                type="checkbox"
                :checked="appSettings.defaultRemindOnTime === 1"
                @change="appSettings.defaultRemindOnTime = ($event.target as HTMLInputElement).checked ? 1 : 0"
              />
              <span>{{ t.defaultReminderCheck }}</span>
            </label>
          </div>
          <div class="form-group">
            <label>{{ t.dataLocation }}</label>
            <button class="data-path-button" type="button" @click="openDataFolder">
              <span>{{ dataPath }}</span>
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M3 6.5A2.5 2.5 0 0 1 5.5 4H9l2 2h7.5A2.5 2.5 0 0 1 21 8.5v8A2.5 2.5 0 0 1 18.5 19h-13A2.5 2.5 0 0 1 3 16.5z"/>
              </svg>
            </button>
            <span v-if="dataPathError" class="data-path-error" role="alert">{{ dataPathError }}</span>
          </div>
          <div class="form-group">
            <label>{{ t.agentAccess }}</label>
            <span class="agent-help">{{ t.agentAccessDescription }}</span>
            <span class="agent-field-label">{{ t.agentEndpoint }}</span>
            <input
              class="agent-value"
              :value="agentAccess?.endpoint || ''"
              readonly
              aria-readonly="true"
            />
            <span class="agent-field-label">{{ t.agentMcpEndpoint }}</span>
            <input
              class="agent-value"
              :value="agentAccess?.mcpEndpoint || ''"
              readonly
              aria-readonly="true"
            />
            <span class="agent-field-label">{{ t.agentToken }}</span>
            <input
              class="agent-value agent-token"
              :type="tokenVisible ? 'text' : 'password'"
              :value="agentAccess?.token || ''"
              readonly
              aria-readonly="true"
            />
            <div class="agent-actions">
              <button class="agent-button" type="button" @click="tokenVisible = !tokenVisible">
                {{ tokenVisible ? t.hideToken : t.showToken }}
              </button>
              <button class="agent-button" type="button" :disabled="!agentAccess" @click="copyAgentToken">
                {{ tokenCopied ? t.copiedToken : t.copyToken }}
              </button>
              <button class="agent-button agent-button-danger" type="button" @click="regenerateAgentToken">
                {{ t.regenerateToken }}
              </button>
            </div>
            <span v-if="agentAccessError" class="data-path-error" role="alert">{{ agentAccessError }}</span>
          </div>
        </div>
        <div class="form-footer">
          <button class="btn btn-reset" @click="handleResetDefaults">{{ t.resetDefaults }}</button>
          <button class="btn btn-done" @click="emit('close')">{{ t.done }}</button>
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
  overflow-y: auto;
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
.switch-row {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: rgba(255, 255, 255, 0.75);
  cursor: pointer;
}
.switch-row input[type="checkbox"] {
  width: 16px;
  height: 16px;
  accent-color: var(--dl-accent);
  cursor: pointer;
}
.lang-toggle {
  display: flex;
  gap: 6px;
}
.lang-btn {
  flex: 1;
  padding: 7px 0;
  border: 1px solid var(--dl-border-subtle);
  border-radius: 8px;
  background: var(--dl-surface);
  color: rgba(255, 255, 255, 0.6);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}
.lang-btn:hover {
  background: var(--dl-surface-stronger);
  color: rgba(255, 255, 255, 0.9);
}
.lang-btn.active {
  background: var(--dl-accent-gradient);
  border-color: transparent;
  color: white;
  box-shadow: 0 2px 8px var(--dl-accent-shadow);
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
.data-path-button {
  min-height: 38px;
  padding: 8px 10px;
  border: 1px solid var(--dl-border-subtle);
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  background: var(--dl-surface);
  color: rgba(255, 255, 255, 0.78);
  cursor: pointer;
}
.data-path-button span {
  overflow: hidden;
  font-size: 12px;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.data-path-button:hover {
  background: var(--dl-surface-stronger);
}
.data-path-error {
  color: #fca5a5;
  font-size: 11px;
  line-height: 1.4;
}
.agent-help {
  color: rgba(255, 255, 255, 0.5);
  font-size: 11px;
  line-height: 1.45;
}
.agent-field-label {
  margin-top: 2px;
  color: rgba(255, 255, 255, 0.52);
  font-size: 11px;
}
.agent-value {
  width: 100%;
  min-height: 34px;
  box-sizing: border-box;
  padding: 7px 9px;
  border: 1px solid var(--dl-border-subtle);
  border-radius: 8px;
  outline: none;
  background: var(--dl-surface);
  color: rgba(255, 255, 255, 0.78);
  font-family: ui-monospace, SFMono-Regular, Consolas, monospace;
  font-size: 10px;
}
.agent-value:focus {
  border-color: var(--dl-accent);
}
.agent-token {
  letter-spacing: 0.2px;
}
.agent-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}
.agent-button {
  padding: 6px 8px;
  border: 1px solid var(--dl-border-subtle);
  border-radius: 7px;
  background: var(--dl-surface);
  color: rgba(255, 255, 255, 0.72);
  cursor: pointer;
  font-size: 11px;
}
.agent-button:hover:not(:disabled) {
  background: var(--dl-surface-stronger);
  color: rgba(255, 255, 255, 0.92);
}
.agent-button:disabled {
  cursor: not-allowed;
  opacity: 0.45;
}
.agent-button-danger {
  border-color: rgba(248, 113, 113, 0.35);
  color: #fca5a5;
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
