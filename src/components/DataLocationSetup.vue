<script setup lang="ts">
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import type { DataStatus } from '../types';
import { useLocale } from '../composables/useLocale';

const props = defineProps<{ status: DataStatus }>();
const emit = defineEmits<{ ready: [status: DataStatus] }>();
const { t } = useLocale();
const selectedPath = ref('');
const working = ref(false);
const errorMessage = ref(props.status.error || '');

async function chooseFolder() {
  errorMessage.value = '';
  const selected = await open({ directory: true, multiple: false });
  if (typeof selected === 'string') {
    selectedPath.value = selected;
  }
}

async function confirmLocation() {
  if (!selectedPath.value || working.value) return;
  working.value = true;
  errorMessage.value = '';
  try {
    const status = await invoke<DataStatus>('configure_data_directory', {
      dataDir: selectedPath.value,
    });
    emit('ready', status);
  } catch (error) {
    errorMessage.value = String(error);
  } finally {
    working.value = false;
  }
}
</script>

<template>
  <main class="setup-page">
    <div class="setup-icon" aria-hidden="true">
      <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8">
        <path d="M3 6.5A2.5 2.5 0 0 1 5.5 4H9l2 2h7.5A2.5 2.5 0 0 1 21 8.5v8A2.5 2.5 0 0 1 18.5 19h-13A2.5 2.5 0 0 1 3 16.5z"/>
      </svg>
    </div>
    <h1>{{ t.dataSetupTitle }}</h1>
    <p>{{ status.legacyDatabaseFound ? t.dataSetupMigrationHint : t.dataSetupNewHint }}</p>

    <button class="choose-button" type="button" :disabled="working" @click="chooseFolder">
      {{ selectedPath || t.chooseDataFolder }}
    </button>

    <button
      class="confirm-button"
      type="button"
      :disabled="!selectedPath || working"
      @click="confirmLocation"
    >
      {{ working ? t.migratingData : t.confirmDataFolder }}
    </button>

    <p v-if="errorMessage" class="setup-error" role="alert">{{ errorMessage }}</p>
    <p class="setup-note">{{ t.dataSetupSafetyNote }}</p>
  </main>
</template>

<style scoped>
.setup-page {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 28px;
  text-align: center;
  color: rgba(255, 255, 255, 0.9);
}
.setup-icon {
  width: 56px;
  height: 56px;
  margin-bottom: 16px;
  border-radius: 16px;
  display: grid;
  place-items: center;
  color: var(--dl-accent-light);
  background: var(--dl-accent-subtle);
  border: 1px solid var(--dl-accent-border-hover);
}
h1 {
  margin-bottom: 8px;
  font-size: 19px;
}
p {
  max-width: 270px;
  color: rgba(255, 255, 255, 0.58);
  font-size: 13px;
  line-height: 1.6;
}
.choose-button,
.confirm-button {
  width: 100%;
  max-width: 280px;
  min-height: 42px;
  margin-top: 18px;
  padding: 9px 14px;
  border-radius: 10px;
  font-size: 13px;
  cursor: pointer;
}
.choose-button {
  overflow: hidden;
  color: rgba(255, 255, 255, 0.8);
  text-overflow: ellipsis;
  white-space: nowrap;
  background: var(--dl-surface);
  border: 1px solid var(--dl-border-subtle);
}
.confirm-button {
  margin-top: 10px;
  border: none;
  color: white;
  font-weight: 600;
  background: var(--dl-accent-gradient);
}
button:disabled {
  cursor: not-allowed;
  opacity: 0.5;
}
.setup-error {
  margin-top: 14px;
  color: #fca5a5;
}
.setup-note {
  margin-top: 16px;
  font-size: 11px;
}
</style>
