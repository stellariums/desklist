<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import type { DeskEvent, FilterTab } from '../types';
import { useEvents } from '../composables/useEvents';
import EventCard from './EventCard.vue';

const { events, loading, fetchEvents, toggleComplete, deleteEvent } = useEvents();

const activeTab = ref<FilterTab>('today');

const tabs: { key: FilterTab; label: string }[] = [
  { key: 'today', label: '今天' },
  { key: 'upcoming', label: '即将' },
  { key: 'completed', label: '已完成' },
  { key: 'all', label: '全部' },
];

const emit = defineEmits<{
  create: [];
  edit: [event: DeskEvent];
}>();

watch(activeTab, () => {
  fetchEvents(activeTab.value);
});

onMounted(() => {
  fetchEvents(activeTab.value);
});

async function handleToggle(id: string) {
  await toggleComplete(id);
  await fetchEvents(activeTab.value);
}

async function handleDelete(id: string) {
  await deleteEvent(id);
  await fetchEvents(activeTab.value);
}

defineExpose({ refresh: () => fetchEvents(activeTab.value) });
</script>

<template>
  <div class="event-list">
    <div class="tab-bar">
      <button
        v-for="tab in tabs"
        :key="tab.key"
        class="tab-btn"
        :class="{ active: activeTab === tab.key }"
        @click="activeTab = tab.key"
      >
        {{ tab.label }}
      </button>
    </div>

    <div class="list-content">
      <div v-if="loading" class="empty-state">加载中...</div>
      <div v-else-if="events.length === 0" class="empty-state">
        <div class="empty-icon">
          <svg width="40" height="40" viewBox="0 0 40 40" fill="none">
            <rect x="4" y="8" width="32" height="28" rx="4" stroke="#CBD5E1" stroke-width="2"/>
            <line x1="4" y1="16" x2="36" y2="16" stroke="#CBD5E1" stroke-width="2"/>
            <circle cx="12" cy="12" r="2" fill="#CBD5E1"/>
            <circle cx="20" cy="12" r="2" fill="#CBD5E1"/>
          </svg>
        </div>
        <span>暂无事件</span>
      </div>
      <div v-else class="card-list">
        <EventCard
          v-for="event in events"
          :key="event.id"
          :event="event"
          @toggle="handleToggle"
          @edit="emit('edit', $event)"
          @delete="handleDelete"
        />
      </div>
    </div>

    <button class="fab" @click="emit('create')" aria-label="新建事件">
      <svg width="20" height="20" viewBox="0 0 20 20">
        <line x1="10" y1="4" x2="10" y2="16" stroke="white" stroke-width="2.5" stroke-linecap="round"/>
        <line x1="4" y1="10" x2="16" y2="10" stroke="white" stroke-width="2.5" stroke-linecap="round"/>
      </svg>
    </button>
  </div>
</template>

<style scoped>
.event-list {
  flex: 1;
  display: flex;
  flex-direction: column;
  position: relative;
  min-height: 0;
}
.tab-bar {
  display: flex;
  gap: 2px;
  padding: 8px 12px;
  background: #F1F5F9;
  flex-shrink: 0;
}
.tab-btn {
  flex: 1;
  padding: 6px 0;
  border: none;
  background: transparent;
  color: #64748B;
  font-size: 12px;
  font-weight: 500;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.15s;
}
.tab-btn.active {
  background: white;
  color: #4F46E5;
  box-shadow: 0 1px 3px rgba(0,0,0,0.08);
}
.tab-btn:hover:not(.active) {
  color: #334155;
}
.list-content {
  flex: 1;
  overflow-y: auto;
  padding: 8px 12px 80px;
}
.card-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 200px;
  color: #94A3B8;
  font-size: 13px;
  gap: 12px;
}
.empty-icon {
  opacity: 0.5;
}
.fab {
  position: absolute;
  bottom: 16px;
  right: 16px;
  width: 44px;
  height: 44px;
  border-radius: 14px;
  border: none;
  background: #4F46E5;
  color: white;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 4px 12px rgba(79, 70, 229, 0.4);
  transition: transform 0.15s, box-shadow 0.15s;
  padding: 0;
}
.fab:hover {
  transform: scale(1.05);
  box-shadow: 0 6px 16px rgba(79, 70, 229, 0.5);
}
.fab:active {
  transform: scale(0.95);
}
</style>
