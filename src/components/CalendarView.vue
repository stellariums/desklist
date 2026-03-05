<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue';
import type { DeskEvent } from '../types';
import { useEvents } from '../composables/useEvents';
import { useLocale } from '../composables/useLocale';
import EventCard from './EventCard.vue';

const { fetchMonthEvents, toggleComplete, deleteEvent } = useEvents();
const { t, locale } = useLocale();

const emit = defineEmits<{
  create: [defaultTime: string | null];
  edit: [event: DeskEvent];
}>();

const today = ref(new Date());
const viewYear = ref(today.value.getFullYear());
const viewMonth = ref(today.value.getMonth()); // 0-indexed
const monthEvents = ref<DeskEvent[]>([]);
const selectedDate = ref<number | null>(null);
let todayTickTimer: ReturnType<typeof setTimeout> | null = null;

const monthLabel = computed(() => {
  const date = new Date(viewYear.value, viewMonth.value, 1);
  return locale.locale === 'en'
    ? date.toLocaleString('en-US', { month: 'long', year: 'numeric' })
    : date.toLocaleDateString('zh-CN', { year: 'numeric', month: 'long' });
});

const calendarCells = computed<(number | null)[]>(() => {
  const firstDay = new Date(viewYear.value, viewMonth.value, 1).getDay();
  const daysInMonth = new Date(viewYear.value, viewMonth.value + 1, 0).getDate();
  const cells: (number | null)[] = [];
  for (let i = 0; i < firstDay; i++) cells.push(null);
  for (let d = 1; d <= daysInMonth; d++) cells.push(d);
  while (cells.length < 42) cells.push(null);
  return cells;
});

function getLocalDate(iso: string) {
  const d = new Date(iso);
  return { year: d.getFullYear(), month: d.getMonth(), day: d.getDate() };
}

const daysWithEvents = computed<Set<number>>(() => {
  const s = new Set<number>();
  for (const e of monthEvents.value) {
    const { year, month, day } = getLocalDate(e.event_time);
    if (year === viewYear.value && month === viewMonth.value) {
      s.add(day);
    }
  }
  return s;
});

const selectedDayEvents = computed<DeskEvent[]>(() => {
  if (selectedDate.value === null) return [];
  return monthEvents.value.filter(e => {
    const { year, month, day } = getLocalDate(e.event_time);
    return year === viewYear.value && month === viewMonth.value && day === selectedDate.value;
  });
});

function isToday(day: number | null): boolean {
  if (day === null) return false;
  return day === today.value.getDate()
    && viewMonth.value === today.value.getMonth()
    && viewYear.value === today.value.getFullYear();
}

function scheduleTodayTick() {
  if (todayTickTimer) clearTimeout(todayTickTimer);

  const now = new Date();
  const nextMidnight = new Date(now);
  // Add a tiny buffer to avoid borderline timing issues exactly at midnight.
  nextMidnight.setHours(24, 0, 1, 0);
  const waitMs = Math.max(1000, nextMidnight.getTime() - now.getTime());

  todayTickTimer = setTimeout(() => {
    today.value = new Date();
    scheduleTodayTick();
  }, waitMs);
}

function getCreateDefaultTime(): string {
  const defaultTime = new Date();
  defaultTime.setMinutes(defaultTime.getMinutes() + 30);
  defaultTime.setSeconds(0, 0);

  if (selectedDate.value !== null) {
    defaultTime.setFullYear(viewYear.value, viewMonth.value, selectedDate.value);
  }

  return defaultTime.toISOString();
}

function formatSelectedDateLabel(day: number): string {
  const date = new Date(viewYear.value, viewMonth.value, day);
  return locale.locale === 'en'
    ? date.toLocaleString('en-US', { month: 'long', day: 'numeric' })
    : date.toLocaleDateString('zh-CN', { month: 'long', day: 'numeric' });
}
function prevMonth() {
  if (viewMonth.value === 0) {
    viewMonth.value = 11;
    viewYear.value--;
  } else {
    viewMonth.value--;
  }
  selectedDate.value = null;
}

function nextMonth() {
  if (viewMonth.value === 11) {
    viewMonth.value = 0;
    viewYear.value++;
  } else {
    viewMonth.value++;
  }
  selectedDate.value = null;
}

function selectDate(day: number | null) {
  if (day === null) return;
  selectedDate.value = selectedDate.value === day ? null : day;
}

async function loadMonthData() {
  monthEvents.value = await fetchMonthEvents(viewYear.value, viewMonth.value);
}

async function handleToggle(id: string) {
  await toggleComplete(id);
  await loadMonthData();
}

async function handleDelete(id: string) {
  await deleteEvent(id);
  await loadMonthData();
}

watch([viewYear, viewMonth], loadMonthData);
onMounted(() => {
  loadMonthData();
  scheduleTodayTick();
});
onUnmounted(() => {
  if (!todayTickTimer) return;
  clearTimeout(todayTickTimer);
  todayTickTimer = null;
});
defineExpose({ refresh: loadMonthData });
</script>

<template>
  <div class="calendar-view">
    <!-- Month navigation header -->
    <div class="cal-header">
      <button class="nav-btn" @click="prevMonth" aria-label="涓婃湀">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="15 18 9 12 15 6"/>
        </svg>
      </button>
      <span class="month-label">{{ monthLabel }}</span>
      <button class="nav-btn" @click="nextMonth" aria-label="涓嬫湀">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="9 18 15 12 9 6"/>
        </svg>
      </button>
    </div>

    <!-- Weekday row -->
    <div class="weekday-row">
      <span v-for="wd in t.calendarWeekdays" :key="wd" class="weekday-label">{{ wd }}</span>
    </div>

    <!-- Calendar grid -->
    <div class="cal-grid">
      <div
        v-for="(day, idx) in calendarCells"
        :key="idx"
        class="day-cell"
        :class="{
          'is-today': isToday(day),
          'is-selected': day !== null && day === selectedDate,
          'has-events': day !== null && daysWithEvents.has(day),
          'is-empty': day === null
        }"
        @click="selectDate(day)"
      >
        <span v-if="day !== null" class="day-num">{{ day }}</span>
        <span v-if="day !== null && daysWithEvents.has(day)" class="event-dot"></span>
      </div>
    </div>

    <!-- Selected day events section -->
    <div class="day-events-section">
      <template v-if="selectedDate !== null">
        <div class="day-events-header">
          {{ formatSelectedDateLabel(selectedDate) }}
        </div>
<div v-if="selectedDayEvents.length === 0" class="empty-state">
          {{ t.calendarNoEvents }}
        </div>
        <div v-else class="card-list">
          <EventCard
            v-for="event in selectedDayEvents"
            :key="event.id"
            :event="event"
            @toggle="handleToggle"
            @edit="emit('edit', $event)"
            @delete="handleDelete"
          />
        </div>
      </template>
      <div v-else class="empty-state hint">
        {{ t.calendarSelectHint }}
      </div>
    </div>

    <!-- FAB -->
    <button class="fab" @click="emit('create', getCreateDefaultTime())" :aria-label="t.newEvent">
      <svg width="20" height="20" viewBox="0 0 20 20">
        <line x1="10" y1="4" x2="10" y2="16" stroke="white" stroke-width="2.5" stroke-linecap="round"/>
        <line x1="4" y1="10" x2="16" y2="10" stroke="white" stroke-width="2.5" stroke-linecap="round"/>
      </svg>
    </button>
  </div>
</template>

<style scoped>
.calendar-view {
  flex: 1;
  display: flex;
  flex-direction: column;
  position: relative;
  min-height: 0;
  background: transparent;
}

/* Header */
.cal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 40px;
  padding: 0 16px;
  flex-shrink: 0;
  border-bottom: 1px solid var(--dl-border-dim);
}
.month-label {
  font-size: 14px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.85);
}
.nav-btn {
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
.nav-btn:hover {
  background: var(--dl-surface-strong);
  color: rgba(255, 255, 255, 0.85);
}

/* Weekday row */
.weekday-row {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  height: 28px;
  padding: 0 8px;
  flex-shrink: 0;
  border-bottom: 1px solid var(--dl-border-dim);
}
.weekday-label {
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 11px;
  color: rgba(255, 255, 255, 0.4);
  font-weight: 500;
}

/* Calendar grid */
.cal-grid {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  grid-template-rows: repeat(6, 1fr);
  height: 234px;
  padding: 4px 8px;
  flex-shrink: 0;
  border-bottom: 1px solid var(--dl-border-dim);
}
.day-cell {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 2px;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.15s;
  position: relative;
}
.day-cell.is-empty {
  cursor: default;
}
.day-cell:not(.is-empty):hover {
  background: var(--dl-surface-hover);
}
.day-cell.is-selected {
  background: var(--dl-surface-strong);
}
.day-num {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.6);
  line-height: 1;
}
.day-cell.is-today .day-num {
  color: var(--dl-accent-light);
  font-weight: 700;
}
.day-cell.is-selected .day-num {
  color: rgba(255, 255, 255, 0.95);
}
.event-dot {
  width: 4px;
  height: 4px;
  border-radius: 50%;
  background: var(--dl-accent-light);
  flex-shrink: 0;
}

/* Day events section */
.day-events-section {
  flex: 1;
  overflow-y: auto;
  padding: 8px 16px 80px;
  min-height: 0;
}
.day-events-header {
  font-size: 13px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.7);
  margin-bottom: 8px;
  padding-bottom: 6px;
  border-bottom: 1px solid var(--dl-border-dim);
}
.card-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 60px;
  color: rgba(255, 255, 255, 0.35);
  font-size: 12px;
}
.hint {
  height: 80px;
}

/* FAB */
.fab {
  position: absolute;
  bottom: 20px;
  right: 20px;
  width: 52px;
  height: 52px;
  border-radius: 16px;
  border: none;
  background: var(--dl-accent-gradient);
  color: white;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 4px 16px var(--dl-accent-shadow), 0 2px 6px rgba(0, 0, 0, 0.1);
  transition: all 0.2s;
  padding: 0;
}
.fab:hover {
  transform: scale(1.08) translateY(-2px);
  box-shadow: 0 6px 20px var(--dl-accent-shadow-strong), 0 3px 8px rgba(0, 0, 0, 0.15);
}
.fab:active {
  transform: scale(0.98);
}
</style>

