<script setup lang="ts">
import { ref } from 'vue';
import type { DeskEvent } from './types';
import { useEvents } from './composables/useEvents';
import { useTheme } from './composables/useTheme';
import TitleBar from './components/TitleBar.vue';
import EventList from './components/EventList.vue';
import EventForm from './components/EventForm.vue';
import SettingsPanel from './components/SettingsPanel.vue';

const { createEvent, updateEvent } = useEvents();
useTheme().init();

const formVisible = ref(false);
const settingsVisible = ref(false);
const editEvent = ref<DeskEvent | null>(null);
const eventListRef = ref<InstanceType<typeof EventList> | null>(null);

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

async function handleSave(data: Omit<DeskEvent, 'id' | 'created_at' | 'updated_at'>) {
  await createEvent(data);
  closeForm();
  eventListRef.value?.refresh();
}

async function handleUpdate(id: string, data: Partial<DeskEvent>) {
  await updateEvent(id, data);
  closeForm();
  eventListRef.value?.refresh();
}
</script>

<template>
  <TitleBar @settings="settingsVisible = true" />
  <EventList
    ref="eventListRef"
    @create="openCreate"
    @edit="openEdit"
  />
  <EventForm
    :visible="formVisible"
    :edit-event="editEvent"
    @close="closeForm"
    @save="handleSave"
    @update="handleUpdate"
  />
  <SettingsPanel :visible="settingsVisible" @close="settingsVisible = false" />
</template>
