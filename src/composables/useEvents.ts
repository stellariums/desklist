import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { DeskEvent, EventInput, FilterTab } from '../types';

export function useEvents() {
  const events = ref<DeskEvent[]>([]);
  const loading = ref(false);

  async function fetchEvents(filter: FilterTab = 'all') {
    loading.value = true;
    try {
      const todayStart = new Date();
      todayStart.setHours(0, 0, 0, 0);
      const todayEnd = new Date();
      todayEnd.setHours(23, 59, 59, 999);

      events.value = await invoke<DeskEvent[]>('fetch_events', {
        filter,
        todayStart: filter === 'today' ? todayStart.toISOString() : null,
        todayEnd: filter === 'today' ? todayEnd.toISOString() : null,
      });
    } finally {
      loading.value = false;
    }
  }

  async function createEvent(event: EventInput) {
    return await invoke<string>('create_event', { event });
  }

  async function updateEvent(id: string, event: EventInput) {
    await invoke('update_event', { id, event });
  }

  async function deleteEvent(id: string) {
    await invoke('delete_event', { id });
  }

  async function toggleComplete(id: string) {
    await invoke('toggle_complete', { id });
  }

  async function fetchMonthEvents(year: number, month: number): Promise<DeskEvent[]> {
    const monthStart = new Date(year, month, 1, 0, 0, 0, 0);
    const monthEnd = new Date(year, month + 1, 0, 23, 59, 59, 999);
    return await invoke<DeskEvent[]>('fetch_month_events', {
      monthStart: monthStart.toISOString(),
      monthEnd: monthEnd.toISOString(),
    });
  }

  return { events, loading, fetchEvents, fetchMonthEvents, createEvent, updateEvent, deleteEvent, toggleComplete };
}
