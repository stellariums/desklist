<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { invoke, isTauri } from '@tauri-apps/api/core';
import type { DataStatus, DeskEvent, EventInput } from './types';
import { useEvents } from './composables/useEvents';
import { useTheme } from './composables/useTheme';
import { useAppSettings } from './composables/useAppSettings';
import { useLocale } from './composables/useLocale';
import TitleBar from './components/TitleBar.vue';
import EventList from './components/EventList.vue';
import EventForm from './components/EventForm.vue';
import SettingsPanel from './components/SettingsPanel.vue';
import CalendarView from './components/CalendarView.vue';
import DataLocationSetup from './components/DataLocationSetup.vue';
import BrowserWorkbench from './components/BrowserWorkbench.vue';

const desktopMode = isTauri();
const { createEvent, updateEvent } = useEvents();
if (desktopMode) {
  useTheme().init();
  useAppSettings().init();
  useLocale().init();
}

const formVisible = ref(false);
const settingsVisible = ref(false);
const editEvent = ref<DeskEvent | null>(null);
const createDefaultTime = ref<string | null>(null);
const eventListRef = ref<InstanceType<typeof EventList> | null>(null);
const calendarMode = ref(false);
const calendarViewRef = ref<InstanceType<typeof CalendarView> | null>(null);
const dataStatus = ref<DataStatus | null>(null);
const dataStatusLoading = ref(true);

onMounted(async () => {
  if (!desktopMode) return;
  try {
    dataStatus.value = await invoke<DataStatus>('get_data_status');
  } finally {
    dataStatusLoading.value = false;
  }
});

function handleDataReady(status: DataStatus) {
  dataStatus.value = status;
}

function openCreate(defaultTime: string | null = null) {
  editEvent.value = null;
  createDefaultTime.value = defaultTime;
  formVisible.value = true;
}

function openEdit(event: DeskEvent) {
  editEvent.value = event;
  createDefaultTime.value = null;
  formVisible.value = true;
}

function closeForm() {
  formVisible.value = false;
  editEvent.value = null;
  createDefaultTime.value = null;
}

function toggleCalendar() {
  calendarMode.value = !calendarMode.value;
}

async function handleSave(data: EventInput) {
  await createEvent(data);
  closeForm();
  calendarMode.value ? calendarViewRef.value?.refresh() : eventListRef.value?.refresh();
}

async function handleUpdate(id: string, data: EventInput) {
  await updateEvent(id, data);
  closeForm();
  calendarMode.value ? calendarViewRef.value?.refresh() : eventListRef.value?.refresh();
}
</script>

<template>
  <BrowserWorkbench v-if="!desktopMode" />
  <template v-else>
    <TitleBar :is-calendar="calendarMode" @settings="settingsVisible = true" @toggle-calendar="toggleCalendar" />
    <div v-if="dataStatusLoading" class="data-loading">正在读取任务数据...</div>
    <DataLocationSetup
      v-else-if="dataStatus && !dataStatus.configured"
      :status="dataStatus"
      @ready="handleDataReady"
    />
    <template v-else-if="dataStatus?.configured">
      <EventList v-if="!calendarMode" ref="eventListRef" @create="openCreate" @edit="openEdit" />
      <CalendarView v-else ref="calendarViewRef" @create="openCreate" @edit="openEdit" />
      <EventForm
        :visible="formVisible"
        :edit-event="editEvent"
        :default-time="createDefaultTime"
        @close="closeForm"
        @save="handleSave"
        @update="handleUpdate"
      />
      <SettingsPanel :visible="settingsVisible" @close="settingsVisible = false" />
    </template>
  </template>
</template>

<style scoped>
.data-loading {
  flex: 1;
  display: grid;
  place-items: center;
  color: rgba(255, 255, 255, 0.55);
  font-size: 13px;
}
</style>
