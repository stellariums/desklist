<script setup lang="ts">
import { ref } from 'vue';
import type { DeskEvent } from './types';
import { useEvents } from './composables/useEvents';
import { useTheme } from './composables/useTheme';
import { useAppSettings } from './composables/useAppSettings';
import { useLocale } from './composables/useLocale';
import TitleBar from './components/TitleBar.vue';
import EventList from './components/EventList.vue';
import EventForm from './components/EventForm.vue';
import SettingsPanel from './components/SettingsPanel.vue';
import CalendarView from './components/CalendarView.vue';

const { createEvent, updateEvent } = useEvents();
useTheme().init();
useAppSettings().init();
useLocale().init();

const formVisible = ref(false);
const settingsVisible = ref(false);
const editEvent = ref<DeskEvent | null>(null);
const eventListRef = ref<InstanceType<typeof EventList> | null>(null);
const calendarMode = ref(false);
const calendarViewRef = ref<InstanceType<typeof CalendarView> | null>(null);

function openCreate() {
  editEvent.value = null;
  formVisible.value = true;
}

function openEdit(event: DeskEvent) {
  editEvent.value = event;
  formVisible.value = true;
}

function closeForm() {
  formVisible.value = false;
  editEvent.value = null;
}

function toggleCalendar() {
  calendarMode.value = !calendarMode.value;
}

async function handleSave(data: Omit<DeskEvent, 'id' | 'created_at' | 'updated_at'>) {
  await createEvent(data);
  closeForm();
  calendarMode.value ? calendarViewRef.value?.refresh() : eventListRef.value?.refresh();
}

async function handleUpdate(id: string, data: Partial<DeskEvent>) {
  await updateEvent(id, data);
  closeForm();
  calendarMode.value ? calendarViewRef.value?.refresh() : eventListRef.value?.refresh();
}
</script>

<template>
  <TitleBar :is-calendar="calendarMode" @settings="settingsVisible = true" @toggle-calendar="toggleCalendar" />
  <EventList v-if="!calendarMode" ref="eventListRef" @create="openCreate" @edit="openEdit" />
  <CalendarView v-else ref="calendarViewRef" @create="openCreate" @edit="openEdit" />
  <EventForm
    :visible="formVisible"
    :edit-event="editEvent"
    @close="closeForm"
    @save="handleSave"
    @update="handleUpdate"
  />
  <SettingsPanel :visible="settingsVisible" @close="settingsVisible = false" />
</template>
