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
  background: rgba(255, 255, 255, 0.98);
}
.tab-bar {
  display: flex;
  gap: 4px;
  padding: 12px 16px;
  background: rgba(248, 250, 252, 0.8);
  flex-shrink: 0;
  backdrop-filter: blur(10px);
}
.tab-btn {
  flex: 1;
  padding: 8px 0;
  border: none;
  background: transparent;
  color: #64748B;
  font-size: 13px;
  font-weight: 500;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
}
.tab-btn.active {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  box-shadow: 0 2px 8px rgba(102, 126, 234, 0.3);
  transform: translateY(-1px);
}
.tab-btn:hover:not(.active) {
  color: #334155;
  background: rgba(100, 116, 139, 0.05);
}
.list-content {
  flex: 1;
  overflow-y: auto;
  padding: 12px 16px 80px;
  background: rgba(255, 255, 255, 0.5);
}
.card-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
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
  bottom: 20px;
  right: 20px;
  width: 52px;
  height: 52px;
  border-radius: 16px;
  border: none;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 6px 20px rgba(102, 126, 234, 0.4), 0 2px 8px rgba(0, 0, 0, 0.1);
  transition: all 0.2s;
  padding: 0;
}
.fab:hover {
  transform: scale(1.08) translateY(-2px);
  box-shadow: 0 8px 24px rgba(102, 126, 234, 0.5), 0 4px 12px rgba(0, 0, 0, 0.15);
}
.fab:active {
  transform: scale(0.98);
}
</style>
