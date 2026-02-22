<script setup lang="ts">
import type { DeskEvent } from '../types';

defineProps<{
  event: DeskEvent;
}>();

const emit = defineEmits<{
  toggle: [id: string];
  edit: [event: DeskEvent];
  delete: [id: string];
}>();

function formatTime(iso: string): string {
  const d = new Date(iso);
  const month = d.getMonth() + 1;
  const day = d.getDate();
  const hours = d.getHours().toString().padStart(2, '0');
  const minutes = d.getMinutes().toString().padStart(2, '0');
  return `${month}/${day} ${hours}:${minutes}`;
}

function isOverdue(event: DeskEvent): boolean {
  return !event.completed && new Date(event.event_time) < new Date();
}

function getRecurrenceLabel(r: string): string {
  const map: Record<string, string> = { daily: '每天', weekly: '每周', monthly: '每月' };
  return map[r] || '';
}
</script>

<template>
  <div class="event-card" :class="{ completed: event.completed, overdue: isOverdue(event) }">
    <div class="card-left">
      <button
        class="checkbox"
        :class="{ checked: event.completed }"
        @click="emit('toggle', event.id)"
        :aria-label="event.completed ? '标记为未完成' : '标记为完成'"
      >
        <svg v-if="event.completed" width="14" height="14" viewBox="0 0 14 14">
          <polyline points="2,7 6,11 12,3" fill="none" stroke="white" stroke-width="2" stroke-linecap="round"/>
        </svg>
      </button>
    </div>
    <div class="card-content" @click="emit('edit', event)">
      <div class="card-title">{{ event.title }}</div>
      <div class="card-meta">
        <span class="card-time">{{ formatTime(event.event_time) }}</span>
        <span v-if="event.recurrence !== 'none'" class="card-recurrence">{{ getRecurrenceLabel(event.recurrence) }}</span>
        <span v-if="event.remind_at || event.remind_on_time" class="card-reminder">bell</span>
      </div>
    </div>
    <button class="delete-btn" @click.stop="emit('delete', event.id)" aria-label="删除事件">
      <svg width="14" height="14" viewBox="0 0 14 14">
        <line x1="3" y1="3" x2="11" y2="11" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
        <line x1="11" y1="3" x2="3" y2="11" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
      </svg>
    </button>
  </div>
</template>

<style scoped>
.event-card {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 14px;
  background: var(--dl-surface);
  border-radius: 12px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.04);
  transition: all 0.2s;
  cursor: default;
  border: 1px solid var(--dl-border-dim);
}
.event-card:hover {
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
  transform: translateY(-1px);
  border-color: var(--dl-accent-border-hover);
}
.event-card.completed {
  opacity: 0.75;
}
.event-card.completed .card-title {
  text-decoration: line-through;
  color: rgba(255, 255, 255, 0.4);
}
.event-card.overdue {
  border-left: 3px solid #EF4444;
}
.checkbox {
  width: 24px;
  height: 24px;
  border-radius: 8px;
  border: 2px solid rgba(255, 255, 255, 0.25);
  background: transparent;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  transition: all 0.2s;
  flex-shrink: 0;
}
.checkbox.checked {
  background: var(--dl-accent-gradient);
  border-color: transparent;
  box-shadow: 0 2px 8px var(--dl-accent-shadow);
}
.card-content {
  flex: 1;
  min-width: 0;
  cursor: pointer;
}
.card-title {
  font-size: 14px;
  font-weight: 500;
  color: rgba(255, 255, 255, 0.9);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.card-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 3px;
}
.card-time {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.55);
}
.card-recurrence {
  font-size: 10px;
  color: var(--dl-accent-light);
  background: var(--dl-accent-subtle);
  padding: 2px 8px;
  border-radius: 6px;
  font-weight: 500;
}
.card-reminder {
  font-size: 10px;
  color: #fbbf24;
  background: rgba(251, 191, 36, 0.12);
  padding: 1px 6px;
  border-radius: 4px;
}
.delete-btn {
  width: 24px;
  height: 24px;
  border: none;
  background: transparent;
  color: rgba(255, 255, 255, 0.2);
  cursor: pointer;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  opacity: 0;
  transition: all 0.15s;
  flex-shrink: 0;
}
.event-card:hover .delete-btn {
  opacity: 1;
}
.delete-btn:hover {
  color: #f87171;
  background: rgba(239, 68, 68, 0.15);
}
</style>
