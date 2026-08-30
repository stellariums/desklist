<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue';
import type { DeskEvent } from '../types';

interface CalendarCell {
  key: string;
  day: number;
  events: DeskEvent[];
}

type WorkbenchView = 'home' | 'review' | 'inbox' | 'trash';
type CalendarMode = 'week' | 'month';
type DestructiveAction = 'trash' | 'purge';

interface NavItem {
  icon: string;
  label: string;
  key?: WorkbenchView;
  planned?: boolean;
}

const events = ref<DeskEvent[]>([]);
const inboxEvents = ref<DeskEvent[]>([]);
const trashEvents = ref<DeskEvent[]>([]);
const loading = ref(true);
const error = ref('');
const activeView = ref<WorkbenchView>('home');
const now = new Date();
const todayKey = dateKey(now);
const viewMonth = ref(new Date(now.getFullYear(), now.getMonth(), 1));
const selectedDate = ref(todayKey);
const calendarMode = ref<CalendarMode>('month');
const formVisible = ref(false);
const editingEvent = ref<DeskEvent | null>(null);
const saving = ref(false);
const capturing = ref(false);
const formError = ref('');
const inboxError = ref('');
const busyEventId = ref<string | null>(null);
const pendingAction = ref<{ mode: DestructiveAction; event: DeskEvent } | null>(null);
const form = reactive({
  title: '',
  description: '',
  date: todayKey,
  time: '09:00',
  endTime: '',
  hasDeadline: false,
  dueDate: todayKey,
  dueTime: '18:00',
  remindOnTime: true,
});
const inboxForm = reactive({
  title: '',
  description: '',
});
const weekdays = ['一', '二', '三', '四', '五', '六', '日'];

const navGroups: { label: string; items: NavItem[] }[] = [
  {
    label: '每天打开',
    items: [
      { icon: '⌂', label: '今日', key: 'home' },
      { icon: '▤', label: '收件箱', key: 'inbox' },
      { icon: '◫', label: '每日复盘', key: 'review' },
    ],
  },
  {
    label: '业务 · 待接入',
    items: [
      { icon: '✦', label: '内容工作台', planned: true },
      { icon: '▥', label: '复盘看板', planned: true },
      { icon: '♙', label: '账号分析', planned: true },
      { icon: '▣', label: '业务订单', planned: true },
    ],
  },
  {
    label: '工具',
    items: [
      { icon: '⌕', label: '全局搜索', planned: true },
      { icon: '✎', label: '单篇笔记', planned: true },
      { icon: '◎', label: '自动任务', planned: true },
      { icon: 'ϟ', label: 'Skill 库', planned: true },
      { icon: '▰', label: '知识资产', planned: true },
      { icon: '⌫', label: '回收站', key: 'trash' },
    ],
  },
];

const monthLabel = computed(() => `${viewMonth.value.getFullYear()} 年 ${viewMonth.value.getMonth() + 1} 月`);
const weekStart = computed(() => startOfWeek(parseDateKey(selectedDate.value)));
const weekEnd = computed(() => addDays(weekStart.value, 6));
const weekLabel = computed(() => {
  const start = weekStart.value;
  const end = weekEnd.value;
  if (start.getFullYear() !== end.getFullYear()) {
    return `${start.getFullYear()}年${start.getMonth() + 1}月${start.getDate()}日—${end.getFullYear()}年${end.getMonth() + 1}月${end.getDate()}日`;
  }
  if (start.getMonth() !== end.getMonth()) {
    return `${start.getFullYear()}年${start.getMonth() + 1}月${start.getDate()}日—${end.getMonth() + 1}月${end.getDate()}日`;
  }
  return `${start.getFullYear()}年${start.getMonth() + 1}月${start.getDate()}—${end.getDate()}日`;
});
const periodLabel = computed(() => (calendarMode.value === 'month' ? monthLabel.value : weekLabel.value));
const periodName = computed(() => (calendarMode.value === 'month' ? '本月' : '本周'));
const schedulingInbox = computed(() => editingEvent.value?.is_inbox === 1);
const todayEvents = computed(() => [...events.value]
  .filter((event) => dateKey(new Date(event.event_time)) === todayKey)
  .sort((a, b) => a.event_time.localeCompare(b.event_time)));
const todayCompleted = computed(() => todayEvents.value.filter((event) => event.completed === 1).length);
const todayOpen = computed(() => todayEvents.value.length - todayCompleted.value);
const historicalOverdue = computed(() => [...events.value]
  .filter((event) => event.completed === 0 && dateKey(new Date(effectiveDueTime(event))) < todayKey)
  .sort((a, b) => effectiveDueTime(a).localeCompare(effectiveDueTime(b))));
const todayDateParts = computed(() => ({
  day: String(now.getDate()).padStart(2, '0'),
  month: new Intl.DateTimeFormat('zh-CN', { year: 'numeric', month: 'long' }).format(now),
  weekday: new Intl.DateTimeFormat('zh-CN', { weekday: 'long' }).format(now),
}));
const homeLead = computed(() => {
  if (todayOpen.value > 0) return `今天还有 ${todayOpen.value} 项任务，先完成最重要的一件。`;
  if (inboxEvents.value.length > 0) return '今天的安排已经清空，可以整理一下收件箱。';
  return '今天的任务已经处理完，留一点时间做复盘。';
});

const monthEvents = computed(() => events.value.filter((event) => {
  const date = new Date(event.event_time);
  return date.getFullYear() === viewMonth.value.getFullYear()
    && date.getMonth() === viewMonth.value.getMonth();
}));

const monthStats = computed(() => ({
  activeDays: new Set(monthEvents.value.map((event) => dateKey(new Date(event.event_time)))).size,
  open: monthEvents.value.filter((event) => event.completed === 0).length,
  overdue: monthEvents.value.filter((event) => isOverdue(event)).length,
}));

const weekEvents = computed(() => {
  const start = weekStart.value.getTime();
  const end = addDays(weekEnd.value, 1).getTime();
  return events.value.filter((event) => {
    const eventTime = new Date(event.event_time).getTime();
    return eventTime >= start && eventTime < end;
  });
});

const weekStats = computed(() => ({
  activeDays: new Set(weekEvents.value.map((event) => dateKey(new Date(event.event_time)))).size,
  open: weekEvents.value.filter((event) => event.completed === 0).length,
  overdue: weekEvents.value.filter((event) => isOverdue(event)).length,
}));

const periodStats = computed(() => (calendarMode.value === 'month' ? monthStats.value : weekStats.value));

const eventMap = computed(() => {
  const map = new Map<string, DeskEvent[]>();
  for (const event of monthEvents.value) {
    const key = dateKey(new Date(event.event_time));
    const items = map.get(key) ?? [];
    items.push(event);
    map.set(key, items);
  }
  return map;
});

const calendarCells = computed<(CalendarCell | null)[]>(() => {
  const year = viewMonth.value.getFullYear();
  const month = viewMonth.value.getMonth();
  const firstDay = new Date(year, month, 1);
  const offset = (firstDay.getDay() + 6) % 7;
  const days = new Date(year, month + 1, 0).getDate();
  const cells: (CalendarCell | null)[] = Array.from({ length: offset }, () => null);
  for (let day = 1; day <= days; day += 1) {
    const key = dateKey(new Date(year, month, day));
    cells.push({ key, day, events: eventMap.value.get(key) ?? [] });
  }
  while (cells.length < 42) cells.push(null);
  return cells;
});

const weekCells = computed<CalendarCell[]>(() => Array.from({ length: 7 }, (_, index) => {
  const date = addDays(weekStart.value, index);
  const key = dateKey(date);
  return {
    key,
    day: date.getDate(),
    events: events.value
      .filter((event) => dateKey(new Date(event.event_time)) === key)
      .sort((a, b) => a.event_time.localeCompare(b.event_time)),
  };
}));

const selectedEvents = computed(() => (
  [...events.value]
    .filter((event) => dateKey(new Date(event.event_time)) === selectedDate.value)
    .sort((a, b) => a.event_time.localeCompare(b.event_time))
));

const selectedDateLabel = computed(() => {
  const date = parseDateKey(selectedDate.value);
  return new Intl.DateTimeFormat('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    weekday: 'long',
  }).format(date);
});

const selectedSummary = computed(() => {
  if (selectedEvents.value.length === 0) {
    return '当天还没有任务记录。这里已经为每日复盘、进展总结和业务数据留好位置，后续接入真实内容后会统一展示。';
  }
  const completed = selectedEvents.value.filter((event) => event.completed === 1).length;
  const open = selectedEvents.value.length - completed;
  return `当天共记录 ${selectedEvents.value.length} 项任务，已完成 ${completed} 项，仍有 ${open} 项待处理。当前先根据 Desklist 任务生成摘要，后续再接入更完整的每日复盘内容。`;
});

const nextActions = computed(() => {
  const selectedOpen = selectedEvents.value.filter((event) => event.completed === 0);
  if (selectedOpen.length > 0) return selectedOpen.slice(0, 3);
  return [...events.value]
    .filter((event) => event.completed === 0)
    .sort((a, b) => a.event_time.localeCompare(b.event_time))
    .slice(0, 3);
});

onMounted(loadEvents);

async function loadEvents() {
  loading.value = true;
  error.value = '';
  try {
    const [activeResponse, inboxResponse, trashResponse] = await Promise.all([
      fetch('/api/events?filter=all'),
      fetch('/api/events?filter=inbox'),
      fetch('/api/events?filter=trash'),
    ]);
    const [activeBody, inboxBody, trashBody] = await Promise.all([
      activeResponse.json(),
      inboxResponse.json(),
      trashResponse.json(),
    ]);
    if (!activeResponse.ok) throw new Error(activeBody.error || '任务读取失败');
    if (!inboxResponse.ok) throw new Error(inboxBody.error || '收件箱读取失败');
    if (!trashResponse.ok) throw new Error(trashBody.error || '回收站读取失败');
    events.value = activeBody;
    inboxEvents.value = inboxBody;
    trashEvents.value = trashBody;
  } catch (reason) {
    error.value = reason instanceof Error ? reason.message : '任务读取失败';
  } finally {
    loading.value = false;
  }
}

async function submitInbox() {
  const title = inboxForm.title.trim();
  if (!title) {
    inboxError.value = '先写下一件需要记住的事';
    return;
  }
  capturing.value = true;
  inboxError.value = '';
  try {
    const response = await fetch('/api/inbox', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        title,
        description: inboxForm.description.trim(),
      }),
    });
    const body = await response.json().catch(() => ({}));
    if (!response.ok) throw new Error(body.error || '收集任务失败');
    inboxForm.title = '';
    inboxForm.description = '';
    await loadEvents();
  } catch (reason) {
    inboxError.value = reason instanceof Error ? reason.message : '收集任务失败';
  } finally {
    capturing.value = false;
  }
}

function selectNav(item: NavItem) {
  if (item.planned || !item.key) return;
  activeView.value = item.key;
  error.value = '';
}

function openCreate(date = selectedDate.value) {
  editingEvent.value = null;
  form.title = '';
  form.description = '';
  form.date = date;
  form.time = '09:00';
  form.endTime = '';
  form.hasDeadline = false;
  form.dueDate = date;
  form.dueTime = '18:00';
  form.remindOnTime = true;
  formError.value = '';
  formVisible.value = true;
}

function openEdit(event: DeskEvent) {
  const eventDate = new Date(event.event_time);
  editingEvent.value = event;
  form.title = event.title;
  form.description = event.description;
  form.date = dateKey(eventDate);
  form.time = `${String(eventDate.getHours()).padStart(2, '0')}:${String(eventDate.getMinutes()).padStart(2, '0')}`;
  const scheduledEnd = event.scheduled_end ? new Date(event.scheduled_end) : null;
  form.endTime = scheduledEnd
    ? `${String(scheduledEnd.getHours()).padStart(2, '0')}:${String(scheduledEnd.getMinutes()).padStart(2, '0')}`
    : '';
  const dueTime = event.due_time ? new Date(event.due_time) : null;
  form.hasDeadline = dueTime !== null;
  form.dueDate = dueTime ? dateKey(dueTime) : form.date;
  form.dueTime = dueTime
    ? `${String(dueTime.getHours()).padStart(2, '0')}:${String(dueTime.getMinutes()).padStart(2, '0')}`
    : '18:00';
  form.remindOnTime = event.remind_on_time !== 0;
  formError.value = '';
  formVisible.value = true;
}

function openSchedule(event: DeskEvent) {
  openEdit(event);
  const current = new Date();
  form.date = dateKey(current);
  form.time = `${String(current.getHours()).padStart(2, '0')}:${String(current.getMinutes()).padStart(2, '0')}`;
  form.endTime = '';
  form.hasDeadline = false;
  form.dueDate = form.date;
  form.dueTime = '18:00';
  form.remindOnTime = true;
}

function closeForm() {
  if (saving.value) return;
  formVisible.value = false;
  editingEvent.value = null;
  formError.value = '';
}

async function submitForm() {
  const title = form.title.trim();
  if (!title) {
    formError.value = '请填写任务标题';
    return;
  }
  const localTime = new Date(`${form.date}T${form.time}:00`);
  if (Number.isNaN(localTime.getTime())) {
    formError.value = '日期或时间格式不正确';
    return;
  }
  const scheduledEnd = form.endTime ? new Date(`${form.date}T${form.endTime}:00`) : null;
  if (scheduledEnd && scheduledEnd <= localTime) {
    formError.value = '安排结束时间必须晚于开始时间';
    return;
  }
  const deadline = form.hasDeadline ? new Date(`${form.dueDate}T${form.dueTime}:00`) : null;
  if (deadline && deadline < localTime) {
    formError.value = '截止时间不能早于安排开始时间';
    return;
  }

  saving.value = true;
  formError.value = '';
  try {
    const payload = {
      title,
      description: form.description.trim(),
      event_time: localTime.toISOString(),
      scheduled_end: scheduledEnd?.toISOString() ?? null,
      due_time: deadline?.toISOString() ?? null,
      remind_at: adjustedAdvanceReminder(deadline ?? localTime),
      remind_on_time: form.remindOnTime ? 1 : 0,
      recurrence: editingEvent.value?.recurrence ?? 'none',
      recurrence_end: editingEvent.value?.recurrence_end ?? null,
    };
    const response = await fetch(
      editingEvent.value ? `/api/events/${editingEvent.value.id}` : '/api/events',
      {
        method: editingEvent.value ? 'PUT' : 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
      },
    );
    const body = await response.json().catch(() => ({}));
    if (!response.ok) throw new Error(body.error || '保存失败');

    selectedDate.value = form.date;
    viewMonth.value = new Date(localTime.getFullYear(), localTime.getMonth(), 1);
    formVisible.value = false;
    editingEvent.value = null;
    await loadEvents();
  } catch (reason) {
    formError.value = reason instanceof Error ? reason.message : '保存失败';
  } finally {
    saving.value = false;
  }
}

function adjustedAdvanceReminder(newReminderTarget: Date) {
  const event = editingEvent.value;
  if (!event?.remind_at) return null;
  const originalTime = new Date(event.due_time || event.event_time).getTime();
  const originalReminder = new Date(event.remind_at).getTime();
  if (Number.isNaN(originalTime) || Number.isNaN(originalReminder)) return null;
  return new Date(newReminderTarget.getTime() - (originalTime - originalReminder)).toISOString();
}

async function toggleEvent(event: DeskEvent) {
  busyEventId.value = event.id;
  try {
    const response = await fetch(`/api/events/${event.id}/toggle`, { method: 'POST' });
    const body = await response.json().catch(() => ({}));
    if (!response.ok) throw new Error(body.error || '状态更新失败');
    await loadEvents();
  } catch (reason) {
    error.value = reason instanceof Error ? reason.message : '状态更新失败';
  } finally {
    busyEventId.value = null;
  }
}

function requestTrash(event: DeskEvent) {
  pendingAction.value = { mode: 'trash', event };
}

function requestPurge(event: DeskEvent) {
  pendingAction.value = { mode: 'purge', event };
}

function closeActionPrompt() {
  if (busyEventId.value) return;
  pendingAction.value = null;
}

async function confirmAction() {
  const action = pendingAction.value;
  if (!action) return;
  busyEventId.value = action.event.id;
  try {
    const response = await fetch(
      action.mode === 'trash'
        ? `/api/events/${action.event.id}/trash`
        : `/api/events/${action.event.id}`,
      { method: action.mode === 'trash' ? 'POST' : 'DELETE' },
    );
    const body = await response.json().catch(() => ({}));
    if (!response.ok) {
      throw new Error(body.error || (action.mode === 'trash' ? '移入回收站失败' : '彻底删除失败'));
    }
    pendingAction.value = null;
    await loadEvents();
  } catch (reason) {
    error.value = reason instanceof Error ? reason.message : '任务操作失败';
  } finally {
    busyEventId.value = null;
  }
}

async function restoreEvent(event: DeskEvent) {
  busyEventId.value = event.id;
  try {
    const response = await fetch(`/api/events/${event.id}/restore`, { method: 'POST' });
    const body = await response.json().catch(() => ({}));
    if (!response.ok) throw new Error(body.error || '恢复任务失败');
    await loadEvents();
  } catch (reason) {
    error.value = reason instanceof Error ? reason.message : '恢复任务失败';
  } finally {
    busyEventId.value = null;
  }
}

function changeMonth(offset: number) {
  const next = new Date(viewMonth.value.getFullYear(), viewMonth.value.getMonth() + offset, 1);
  viewMonth.value = next;
  const sameAsToday = next.getFullYear() === now.getFullYear() && next.getMonth() === now.getMonth();
  selectedDate.value = dateKey(sameAsToday ? now : next);
}

function changePeriod(offset: number) {
  if (calendarMode.value === 'month') {
    changeMonth(offset);
    return;
  }
  const next = addDays(parseDateKey(selectedDate.value), offset * 7);
  selectedDate.value = dateKey(next);
  viewMonth.value = new Date(next.getFullYear(), next.getMonth(), 1);
}

function setCalendarMode(mode: CalendarMode) {
  calendarMode.value = mode;
  if (mode === 'month') {
    const selected = parseDateKey(selectedDate.value);
    viewMonth.value = new Date(selected.getFullYear(), selected.getMonth(), 1);
  }
}

function goToToday() {
  selectedDate.value = todayKey;
  viewMonth.value = new Date(now.getFullYear(), now.getMonth(), 1);
}

function selectDate(cell: CalendarCell) {
  selectedDate.value = cell.key;
}

function dateKey(date: Date) {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');
  return `${year}-${month}-${day}`;
}

function parseDateKey(value: string) {
  const [year, month, day] = value.split('-').map(Number);
  return new Date(year, month - 1, day);
}

function addDays(date: Date, amount: number) {
  return new Date(date.getFullYear(), date.getMonth(), date.getDate() + amount);
}

function startOfWeek(date: Date) {
  const mondayOffset = (date.getDay() + 6) % 7;
  return addDays(date, -mondayOffset);
}

function weekDateLabel(value: string) {
  const date = parseDateKey(value);
  return `${date.getMonth() + 1}月${date.getDate()}日`;
}

function isOverdue(event: DeskEvent) {
  return event.completed === 0 && new Date(effectiveDueTime(event)).getTime() < Date.now();
}

function effectiveDueTime(event: DeskEvent) {
  return event.due_time || event.event_time;
}

function cellStatus(cell: CalendarCell) {
  if (cell.events.length === 0) return 'empty';
  if (cell.events.every((event) => event.completed === 1)) return 'completed';
  if (cell.events.some((event) => isOverdue(event))) return 'overdue';
  return 'active';
}

function statusLabel(cell: CalendarCell) {
  const status = cellStatus(cell);
  if (status === 'completed') return '已经完成';
  if (status === 'overdue') return '需要跟进';
  return '任务推进';
}

function formatTime(value: string) {
  return new Intl.DateTimeFormat('zh-CN', {
    hour: '2-digit',
    minute: '2-digit',
  }).format(new Date(value));
}

function formatSchedule(event: DeskEvent) {
  const start = formatTime(event.event_time);
  if (!event.scheduled_end) return start;
  const endDate = new Date(event.scheduled_end);
  if (dateKey(endDate) === dateKey(new Date(event.event_time))) {
    return `${start}–${formatTime(event.scheduled_end)}`;
  }
  return `${start}–${formatShortDate(event.scheduled_end)} ${formatTime(event.scheduled_end)}`;
}

function formatDeadline(event: DeskEvent) {
  if (!event.due_time) return '';
  return `截止 ${formatDateTime(event.due_time)}`;
}

function formatDateTime(value: string) {
  return new Intl.DateTimeFormat('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  }).format(new Date(value));
}

function formatShortDate(value: string) {
  return new Intl.DateTimeFormat('zh-CN', {
    month: '2-digit',
    day: '2-digit',
  }).format(new Date(value));
}

function priority(event: DeskEvent) {
  if (isOverdue(event)) return 'P0';
  if (dateKey(new Date(event.event_time)) === todayKey) return 'P1';
  return 'P2';
}
</script>

<template>
  <div class="dashboard-shell">
    <aside class="sidebar">
      <div class="brand">
        <span class="brand-mark">桌</span>
        <div>
          <strong>个人工作台</strong>
          <small>DESKLIST OS</small>
        </div>
      </div>

      <div class="mobile-switcher" aria-label="移动端页面导航">
        <button :class="{ active: activeView === 'home' }" @click="activeView = 'home'">今日</button>
        <button :class="{ active: activeView === 'review' }" @click="activeView = 'review'">复盘</button>
        <button :class="{ active: activeView === 'inbox' }" @click="activeView = 'inbox'">
          收件箱 {{ inboxEvents.length }}
        </button>
        <button :class="{ active: activeView === 'trash' }" @click="activeView = 'trash'">
          回收站 {{ trashEvents.length }}
        </button>
      </div>

      <nav class="sidebar-nav" aria-label="工作台导航">
        <section v-for="group in navGroups" :key="group.label" class="nav-group">
          <p>{{ group.label }}</p>
          <button
            v-for="item in group.items"
            :key="item.label"
            :class="{ active: item.key === activeView, planned: item.planned }"
            :aria-current="item.key === activeView ? 'page' : undefined"
            :title="item.planned ? '位置已预留，功能稍后接入' : item.label"
            :disabled="item.planned"
            @click="selectNav(item)"
          >
            <span class="nav-icon">{{ item.icon }}</span>
            <span>{{ item.label }}</span>
            <small v-if="item.key === 'inbox'" class="nav-count inbox">{{ inboxEvents.length }}</small>
            <small v-else-if="item.key === 'trash'" class="nav-count">{{ trashEvents.length }}</small>
            <small v-else-if="item.planned">待接入</small>
          </button>
        </section>
      </nav>

      <div class="sidebar-status">
        <span class="status-dot"></span>
        <div>
          <strong>数据源：本机 Desklist</strong>
          <small>浏览器工作台 · 可操作</small>
        </div>
      </div>
    </aside>

    <main class="workspace">
      <template v-if="activeView === 'home'">
        <header class="workspace-header home-header">
          <div class="title-row">
            <span class="title-icon">⌂</span>
            <h1>今日</h1>
            <span class="header-tag">每天打开</span>
            <span class="header-tag quiet">{{ todayDateParts.weekday }}</span>
          </div>
          <p>先看清今天，再决定下一步。</p>
        </header>

        <div v-if="error" class="home-error">
          <strong>今日数据暂时无法读取</strong>
          <span>{{ error }}</span>
        </div>

        <section class="home-hero" aria-label="今日摘要">
          <div class="today-date-block">
            <small>TODAY</small>
            <strong>{{ todayDateParts.day }}</strong>
            <span>{{ todayDateParts.month }} · {{ todayDateParts.weekday }}</span>
          </div>
          <div class="home-lead">
            <span class="home-kicker">DAILY BRIEFING</span>
            <h2>把今天过清楚。</h2>
            <p>{{ homeLead }}</p>
            <div class="home-quick-actions">
              <button type="button" class="primary" @click="openCreate(todayKey)">＋ 新建今日任务</button>
              <button type="button" @click="activeView = 'inbox'">放入收件箱</button>
              <button type="button" @click="activeView = 'review'">打开日历复盘</button>
            </div>
          </div>
          <div class="home-stats">
            <div>
              <small>今日待办</small>
              <strong>{{ todayOpen }}</strong>
              <span>{{ todayCompleted }} 项已完成</span>
            </div>
            <div>
              <small>历史逾期</small>
              <strong class="red">{{ historicalOverdue.length }}</strong>
              <span>需要重新决定</span>
            </div>
            <div>
              <small>收件箱</small>
              <strong class="green">{{ inboxEvents.length }}</strong>
              <span>尚未安排时间</span>
            </div>
          </div>
        </section>

        <section class="home-grid">
          <article class="home-panel today-agenda">
            <header class="home-panel-header">
              <div>
                <small>AGENDA</small>
                <h2>今日安排</h2>
              </div>
              <span>{{ todayEvents.length }} 项</span>
            </header>

            <div v-if="loading" class="home-empty">正在整理今天的安排……</div>
            <div v-else-if="todayEvents.length === 0" class="home-empty">
              <span>○</span>
              <strong>今天还没有安排</strong>
              <p>可以新建一个今日任务，或者先从收件箱安排。</p>
              <button type="button" @click="openCreate(todayKey)">新建今日任务</button>
            </div>
            <ol v-else class="home-task-list">
              <li v-for="event in todayEvents" :key="event.id" :class="{ completed: event.completed }">
                <time :datetime="event.event_time">{{ formatSchedule(event) }}</time>
                <span class="agenda-line" aria-hidden="true"></span>
                <div class="agenda-copy">
                  <strong>{{ event.title }}</strong>
                  <p>{{ event.description || (event.completed ? '已经完成。' : '等待处理。') }}</p>
                  <span v-if="event.due_time" class="deadline-note">{{ formatDeadline(event) }}</span>
                </div>
                <div class="agenda-actions">
                  <button type="button" :disabled="busyEventId === event.id" @click="toggleEvent(event)">
                    {{ event.completed ? '恢复' : '完成' }}
                  </button>
                  <button type="button" @click="openEdit(event)">编辑</button>
                  <button type="button" class="danger" @click="requestTrash(event)">删除</button>
                </div>
              </li>
            </ol>
          </article>

          <aside class="home-side-stack">
            <article class="home-panel overdue-panel">
              <header class="home-panel-header compact">
                <div>
                  <small>NEEDS A DECISION</small>
                  <h2>历史逾期</h2>
                </div>
                <span>{{ historicalOverdue.length }} 项</span>
              </header>
              <div v-if="historicalOverdue.length === 0" class="side-empty">没有历史逾期任务。</div>
              <ul v-else class="overdue-list">
                <li v-for="event in historicalOverdue.slice(0, 4)" :key="event.id">
                  <time :datetime="effectiveDueTime(event)">{{ formatShortDate(effectiveDueTime(event)) }}</time>
                  <div>
                    <strong>{{ event.title }}</strong>
                    <span>{{ event.due_time ? '截止' : '原安排' }} {{ formatTime(effectiveDueTime(event)) }}</span>
                  </div>
                  <button type="button" @click="openEdit(event)">重排</button>
                </li>
              </ul>
              <button
                v-if="historicalOverdue.length > 4"
                type="button"
                class="panel-link"
                @click="activeView = 'review'"
              >
                查看其余 {{ historicalOverdue.length - 4 }} 项 →
              </button>
            </article>

            <article class="home-panel inbox-preview-panel">
              <header class="home-panel-header compact">
                <div>
                  <small>UNSCHEDULED</small>
                  <h2>收件箱</h2>
                </div>
                <button type="button" @click="activeView = 'inbox'">打开全部</button>
              </header>
              <div v-if="inboxEvents.length === 0" class="side-empty">没有等待安排的任务。</div>
              <ul v-else class="home-inbox-list">
                <li v-for="event in inboxEvents.slice(0, 3)" :key="event.id">
                  <span aria-hidden="true">▤</span>
                  <strong>{{ event.title }}</strong>
                  <button type="button" @click="openSchedule(event)">安排</button>
                </li>
              </ul>
            </article>
          </aside>
        </section>
      </template>

      <template v-else-if="activeView === 'review'">
      <header class="workspace-header">
        <div class="title-row">
          <span class="title-icon">▰</span>
          <h1>每日复盘</h1>
          <span class="header-tag">日历复盘</span>
          <span class="header-tag quiet">本机任务</span>
          <button class="quick-add" type="button" @click="openCreate()">＋ 新建任务</button>
        </div>
        <p>每天做了什么、推进了什么、下一步是什么——先从真实任务记录开始。</p>
      </header>

      <section class="calendar-panel" :aria-label="calendarMode === 'month' ? '任务月历' : '任务周历'">
        <div class="calendar-toolbar">
          <button
            class="month-button"
            :aria-label="calendarMode === 'month' ? '上个月' : '上一周'"
            @click="changePeriod(-1)"
          >←</button>
          <h2>{{ periodLabel }}</h2>
          <button
            class="month-button"
            :aria-label="calendarMode === 'month' ? '下个月' : '下一周'"
            @click="changePeriod(1)"
          >→</button>
          <button class="today-button" type="button" @click="goToToday">今天</button>
          <div class="calendar-mode-switch" aria-label="日历视图">
            <button
              type="button"
              :class="{ active: calendarMode === 'week' }"
              :aria-pressed="calendarMode === 'week'"
              @click="setCalendarMode('week')"
            >周</button>
            <button
              type="button"
              :class="{ active: calendarMode === 'month' }"
              :aria-pressed="calendarMode === 'month'"
              @click="setCalendarMode('month')"
            >月</button>
          </div>
          <div class="month-summary">
            <span>{{ periodName }} <strong>{{ periodStats.activeDays }}</strong> 天有记录</span>
            <span>待推进 <strong class="green">{{ periodStats.open }}</strong></span>
            <span>已经逾期 <strong class="red">{{ periodStats.overdue }}</strong></span>
          </div>
        </div>

        <div v-if="loading" class="calendar-state">正在读取任务月历……</div>
        <div v-else-if="error" class="calendar-state error">
          <strong>暂时无法读取任务</strong>
          <span>{{ error }}</span>
        </div>
        <template v-else-if="calendarMode === 'month'">
          <div class="weekday-row" aria-hidden="true">
            <span v-for="weekday in weekdays" :key="weekday">{{ weekday }}</span>
          </div>

          <div class="calendar-grid">
            <template v-for="(cell, index) in calendarCells" :key="cell?.key ?? `blank-${index}`">
              <button
                v-if="cell"
                class="day-cell"
                :class="[
                  `status-${cellStatus(cell)}`,
                  { selected: selectedDate === cell.key, today: cell.key === todayKey },
                ]"
                @click="selectDate(cell)"
              >
                <span class="day-number">{{ cell.day }}</span>
                <small v-if="cell.events.length">{{ cell.events.length }} 项</small>
                <span v-if="cell.events.length" class="cell-title">
                  {{ cell.events[0].title }}
                  <em v-if="cell.events.length > 1">+{{ cell.events.length - 1 }}</em>
                </span>
                <span v-if="cell.events.length" class="cell-status">{{ statusLabel(cell) }}</span>
              </button>
              <div v-else class="day-cell blank"></div>
            </template>
          </div>
        </template>

        <div v-else class="week-board">
          <button
            v-for="(cell, index) in weekCells"
            :key="cell.key"
            class="week-day-card"
            :class="[
              `status-${cellStatus(cell)}`,
              { selected: selectedDate === cell.key, today: cell.key === todayKey },
            ]"
            @click="selectDate(cell)"
            >
              <span class="week-day-heading">
                <small>星期{{ weekdays[index] }}</small>
                <strong>{{ weekDateLabel(cell.key) }}</strong>
              </span>
              <span v-if="cell.events.length === 0" class="week-empty">留白</span>
              <template v-else>
                <span v-for="event in cell.events.slice(0, 3)" :key="event.id" class="week-event">
                  <time :datetime="event.event_time">{{ formatTime(event.event_time) }}</time>
                  <strong>{{ event.title }}</strong>
                  <em v-if="event.due_time" aria-label="有截止时间">●</em>
                </span>
              </template>
            <span v-if="cell.events.length > 3" class="week-more">另有 {{ cell.events.length - 3 }} 项</span>
          </button>
        </div>
      </section>

      <section class="review-grid">
        <article class="review-card daily-review">
          <header class="card-header">
            <div>
              <strong>{{ selectedDateLabel }}</strong>
              <span class="review-pill">每日记录</span>
            </div>
            <small>{{ selectedEvents.length }} 项任务</small>
          </header>

          <p class="review-summary">{{ selectedSummary }}</p>

          <div class="review-section">
            <h3>✅ 当天任务</h3>
            <ul v-if="selectedEvents.length" class="review-list">
              <li v-for="event in selectedEvents" :key="event.id">
                <span :class="['task-dot', { done: event.completed }]" aria-hidden="true"></span>
                <div>
                  <strong>{{ event.title }}</strong>
                  <p>{{ event.description || (event.completed ? '这项任务已经完成。' : '这项任务仍在等待处理。') }}</p>
                  <span v-if="event.due_time" class="deadline-note">{{ formatDeadline(event) }}</span>
                </div>
                <div class="task-actions">
                  <time :datetime="event.event_time">{{ formatSchedule(event) }}</time>
                  <button type="button" :disabled="busyEventId === event.id" @click="toggleEvent(event)">
                    {{ event.completed ? '恢复' : '完成' }}
                  </button>
                  <button type="button" @click="openEdit(event)">编辑</button>
                  <button type="button" class="danger" @click="requestTrash(event)">删除</button>
                </div>
              </li>
            </ul>
            <div v-else class="reserved-copy">
              <span>复盘内容预留区</span>
              <p>以后可以在这里接入当天完成的工作、业务进展和关键反馈。</p>
            </div>
          </div>
        </article>

        <div class="review-side">
          <article class="review-card suggestion-card">
            <header class="card-header compact">
              <strong>🎯 下一步建议</strong>
              <span>根据未完成任务</span>
            </header>
            <ol v-if="nextActions.length" class="suggestion-list">
              <li v-for="event in nextActions" :key="event.id">
                <span>{{ priority(event) }}</span>
                <div>
                  <strong>{{ event.title }}</strong>
                  <small>{{ formatSchedule(event) }}</small>
                </div>
              </li>
            </ol>
            <p v-else class="empty-copy">当前没有需要继续处理的任务。</p>
          </article>

          <article class="review-card reserved-card">
            <header class="card-header compact">
              <strong>📌 业务数据</strong>
              <span>待接入</span>
            </header>
            <div class="reserved-modules">
              <span>内容数据</span>
              <span>账号表现</span>
              <span>订单进展</span>
            </div>
            <p>位置已经预留，等你确认真实业务字段后再接入。</p>
          </article>
        </div>
      </section>
      </template>

      <template v-else-if="activeView === 'inbox'">
        <header class="workspace-header inbox-header">
          <div class="title-row">
            <span class="title-icon">▤</span>
            <h1>收件箱</h1>
            <span class="header-tag inbox-tag">先收集，再安排</span>
            <span class="trash-count">{{ inboxEvents.length }} 项待安排</span>
          </div>
          <p>先把脑海里的事情放下来，不必立刻决定日期；需要执行时再安排进日历。</p>
        </header>

        <section class="inbox-capture">
          <div class="capture-intro">
            <small>QUICK CAPTURE</small>
            <h2>现在想到什么？</h2>
            <p>只写清楚要做什么即可，时间可以以后再定。</p>
          </div>
          <form class="capture-form" @submit.prevent="submitInbox">
            <label>
              <span>任务</span>
              <input
                v-model="inboxForm.title"
                maxlength="200"
                placeholder="例如：整理访学照片"
                autofocus
              />
            </label>
            <label>
              <span>补充说明 <em>可选</em></span>
              <textarea
                v-model="inboxForm.description"
                maxlength="1000"
                rows="2"
                placeholder="背景、想法或之后需要确认的信息"
              ></textarea>
            </label>
            <div class="capture-submit">
              <p v-if="inboxError">{{ inboxError }}</p>
              <button type="submit" :disabled="capturing">
                {{ capturing ? '正在收集…' : '放入收件箱 →' }}
              </button>
            </div>
          </form>
        </section>

        <section class="inbox-panel" aria-label="待安排任务">
          <div class="inbox-panel-heading">
            <div>
              <small>UNSCHEDULED</small>
              <h2>待安排</h2>
            </div>
            <p>安排日期后，任务会离开收件箱并出现在对应日历中。</p>
          </div>

          <div v-if="loading" class="trash-empty">正在读取收件箱……</div>
          <div v-else-if="error" class="trash-empty error">
            <strong>暂时无法读取收件箱</strong>
            <span>{{ error }}</span>
          </div>
          <div v-else-if="inboxEvents.length === 0" class="inbox-empty">
            <span aria-hidden="true">✓</span>
            <strong>没有等待安排的任务</strong>
            <p>想到新事项时，先从上方快速收集。</p>
          </div>
          <ul v-else class="inbox-list">
            <li v-for="event in inboxEvents" :key="event.id">
              <div class="inbox-sequence">{{ String(inboxEvents.indexOf(event) + 1).padStart(2, '0') }}</div>
              <div class="inbox-copy">
                <strong>{{ event.title }}</strong>
                <p>{{ event.description || '尚未补充说明。' }}</p>
                <span>收集于 {{ formatDateTime(event.created_at) }}</span>
              </div>
              <div class="inbox-actions">
                <button type="button" class="schedule" @click="openSchedule(event)">安排时间</button>
                <button type="button" class="remove" @click="requestTrash(event)">删除</button>
              </div>
            </li>
          </ul>
        </section>
      </template>

      <template v-else>
        <header class="workspace-header trash-header">
          <div class="title-row">
            <span class="title-icon">⌫</span>
            <h1>回收站</h1>
            <span class="header-tag quiet">安全删除</span>
            <span class="trash-count">{{ trashEvents.length }} 项</span>
          </div>
          <p>删除的任务会先留在这里；恢复后重新回到原日期，彻底删除后无法找回。</p>
        </header>

        <section class="trash-panel" aria-label="回收站任务">
          <div class="trash-panel-heading">
            <div>
              <small>RECYCLE BIN</small>
              <h2>已删除任务</h2>
            </div>
            <p>提醒已暂停，恢复未完成任务时只会重建未来的提醒。</p>
          </div>

          <div v-if="loading" class="trash-empty">正在读取回收站……</div>
          <div v-else-if="error" class="trash-empty error">
            <strong>暂时无法读取回收站</strong>
            <span>{{ error }}</span>
          </div>
          <div v-else-if="trashEvents.length === 0" class="trash-empty">
            <span class="empty-bin" aria-hidden="true">⌫</span>
            <strong>回收站是空的</strong>
            <p>从每日复盘中删除的任务会暂存在这里。</p>
          </div>
          <ul v-else class="trash-list">
            <li v-for="event in trashEvents" :key="event.id">
              <div class="trash-date">
                <small>删除于</small>
                <strong>{{ formatDateTime(event.deleted_at || event.updated_at) }}</strong>
              </div>
              <div class="trash-copy">
                <strong>{{ event.title }}</strong>
                <p>{{ event.description || '没有补充说明。' }}</p>
                <span v-if="event.is_inbox">来自收件箱 · 尚未安排时间</span>
                <span v-else>
                  原安排 {{ formatDateTime(event.event_time) }}
                  <template v-if="event.due_time"> · {{ formatDeadline(event) }}</template>
                </span>
              </div>
              <div class="trash-actions">
                <button
                  type="button"
                  class="restore"
                  :disabled="busyEventId === event.id"
                  @click="restoreEvent(event)"
                >
                  恢复任务
                </button>
                <button
                  type="button"
                  class="purge"
                  :disabled="busyEventId === event.id"
                  @click="requestPurge(event)"
                >
                  彻底删除
                </button>
              </div>
            </li>
          </ul>
        </section>
      </template>
    </main>

    <div v-if="formVisible" class="modal-overlay" @click.self="closeForm" @keydown.esc="closeForm">
      <form class="task-modal" @submit.prevent="submitForm">
        <header>
          <div>
            <small>{{ schedulingInbox ? 'SCHEDULE TASK' : editingEvent ? 'EDIT TASK' : 'NEW TASK' }}</small>
            <h2>{{ schedulingInbox ? '安排收件箱任务' : editingEvent ? '编辑任务' : '新建任务' }}</h2>
          </div>
          <button type="button" aria-label="关闭" @click="closeForm">×</button>
        </header>

        <label>
          <span>任务标题</span>
          <input v-model="form.title" maxlength="200" placeholder="准备做什么？" autofocus />
        </label>

        <div class="form-section-title">
          <span>安排时段</span>
          <small>决定什么时候做</small>
        </div>

        <div class="form-row schedule-row">
          <label>
            <span>安排日期</span>
            <input v-model="form.date" type="date" required />
          </label>
          <label>
            <span>开始</span>
            <input v-model="form.time" type="time" required />
          </label>
          <label>
            <span>结束 <em>可选</em></span>
            <input v-model="form.endTime" type="time" />
          </label>
        </div>

        <label class="deadline-toggle">
          <input v-model="form.hasDeadline" type="checkbox" />
          <span>设置截止时间</span>
          <small>决定最晚什么时候完成</small>
        </label>

        <div v-if="form.hasDeadline" class="form-row deadline-row">
          <label>
            <span>截止日期</span>
            <input v-model="form.dueDate" type="date" required />
          </label>
          <label>
            <span>截止时间</span>
            <input v-model="form.dueTime" type="time" required />
          </label>
        </div>

        <label>
          <span>补充说明</span>
          <textarea v-model="form.description" maxlength="1000" rows="4" placeholder="可选：背景、要求或下一步"></textarea>
        </label>

        <label class="reminder-check">
          <input v-model="form.remindOnTime" type="checkbox" />
          <span>{{ form.hasDeadline ? '到截止时间时提醒我' : '到安排开始时提醒我' }}</span>
        </label>

        <p v-if="formError" class="form-error">{{ formError }}</p>

        <footer>
          <button type="button" class="secondary" @click="closeForm">取消</button>
          <button type="submit" class="primary" :disabled="saving">
            {{ saving ? '正在保存…' : schedulingInbox ? '确认安排' : editingEvent ? '保存修改' : '创建任务' }}
          </button>
        </footer>
      </form>
    </div>

    <div
      v-if="pendingAction"
      class="modal-overlay"
      @click.self="closeActionPrompt"
      @keydown.esc="closeActionPrompt"
    >
      <section class="confirm-modal" role="alertdialog" aria-modal="true" aria-labelledby="confirm-title">
        <span class="confirm-mark">{{ pendingAction.mode === 'trash' ? '⌫' : '!' }}</span>
        <small>{{ pendingAction.mode === 'trash' ? 'MOVE TO RECYCLE BIN' : 'PERMANENT DELETE' }}</small>
        <h2 id="confirm-title">
          {{ pendingAction.mode === 'trash' ? '移入回收站？' : '确定彻底删除？' }}
        </h2>
        <p>
          <strong>{{ pendingAction.event.title }}</strong>
          {{ pendingAction.mode === 'trash' ? '之后仍可从回收站恢复。' : '删除后无法恢复，请确认不再需要这项任务。' }}
        </p>
        <footer>
          <button type="button" class="secondary" @click="closeActionPrompt">取消</button>
          <button
            type="button"
            :class="pendingAction.mode === 'trash' ? 'primary' : 'destructive'"
            :disabled="busyEventId === pendingAction.event.id"
            @click="confirmAction"
          >
            {{ busyEventId === pendingAction.event.id
              ? '正在处理…'
              : pendingAction.mode === 'trash' ? '移入回收站' : '彻底删除' }}
          </button>
        </footer>
      </section>
    </div>
  </div>
</template>

<style scoped>
.dashboard-shell {
  --page-bg: #f3f4ef;
  --sidebar-bg: #eceee7;
  --card-bg: #fffefa;
  --line: #dde0d7;
  --line-strong: #ccd0c6;
  --ink: #20231f;
  --muted: #6f756b;
  --accent: #d85b4b;
  --accent-soft: #f4ddd8;
  --green: #438b67;
  --amber: #bd873c;
  --shadow-soft: 0 1px 0 rgba(255, 255, 255, 0.8), 0 12px 32px rgba(54, 61, 49, 0.055);
  --shadow-raised: 0 18px 42px rgba(50, 56, 46, 0.09);
  display: grid;
  grid-template-columns: 236px minmax(0, 1fr);
  min-height: 100vh;
  color: var(--ink);
  background:
    radial-gradient(circle at 87% 8%, rgba(216, 91, 75, 0.035), transparent 23%),
    radial-gradient(circle at 70% 82%, rgba(67, 139, 103, 0.035), transparent 25%),
    var(--page-bg);
  font-family: "HarmonyOS Sans SC", "Microsoft YaHei", sans-serif;
}

.dashboard-shell button {
  font-family: inherit;
  transition: color 150ms ease, border-color 150ms ease, background 150ms ease,
    box-shadow 150ms ease, transform 150ms ease;
}

.dashboard-shell button:focus {
  outline: none;
}

.dashboard-shell button:focus-visible {
  outline: 2px solid rgba(216, 91, 75, 0.72);
  outline-offset: 2px;
}

.sidebar {
  position: sticky;
  top: 0;
  display: flex;
  flex-direction: column;
  height: 100vh;
  padding: 20px 14px 16px;
  border-right: 1px solid var(--line-strong);
  background:
    linear-gradient(180deg, rgba(255, 255, 255, 0.3), transparent 34%),
    var(--sidebar-bg);
}

.brand {
  display: flex;
  align-items: center;
  gap: 11px;
  min-height: 54px;
  padding: 0 8px 18px;
  border-bottom: 1px solid var(--line);
}

.mobile-switcher {
  display: none;
}

.brand-mark {
  display: grid;
  width: 38px;
  height: 38px;
  place-items: center;
  border-radius: 10px;
  color: #fff;
  background: var(--accent);
  box-shadow: 0 5px 14px rgba(183, 71, 56, 0.22);
  font-family: "STKaiti", "KaiTi", serif;
  font-size: 18px;
  font-weight: 700;
}

.brand div,
.sidebar-status div {
  display: flex;
  min-width: 0;
  flex-direction: column;
  gap: 2px;
}

.brand strong {
  font-size: 15px;
  letter-spacing: 0.04em;
}

.brand small {
  color: var(--muted);
  font: 600 9px/1.2 Georgia, serif;
  letter-spacing: 0.18em;
}

.sidebar-nav {
  flex: 1;
  overflow-y: auto;
  padding: 15px 0;
}

.nav-group + .nav-group {
  margin-top: 18px;
}

.nav-group > p {
  margin: 0 9px 6px;
  color: #9a9e95;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.14em;
}

.nav-group button {
  display: grid;
  grid-template-columns: 25px minmax(0, 1fr) auto;
  width: 100%;
  min-height: 38px;
  align-items: center;
  gap: 4px;
  padding: 0 10px;
  border: 0;
  border-radius: 8px;
  color: #4f544d;
  background: transparent;
  text-align: left;
  cursor: default;
  font-size: 13px;
}

.nav-group button:not(.planned) {
  cursor: pointer;
}

.nav-group button:disabled {
  opacity: 1;
}

.nav-group button.active {
  color: #292b27;
  background: rgba(255, 255, 252, 0.76);
  box-shadow: inset 3px 0 var(--accent), 0 6px 18px rgba(60, 65, 55, 0.055);
  font-weight: 700;
}

.nav-group button:not(.planned):not(.active):hover {
  color: var(--ink);
  background: rgba(225, 225, 218, 0.55);
  transform: translateX(2px);
}

.nav-group button.planned:not(.active):hover {
  background: rgba(225, 225, 218, 0.36);
}

.nav-icon {
  color: #6b7068;
  font-family: Georgia, serif;
  font-size: 17px;
}

.nav-group button small {
  color: #a2a69e;
  font-size: 9px;
}

.nav-group button .nav-count {
  min-width: 22px;
  padding: 3px 6px;
  border-radius: 999px;
  color: #8f4a40;
  background: var(--accent-soft);
  text-align: center;
  font: 700 9px/1 Georgia, serif;
}

.nav-group button .nav-count.inbox {
  color: #317756;
  background: #dceee4;
}

.sidebar-status {
  display: flex;
  align-items: center;
  gap: 9px;
  padding: 12px 7px 2px;
  border-top: 1px solid var(--line);
}

.status-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--green);
  box-shadow: 0 0 0 4px rgba(67, 139, 103, 0.12);
}

.sidebar-status strong {
  overflow: hidden;
  font-size: 10px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.sidebar-status small {
  color: var(--muted);
  font-size: 9px;
}

.workspace {
  min-width: 0;
  padding: 27px 32px 56px;
}

.workspace-header {
  margin-bottom: 22px;
}

.title-row {
  display: flex;
  align-items: center;
  gap: 9px;
}

.title-icon {
  color: var(--accent);
  font-size: 20px;
}

.title-row h1 {
  margin: 0;
  font-family: "STSong", "Songti SC", serif;
  font-size: 27px;
  font-weight: 700;
  letter-spacing: 0.01em;
}

.quick-add {
  margin-left: auto;
  padding: 8px 13px;
  border: 0;
  border-radius: 9px;
  color: #fff;
  background: var(--accent);
  box-shadow: 0 5px 14px rgba(183, 71, 56, 0.18);
  cursor: pointer;
  font-size: 11px;
  font-weight: 700;
}

.quick-add:hover {
  background: #c94e3f;
  transform: translateY(-1px);
}

.header-tag {
  padding: 4px 8px;
  border-radius: 5px;
  color: #9b4338;
  background: var(--accent-soft);
  font-size: 10px;
  font-weight: 700;
}

.header-tag.quiet {
  color: #667068;
  background: #e6e8e2;
}

.workspace-header > p {
  margin: 7px 0 0 29px;
  color: var(--muted);
  font-size: 12px;
  line-height: 1.65;
}

.calendar-panel {
  margin-bottom: 18px;
  overflow-x: auto;
  scrollbar-color: #c8cbc3 transparent;
  scrollbar-width: thin;
}

.calendar-toolbar {
  display: flex;
  min-height: 42px;
  align-items: center;
  gap: 10px;
}

.calendar-toolbar h2 {
  min-width: 210px;
  margin: 0;
  font-family: Georgia, "STSong", serif;
  font-size: 18px;
  white-space: nowrap;
}

.month-button {
  display: grid;
  width: 28px;
  height: 28px;
  place-items: center;
  border: 1px solid var(--line-strong);
  border-radius: 8px;
  color: #5f645c;
  background: #fafbf8;
  cursor: pointer;
}

.month-button:hover {
  color: var(--accent);
  border-color: #c8a59f;
}

.today-button {
  height: 28px;
  padding: 0 10px;
  border: 1px solid var(--line-strong);
  border-radius: 8px;
  color: #6c7169;
  background: transparent;
  cursor: pointer;
  font-size: 9px;
  font-weight: 700;
}

.today-button:hover {
  color: var(--accent);
  background: #fff;
}

.calendar-mode-switch {
  display: grid;
  grid-template-columns: repeat(2, 32px);
  padding: 3px;
  border: 1px solid var(--line);
  border-radius: 8px;
  background: #eceee8;
}

.calendar-mode-switch button {
  height: 22px;
  border: 0;
  border-radius: 5px;
  color: var(--muted);
  background: transparent;
  cursor: pointer;
  font-size: 9px;
  font-weight: 700;
}

.calendar-mode-switch button.active {
  color: #fff;
  background: var(--accent);
  box-shadow: 0 2px 7px rgba(129, 64, 53, 0.18);
}

.month-summary {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-left: auto;
  color: var(--muted);
  font-size: 10px;
}

.month-summary span + span::before {
  margin-right: 8px;
  color: #c6c9c2;
  content: "·";
}

.month-summary strong {
  color: var(--ink);
}

.month-summary .green { color: var(--green); }
.month-summary .red { color: var(--accent); }

.weekday-row,
.calendar-grid {
  display: grid;
  grid-template-columns: repeat(7, minmax(0, 1fr));
}

.weekday-row {
  height: 30px;
  align-items: center;
  color: #9a9f96;
  text-align: center;
  font-size: 10px;
}

.calendar-grid {
  overflow: hidden;
  border-top: 1px solid var(--line);
  border-left: 1px solid var(--line);
  border-radius: 13px;
  background: var(--card-bg);
  box-shadow: var(--shadow-soft);
}

.week-board {
  display: grid;
  min-width: 760px;
  grid-template-columns: repeat(7, minmax(0, 1fr));
  overflow: hidden;
  border: 1px solid var(--line);
  border-radius: 13px;
  background: var(--card-bg);
  box-shadow: var(--shadow-soft);
}

.week-day-card {
  position: relative;
  display: flex;
  min-width: 0;
  min-height: 226px;
  flex-direction: column;
  gap: 7px;
  padding: 14px 10px 12px;
  overflow: hidden;
  border: 0;
  border-right: 1px solid var(--line);
  color: var(--ink);
  background: var(--card-bg);
  text-align: left;
  cursor: pointer;
}

.week-day-card:last-child {
  border-right: 0;
}

.week-day-card:hover {
  z-index: 1;
  background: #fff;
  box-shadow: inset 0 -3px rgba(216, 91, 75, 0.08);
}

.week-day-card.selected {
  z-index: 2;
  background: #fffaf7;
  box-shadow: inset 0 0 0 2px rgba(216, 91, 75, 0.52);
}

.week-day-card::before {
  position: absolute;
  top: 0;
  right: 10px;
  left: 10px;
  height: 3px;
  border-radius: 0 0 999px 999px;
  background: transparent;
  content: "";
}

.week-day-card.status-active::before { background: var(--amber); }
.week-day-card.status-completed::before { background: var(--green); }
.week-day-card.status-overdue::before { background: var(--accent); }

.week-day-heading {
  display: grid;
  gap: 3px;
  padding-bottom: 9px;
  border-bottom: 1px solid var(--line);
}

.week-day-heading small {
  color: #9a9f96;
  font-size: 8px;
  letter-spacing: 0.08em;
}

.week-day-heading strong {
  font: 700 13px/1.2 Georgia, "STSong", serif;
}

.week-day-card.today .week-day-heading strong::after {
  display: inline-block;
  width: 5px;
  height: 5px;
  margin: 0 0 2px 5px;
  border-radius: 50%;
  background: var(--accent);
  content: "";
}

.week-empty {
  margin: auto 0;
  color: #b3b7af;
  text-align: center;
  font: italic 10px/1.4 Georgia, serif;
}

.week-event {
  position: relative;
  display: grid;
  gap: 2px;
  padding: 7px 7px 7px 9px;
  border-left: 2px solid var(--amber);
  border-radius: 0 6px 6px 0;
  background: #f3f4ef;
}

.week-event time {
  color: var(--muted);
  font-size: 8px;
}

.week-event strong {
  overflow: hidden;
  font-size: 9px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.week-event em {
  position: absolute;
  top: 6px;
  right: 6px;
  color: var(--accent);
  font-size: 5px;
  font-style: normal;
}

.week-more {
  margin-top: auto;
  color: var(--muted);
  font-size: 8px;
  text-align: right;
}

.day-cell {
  position: relative;
  display: flex;
  min-width: 0;
  height: 92px;
  flex-direction: column;
  align-items: stretch;
  padding: 9px 10px 8px;
  overflow: hidden;
  border: 0;
  border-right: 1px solid var(--line);
  border-bottom: 1px solid var(--line);
  color: var(--ink);
  background: var(--card-bg);
  text-align: left;
  cursor: pointer;
}

.day-cell:hover:not(.blank) {
  z-index: 1;
  background: #fff;
  box-shadow: inset 0 0 0 1px #cfd2cb;
  transform: translateY(-1px);
}

.day-cell.blank {
  cursor: default;
  background: #f7f8f4;
}

.day-cell.selected {
  z-index: 2;
  box-shadow: inset 0 0 0 2px rgba(216, 91, 75, 0.55), 0 5px 15px rgba(58, 59, 52, 0.08);
}

.day-cell.status-completed::after,
.day-cell.status-active::after,
.day-cell.status-overdue::after {
  position: absolute;
  right: 7px;
  bottom: 5px;
  left: 7px;
  height: 3px;
  border-radius: 999px;
  background: var(--green);
  content: "";
}

.day-cell.status-overdue::after { background: var(--accent); }
.day-cell.status-active::after { background: var(--amber); }

.day-number {
  font: 700 13px/1.2 Georgia, serif;
}

.day-cell.today .day-number::after {
  display: inline-block;
  width: 5px;
  height: 5px;
  margin: 0 0 2px 4px;
  border-radius: 50%;
  background: var(--accent);
  content: "";
}

.day-cell > small {
  position: absolute;
  top: 9px;
  right: 9px;
  color: #a2a69e;
  font-size: 9px;
}

.cell-title {
  margin-top: 13px;
  overflow: hidden;
  font-size: 11px;
  font-weight: 700;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.cell-title em {
  margin-left: 3px;
  color: var(--muted);
  font-size: 9px;
  font-style: normal;
}

.cell-status {
  margin-top: 4px;
  color: var(--muted);
  font-size: 9px;
}

.calendar-state {
  display: grid;
  height: 582px;
  place-items: center;
  border: 1px solid var(--line);
  border-radius: 10px;
  color: var(--muted);
  background: var(--card-bg);
  font-size: 13px;
}

.calendar-state.error {
  align-content: center;
  gap: 6px;
  color: var(--accent);
}

.calendar-state.error span {
  color: var(--muted);
}

.review-grid {
  display: grid;
  grid-template-columns: minmax(0, 1.65fr) minmax(280px, 0.85fr);
  gap: 14px;
  align-items: start;
}

.review-card {
  overflow: hidden;
  border: 1px solid var(--line);
  border-radius: 14px;
  background: var(--card-bg);
  box-shadow: var(--shadow-soft);
}

.card-header {
  display: flex;
  min-height: 50px;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 0 16px;
  border-bottom: 1px solid var(--line);
}

.card-header > div {
  display: flex;
  align-items: center;
  gap: 8px;
}

.card-header strong {
  font-size: 12px;
}

.card-header > small,
.card-header > span {
  color: var(--muted);
  font-size: 9px;
}

.card-header.compact {
  min-height: 45px;
}

.review-pill {
  padding: 4px 7px;
  border-radius: 999px;
  color: #317756;
  background: #dceee4;
  font-size: 9px;
  font-weight: 700;
}

.review-summary {
  margin: 0;
  padding: 17px 18px;
  border-bottom: 1px solid var(--line);
  font-family: "STSong", "Songti SC", serif;
  font-size: 14px;
  font-weight: 600;
  line-height: 1.75;
}

.review-section {
  padding: 15px 18px 17px;
}

.review-section h3 {
  margin: 0 0 12px;
  font-size: 12px;
}

.review-list,
.suggestion-list {
  margin: 0;
  padding: 0;
  list-style: none;
}

.review-list li {
  display: grid;
  grid-template-columns: 12px minmax(0, 1fr) auto;
  gap: 9px;
  align-items: start;
  padding: 10px 0;
  border-top: 1px dashed var(--line);
}

.task-dot {
  width: 8px;
  height: 8px;
  margin-top: 4px;
  border: 2px solid var(--amber);
  border-radius: 50%;
}

.task-dot.done {
  border-color: var(--green);
  background: var(--green);
}

.review-list strong {
  font-size: 11px;
}

.review-list p {
  margin: 3px 0 0;
  color: var(--muted);
  font-size: 10px;
  line-height: 1.55;
}

.review-list time {
  color: #a2a69e;
  font-size: 9px;
}

.task-actions {
  display: flex;
  align-items: center;
  gap: 6px;
}

.task-actions button {
  padding: 4px 7px;
  border: 1px solid var(--line-strong);
  border-radius: 5px;
  color: #60655d;
  background: #f8f9f5;
  cursor: pointer;
  font-size: 9px;
}

.task-actions button:hover {
  color: var(--accent);
  border-color: #d6a8a0;
  background: #fff;
}

.task-actions button:disabled {
  cursor: wait;
  opacity: 0.5;
}

.task-actions button.danger {
  color: #9f5146;
  border-color: #e1c1bb;
  background: #fff8f6;
}

.home-header {
  margin-bottom: 18px;
}

.home-error {
  display: flex;
  gap: 8px;
  margin-bottom: 14px;
  padding: 10px 13px;
  border: 1px solid #e5c5bf;
  border-radius: 8px;
  color: #9d4439;
  background: #fff6f3;
  font-size: 10px;
}

.home-error span {
  color: var(--muted);
}

.home-hero {
  position: relative;
  display: grid;
  grid-template-columns: 140px minmax(280px, 1fr) 240px;
  gap: 24px;
  overflow: hidden;
  margin-bottom: 18px;
  padding: 25px 27px;
  border: 1px solid #d9ddd5;
  border-radius: 16px;
  background:
    radial-gradient(circle at 7% 12%, rgba(216, 91, 75, 0.13), transparent 28%),
    linear-gradient(120deg, #fffefa, #f3f6ef);
  box-shadow: var(--shadow-raised);
}

.home-hero::before {
  position: absolute;
  top: 24px;
  bottom: 24px;
  left: 0;
  width: 3px;
  border-radius: 0 999px 999px 0;
  background: var(--accent);
  content: "";
}

.home-hero::after {
  position: absolute;
  right: -28px;
  bottom: -54px;
  width: 180px;
  height: 180px;
  border: 1px solid rgba(67, 139, 103, 0.13);
  border-radius: 50%;
  content: "";
}

.today-date-block {
  display: flex;
  flex-direction: column;
  justify-content: center;
  padding-right: 22px;
  border-right: 1px solid var(--line-strong);
}

.today-date-block small,
.home-kicker,
.home-panel-header small {
  color: var(--accent);
  font: 700 9px/1.2 Georgia, serif;
  letter-spacing: 0.18em;
}

.today-date-block strong {
  margin: 4px 0 1px;
  font: 400 62px/0.96 Georgia, "STSong", serif;
  letter-spacing: -0.06em;
}

.today-date-block span {
  color: var(--muted);
  font-size: 9px;
  line-height: 1.5;
}

.home-lead {
  align-self: center;
}

.home-lead h2 {
  margin: 7px 0 7px;
  font-family: "STSong", "Songti SC", serif;
  font-size: 25px;
  letter-spacing: 0.02em;
}

.home-lead > p {
  margin: 0;
  color: #60665d;
  font-size: 11px;
}

.home-quick-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 7px;
  margin-top: 17px;
}

.home-quick-actions button {
  height: 32px;
  padding: 0 11px;
  border: 1px solid var(--line-strong);
  border-radius: 7px;
  color: #656a62;
  background: rgba(255, 255, 255, 0.62);
  cursor: pointer;
  font-size: 9px;
  font-weight: 700;
}

.home-quick-actions button:hover {
  color: var(--accent);
  border-color: #d9aaa3;
  background: #fff;
}

.home-quick-actions button.primary {
  color: #fff;
  border-color: var(--accent);
  background: var(--accent);
  box-shadow: 0 5px 13px rgba(183, 71, 56, 0.16);
}

.home-stats {
  position: relative;
  z-index: 1;
  display: grid;
  gap: 7px;
  align-self: center;
}

.home-stats > div {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 35px;
  align-items: center;
  padding: 6px 9px;
  border-bottom: 1px solid rgba(211, 214, 206, 0.8);
}

.home-stats > div:last-child {
  border-bottom: 0;
}

.home-stats small {
  color: #60665d;
  font-size: 9px;
  font-weight: 700;
}

.home-stats strong {
  grid-row: span 2;
  text-align: right;
  font: 600 24px/1 Georgia, serif;
}

.home-stats span {
  color: #9b9f97;
  font-size: 8px;
}

.home-stats .red {
  color: var(--accent);
}

.home-stats .green {
  color: var(--green);
}

.home-grid {
  display: grid;
  grid-template-columns: minmax(0, 1.55fr) minmax(285px, 0.85fr);
  gap: 14px;
  align-items: start;
}

.home-panel {
  overflow: hidden;
  border: 1px solid var(--line);
  border-radius: 14px;
  background: var(--card-bg);
  box-shadow: var(--shadow-soft);
}

.home-panel-header {
  display: flex;
  min-height: 58px;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 0 17px;
  border-bottom: 1px solid var(--line);
  background: linear-gradient(180deg, #fbfcf8, #f7f8f3);
}

.home-panel-header.compact {
  min-height: 52px;
}

.home-panel-header h2 {
  margin: 4px 0 0;
  font-family: "STSong", "Songti SC", serif;
  font-size: 16px;
}

.home-panel-header > span {
  color: var(--muted);
  font-size: 9px;
}

.home-panel-header > button {
  border: 0;
  color: var(--green);
  background: transparent;
  cursor: pointer;
  font-size: 9px;
  font-weight: 700;
}

.home-task-list {
  margin: 0;
  padding: 6px 18px 10px;
  list-style: none;
}

.home-task-list li {
  display: grid;
  grid-template-columns: 48px 12px minmax(0, 1fr) auto;
  gap: 9px;
  align-items: center;
  min-height: 69px;
  border-bottom: 1px dashed var(--line);
}

.home-task-list li:last-child {
  border-bottom: 0;
}

.home-task-list li.completed {
  opacity: 0.58;
}

.home-task-list > li > time {
  color: #50564e;
  font: 600 11px/1 Georgia, serif;
}

.agenda-line {
  position: relative;
  width: 9px;
  height: 9px;
  border: 2px solid var(--amber);
  border-radius: 50%;
}

.agenda-line::after {
  position: absolute;
  top: 10px;
  left: 2px;
  width: 1px;
  height: 44px;
  background: var(--line);
  content: "";
}

.home-task-list li:last-child .agenda-line::after {
  display: none;
}

.completed .agenda-line {
  border-color: var(--green);
  background: var(--green);
}

.agenda-copy {
  min-width: 0;
}

.agenda-copy strong {
  display: block;
  overflow: hidden;
  font-size: 11px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.agenda-copy p {
  overflow: hidden;
  margin: 4px 0 0;
  color: var(--muted);
  font-size: 9px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.deadline-note {
  display: block;
  margin-top: 4px;
  color: var(--accent);
  font-size: 8px;
  font-weight: 700;
}

.agenda-actions {
  display: flex;
  gap: 5px;
}

.agenda-actions button {
  padding: 4px 7px;
  border: 1px solid var(--line-strong);
  border-radius: 5px;
  color: #656a62;
  background: #f8f9f5;
  cursor: pointer;
  font-size: 9px;
}

.agenda-actions button:hover {
  color: var(--accent);
  border-color: #d6a8a0;
  background: #fff;
}

.agenda-actions button.danger {
  color: #a05247;
}

.agenda-actions button:disabled {
  cursor: wait;
  opacity: 0.5;
}

.home-side-stack {
  display: grid;
  gap: 14px;
}

.overdue-list,
.home-inbox-list {
  margin: 0;
  padding: 6px 14px 10px;
  list-style: none;
}

.overdue-list li {
  display: grid;
  grid-template-columns: 34px minmax(0, 1fr) auto;
  gap: 8px;
  align-items: center;
  min-height: 52px;
  border-bottom: 1px solid var(--line);
}

.overdue-list li:last-child,
.home-inbox-list li:last-child {
  border-bottom: 0;
}

.overdue-list time {
  color: var(--accent);
  font: 600 9px/1 Georgia, serif;
}

.overdue-list div {
  min-width: 0;
}

.overdue-list strong {
  display: block;
  overflow: hidden;
  font-size: 10px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.overdue-list span {
  color: #9ca097;
  font-size: 8px;
}

.overdue-list button,
.home-inbox-list button {
  padding: 4px 6px;
  border: 1px solid var(--line-strong);
  border-radius: 5px;
  color: #747970;
  background: transparent;
  cursor: pointer;
  font-size: 8px;
}

.panel-link {
  width: 100%;
  padding: 8px 14px 11px;
  border: 0;
  color: var(--accent);
  background: transparent;
  text-align: right;
  cursor: pointer;
  font-size: 8px;
}

.home-inbox-list li {
  display: grid;
  grid-template-columns: 18px minmax(0, 1fr) auto;
  gap: 7px;
  align-items: center;
  min-height: 42px;
  border-bottom: 1px solid var(--line);
}

.home-inbox-list li > span {
  color: var(--green);
}

.home-inbox-list strong {
  overflow: hidden;
  font-size: 10px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.side-empty,
.home-empty {
  color: var(--muted);
  text-align: center;
  font-size: 9px;
}

.side-empty {
  padding: 24px 14px;
}

.home-empty {
  display: grid;
  min-height: 250px;
  place-content: center;
  gap: 7px;
  padding: 24px;
}

.home-empty > span {
  color: #a9afa6;
  font: 400 38px/1 Georgia, serif;
}

.home-empty strong {
  color: #555b53;
  font-family: "STSong", "Songti SC", serif;
  font-size: 14px;
}

.home-empty p {
  margin: 0;
}

.home-empty button {
  width: max-content;
  margin: 5px auto 0;
  padding: 7px 10px;
  border: 0;
  border-radius: 6px;
  color: #fff;
  background: var(--accent);
  cursor: pointer;
  font-size: 9px;
  font-weight: 700;
}

.inbox-header {
  margin-bottom: 22px;
}

.header-tag.inbox-tag {
  color: #317756;
  background: #dceee4;
}

.inbox-capture {
  display: grid;
  grid-template-columns: minmax(220px, 0.72fr) minmax(420px, 1.5fr);
  gap: 30px;
  margin-bottom: 18px;
  padding: 24px 26px;
  border: 1px solid #d7ded5;
  border-radius: 16px;
  background:
    radial-gradient(circle at 8% 20%, rgba(67, 139, 103, 0.12), transparent 28%),
    #f9faf6;
  box-shadow: var(--shadow-raised);
}

.capture-intro small,
.inbox-panel-heading small {
  color: var(--green);
  font: 700 9px/1.2 Georgia, serif;
  letter-spacing: 0.18em;
}

.capture-intro h2,
.inbox-panel-heading h2 {
  margin: 6px 0 8px;
  font-family: "STSong", "Songti SC", serif;
  font-size: 20px;
}

.capture-intro p {
  max-width: 250px;
  margin: 0;
  color: var(--muted);
  font-size: 10px;
  line-height: 1.7;
}

.capture-form {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 0.85fr) auto;
  gap: 12px;
  align-items: end;
}

.capture-form label {
  display: grid;
  gap: 7px;
}

.capture-form label > span {
  color: #555c53;
  font-size: 10px;
  font-weight: 700;
}

.capture-form label em {
  color: #9ba097;
  font-size: 9px;
  font-style: normal;
  font-weight: 400;
}

.capture-form input,
.capture-form textarea {
  width: 100%;
  border: 1px solid var(--line-strong);
  border-radius: 8px;
  outline: none;
  color: var(--ink);
  background: rgba(255, 255, 252, 0.94);
  font: 11px/1.45 "HarmonyOS Sans SC", "Microsoft YaHei", sans-serif;
}

.capture-form input {
  height: 42px;
  padding: 0 11px;
}

.capture-form textarea {
  height: 42px;
  min-height: 42px;
  padding: 11px;
  resize: vertical;
}

.capture-form input:focus,
.capture-form textarea:focus {
  border-color: #78a98c;
  box-shadow: 0 0 0 3px rgba(67, 139, 103, 0.1);
}

.capture-submit {
  position: relative;
}

.capture-submit p {
  position: absolute;
  right: 0;
  bottom: 47px;
  width: 180px;
  margin: 0;
  color: #a3463c;
  font-size: 9px;
  text-align: right;
}

.capture-submit button {
  min-width: 126px;
  height: 42px;
  border: 0;
  border-radius: 9px;
  color: #fff;
  background: var(--green);
  box-shadow: 0 6px 16px rgba(67, 139, 103, 0.18);
  cursor: pointer;
  font-size: 10px;
  font-weight: 700;
}

.capture-submit button:hover {
  background: #367b59;
  transform: translateY(-1px);
}

.capture-submit button:disabled {
  cursor: wait;
  opacity: 0.62;
}

.inbox-panel {
  overflow: hidden;
  border: 1px solid var(--line);
  border-radius: 14px;
  background: var(--card-bg);
  box-shadow: var(--shadow-soft);
}

.inbox-panel-heading {
  display: flex;
  min-height: 72px;
  align-items: center;
  justify-content: space-between;
  gap: 24px;
  padding: 14px 22px;
  border-bottom: 1px solid var(--line);
  background: #f8f9f5;
}

.inbox-panel-heading h2 {
  margin-bottom: 0;
  font-size: 17px;
}

.inbox-panel-heading p {
  max-width: 380px;
  margin: 0;
  color: var(--muted);
  font-size: 9px;
  line-height: 1.6;
  text-align: right;
}

.inbox-list {
  margin: 0;
  padding: 0 22px;
  list-style: none;
}

.inbox-list li {
  display: grid;
  grid-template-columns: 34px minmax(0, 1fr) auto;
  gap: 14px;
  align-items: center;
  min-height: 92px;
  padding: 15px 0;
  border-bottom: 1px solid var(--line);
}

.inbox-list li:last-child {
  border-bottom: 0;
}

.inbox-sequence {
  color: #9ca79e;
  font: 600 12px/1 Georgia, serif;
}

.inbox-copy {
  min-width: 0;
}

.inbox-copy strong {
  display: block;
  overflow: hidden;
  font-family: "STSong", "Songti SC", serif;
  font-size: 14px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.inbox-copy p {
  overflow: hidden;
  margin: 4px 0 6px;
  color: var(--muted);
  font-size: 10px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.inbox-copy span {
  color: #9ca198;
  font-size: 9px;
}

.inbox-actions {
  display: flex;
  gap: 7px;
}

.inbox-actions button {
  min-width: 70px;
  height: 32px;
  border-radius: 7px;
  cursor: pointer;
  font-size: 10px;
  font-weight: 700;
}

.inbox-actions .schedule {
  border: 1px solid #a8c7b5;
  color: #347355;
  background: #eef7f1;
}

.inbox-actions .remove {
  border: 1px solid var(--line-strong);
  color: #777c74;
  background: transparent;
}

.inbox-empty {
  display: grid;
  min-height: 260px;
  place-content: center;
  gap: 8px;
  color: var(--muted);
  text-align: center;
}

.inbox-empty > span {
  display: grid;
  width: 48px;
  height: 48px;
  margin: 0 auto 4px;
  place-items: center;
  border-radius: 50%;
  color: var(--green);
  background: #e5f2e9;
  font: 700 22px/1 Georgia, serif;
}

.inbox-empty strong {
  color: #555b53;
  font-family: "STSong", "Songti SC", serif;
  font-size: 15px;
}

.inbox-empty p {
  margin: 0;
  font-size: 10px;
}

.trash-header {
  margin-bottom: 26px;
}

.trash-count {
  margin-left: auto;
  color: var(--muted);
  font: 600 11px/1 Georgia, serif;
}

.trash-panel {
  overflow: hidden;
  border: 1px solid var(--line);
  border-radius: 14px;
  background: var(--card-bg);
  box-shadow: var(--shadow-soft);
}

.trash-panel-heading {
  display: flex;
  min-height: 88px;
  align-items: center;
  justify-content: space-between;
  gap: 24px;
  padding: 18px 22px;
  border-bottom: 1px solid var(--line);
  background:
    linear-gradient(110deg, rgba(216, 91, 75, 0.08), transparent 38%),
    #f8f9f5;
}

.trash-panel-heading small {
  color: var(--accent);
  font: 700 9px/1.2 Georgia, serif;
  letter-spacing: 0.18em;
}

.trash-panel-heading h2 {
  margin: 5px 0 0;
  font-family: "STSong", "Songti SC", serif;
  font-size: 19px;
}

.trash-panel-heading p {
  max-width: 380px;
  margin: 0;
  color: var(--muted);
  font-size: 10px;
  line-height: 1.65;
  text-align: right;
}

.trash-list {
  margin: 0;
  padding: 0 22px;
  list-style: none;
}

.trash-list li {
  display: grid;
  grid-template-columns: 138px minmax(0, 1fr) auto;
  gap: 22px;
  align-items: center;
  min-height: 104px;
  padding: 18px 0;
  border-bottom: 1px solid var(--line);
}

.trash-list li:last-child {
  border-bottom: 0;
}

.trash-date {
  display: grid;
  gap: 5px;
  padding-left: 13px;
  border-left: 3px solid #d8b3ac;
}

.trash-date small,
.trash-copy span {
  color: #999e95;
  font-size: 9px;
}

.trash-date strong {
  color: #696e66;
  font: 600 10px/1.4 Georgia, "Microsoft YaHei", sans-serif;
}

.trash-copy {
  min-width: 0;
}

.trash-copy > strong {
  display: block;
  overflow: hidden;
  font-family: "STSong", "Songti SC", serif;
  font-size: 14px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.trash-copy p {
  overflow: hidden;
  margin: 5px 0 7px;
  color: var(--muted);
  font-size: 10px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.trash-actions {
  display: flex;
  gap: 7px;
}

.trash-actions button {
  min-width: 76px;
  height: 32px;
  border-radius: 7px;
  cursor: pointer;
  font-size: 10px;
  font-weight: 700;
}

.trash-actions button:disabled {
  cursor: wait;
  opacity: 0.55;
}

.trash-actions .restore {
  border: 1px solid #a8c7b5;
  color: #347355;
  background: #eef7f1;
}

.trash-actions .purge {
  border: 1px solid #dfc2bd;
  color: #9d4a3f;
  background: transparent;
}

.trash-empty {
  display: grid;
  min-height: 285px;
  place-content: center;
  gap: 8px;
  padding: 30px;
  color: var(--muted);
  text-align: center;
}

.trash-empty .empty-bin {
  color: #c9a69f;
  font: 400 52px/1 Georgia, serif;
}

.trash-empty strong {
  color: #575c54;
  font-family: "STSong", "Songti SC", serif;
  font-size: 16px;
}

.trash-empty p,
.trash-empty span {
  margin: 0;
  font-size: 10px;
}

.trash-empty.error strong {
  color: var(--accent);
}

.reserved-copy {
  padding: 12px;
  border: 1px dashed #cfd2ca;
  border-radius: 8px;
  color: var(--muted);
  background: #f7f8f4;
  text-align: center;
}

.reserved-copy span {
  color: #5e635b;
  font-size: 11px;
  font-weight: 700;
}

.reserved-copy p {
  margin: 5px 0 0;
  font-size: 9px;
}

.review-side {
  display: grid;
  gap: 14px;
}

.suggestion-list {
  padding: 8px 15px 13px;
}

.suggestion-list li {
  display: grid;
  grid-template-columns: 28px minmax(0, 1fr);
  gap: 8px;
  align-items: center;
  padding: 10px 0;
  border-bottom: 1px solid var(--line);
}

.suggestion-list li:last-child {
  border-bottom: 0;
}

.suggestion-list li > span {
  padding: 3px 0;
  border-radius: 4px;
  color: #9b4338;
  background: var(--accent-soft);
  text-align: center;
  font: 700 9px/1.2 Georgia, serif;
}

.suggestion-list div {
  display: flex;
  min-width: 0;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.suggestion-list strong {
  overflow: hidden;
  font-size: 10px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.suggestion-list small {
  color: var(--muted);
  font-size: 9px;
}

.reserved-modules {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 7px;
  padding: 14px 14px 8px;
}

.reserved-modules span {
  padding: 13px 4px;
  border: 1px dashed #ccd0c8;
  border-radius: 7px;
  color: #858a81;
  background: #f7f8f4;
  text-align: center;
  font-size: 9px;
}

.reserved-card > p,
.empty-copy {
  margin: 0;
  padding: 2px 15px 15px;
  color: var(--muted);
  font-size: 9px;
  line-height: 1.5;
}

.modal-overlay {
  position: fixed;
  z-index: 50;
  inset: 0;
  display: grid;
  place-items: center;
  padding: 20px;
  background: rgba(34, 36, 32, 0.38);
  backdrop-filter: blur(7px);
}

.task-modal {
  width: min(460px, 100%);
  max-height: calc(100vh - 40px);
  padding: 20px;
  overflow-y: auto;
  border: 1px solid var(--line-strong);
  border-radius: 16px;
  background: #fffefa;
  box-shadow: 0 28px 80px rgba(36, 40, 32, 0.24);
  scrollbar-color: #c8cbc3 transparent;
  scrollbar-width: thin;
}

.task-modal > header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  margin-bottom: 20px;
}

.task-modal > header small {
  color: var(--accent);
  font: 700 9px/1.2 Georgia, serif;
  letter-spacing: 0.16em;
}

.task-modal > header h2 {
  margin: 4px 0 0;
  font-family: "STSong", "Songti SC", serif;
  font-size: 22px;
}

.task-modal > header button {
  width: 30px;
  height: 30px;
  border: 0;
  border-radius: 8px;
  color: var(--muted);
  background: #eceee8;
  cursor: pointer;
  font-size: 20px;
}

.task-modal > header button:hover {
  color: var(--accent);
  background: var(--accent-soft);
}

.task-modal > label,
.form-row label {
  display: grid;
  gap: 7px;
  margin-bottom: 14px;
}

.task-modal label > span {
  color: #565b53;
  font-size: 10px;
  font-weight: 700;
}

.task-modal label em {
  color: #a2a69e;
  font-size: 9px;
  font-style: normal;
  font-weight: 400;
}

.task-modal input,
.task-modal textarea {
  width: 100%;
  border: 1px solid var(--line-strong);
  border-radius: 8px;
  outline: none;
  color: var(--ink);
  background: #fffefa;
  font: 12px/1.5 "HarmonyOS Sans SC", "Microsoft YaHei", sans-serif;
}

.task-modal input {
  height: 40px;
  padding: 0 11px;
}

.task-modal textarea {
  resize: vertical;
  padding: 10px 11px;
}

.task-modal input:focus,
.task-modal textarea:focus {
  border-color: #c98277;
  box-shadow: 0 0 0 3px rgba(216, 91, 75, 0.1);
}

.form-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}

.form-section-title {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin: 4px 0 10px;
  padding-top: 12px;
  border-top: 1px solid var(--line);
}

.form-section-title span {
  color: #565b53;
  font-size: 10px;
  font-weight: 700;
}

.form-section-title small {
  color: var(--muted);
  font-size: 9px;
}

.form-row.schedule-row {
  grid-template-columns: 1.2fr 0.8fr 0.8fr;
}

.deadline-toggle {
  display: grid !important;
  grid-template-columns: 16px auto minmax(0, 1fr) !important;
  align-items: center;
  gap: 8px !important;
  margin: 2px 0 14px !important;
}

.deadline-toggle input {
  width: 15px;
  height: 15px;
  padding: 0;
  accent-color: var(--accent);
}

.deadline-toggle small {
  color: var(--muted);
  font-size: 9px;
}

.deadline-row {
  margin: -3px 0 14px;
  padding: 12px;
  border: 1px solid var(--line);
  border-radius: 8px;
  background: #f3f6f1;
}

.deadline-row label {
  margin-bottom: 0;
}

.reminder-check {
  display: flex !important;
  grid-template-columns: none !important;
  align-items: center;
  gap: 8px !important;
}

.reminder-check input {
  width: 15px;
  height: 15px;
  accent-color: var(--accent);
}

.form-error {
  margin: 0 0 12px;
  color: #b33e31;
  font-size: 10px;
}

.task-modal > footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding-top: 5px;
}

.task-modal > footer button {
  min-width: 86px;
  height: 36px;
  border-radius: 8px;
  cursor: pointer;
  font-size: 11px;
  font-weight: 700;
}

.task-modal .secondary {
  border: 1px solid var(--line-strong);
  color: #5f645c;
  background: transparent;
}

.task-modal .secondary:hover,
.confirm-modal .secondary:hover {
  color: var(--accent);
  border-color: #d6a8a0;
  background: #fff8f6;
}

.task-modal .primary {
  border: 0;
  color: #fff;
  background: var(--accent);
}

.task-modal .primary:hover,
.confirm-modal .primary:hover {
  background: #c94e3f;
  transform: translateY(-1px);
}

.task-modal .primary:disabled {
  cursor: wait;
  opacity: 0.65;
}

.confirm-modal {
  width: min(400px, 100%);
  padding: 24px;
  border: 1px solid var(--line-strong);
  border-radius: 16px;
  background: #fffefa;
  box-shadow: 0 28px 80px rgba(36, 40, 32, 0.24);
  text-align: center;
}

.confirm-mark {
  display: grid;
  width: 48px;
  height: 48px;
  margin: 0 auto 13px;
  place-items: center;
  border-radius: 50%;
  color: #a34e43;
  background: var(--accent-soft);
  font: 700 23px/1 Georgia, serif;
}

.confirm-modal > small {
  color: var(--accent);
  font: 700 9px/1.2 Georgia, serif;
  letter-spacing: 0.16em;
}

.confirm-modal h2 {
  margin: 6px 0 10px;
  font-family: "STSong", "Songti SC", serif;
  font-size: 21px;
}

.confirm-modal p {
  margin: 0;
  color: var(--muted);
  font-size: 11px;
  line-height: 1.7;
}

.confirm-modal p strong {
  display: block;
  overflow: hidden;
  margin-bottom: 2px;
  color: var(--ink);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.confirm-modal footer {
  display: flex;
  justify-content: center;
  gap: 8px;
  margin-top: 21px;
}

.confirm-modal footer button {
  min-width: 104px;
  height: 36px;
  border-radius: 8px;
  cursor: pointer;
  font-size: 11px;
  font-weight: 700;
}

.confirm-modal .secondary {
  border: 1px solid var(--line-strong);
  color: #5f645c;
  background: transparent;
}

.confirm-modal .primary,
.confirm-modal .destructive {
  border: 0;
  color: #fff;
  background: var(--accent);
}

.confirm-modal .destructive {
  background: #9f392f;
}

.confirm-modal button:disabled {
  cursor: wait;
  opacity: 0.62;
}

.workspace > * {
  animation: workbench-enter 260ms ease both;
}

@keyframes workbench-enter {
  from {
    opacity: 0;
    transform: translateY(4px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@media (prefers-reduced-motion: reduce) {
  .dashboard-shell *,
  .dashboard-shell *::before,
  .dashboard-shell *::after {
    scroll-behavior: auto !important;
    transition-duration: 0.01ms !important;
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
  }
}

@media (max-width: 980px) {
  .dashboard-shell {
    grid-template-columns: 192px minmax(0, 1fr);
  }

  .workspace {
    padding-right: 18px;
    padding-left: 18px;
  }

  .review-grid {
    grid-template-columns: 1fr;
  }

  .home-hero {
    grid-template-columns: 120px minmax(0, 1fr);
  }

  .home-stats {
    grid-column: 1 / -1;
    grid-template-columns: repeat(3, 1fr);
    border-top: 1px solid var(--line);
    padding-top: 8px;
  }

  .home-stats > div {
    border-right: 1px solid var(--line);
    border-bottom: 0;
  }

  .home-stats > div:last-child {
    border-right: 0;
  }

  .home-grid {
    grid-template-columns: 1fr;
  }

  .inbox-capture {
    grid-template-columns: 1fr;
    gap: 18px;
  }

  .day-cell {
    height: 82px;
  }
}

@media (max-width: 720px) {
  .dashboard-shell {
    display: block;
  }

  .sidebar {
    position: sticky;
    z-index: 20;
    top: 0;
    width: 100%;
    height: auto;
    padding: 12px 14px 10px;
    border-right: 0;
    border-bottom: 1px solid rgba(204, 208, 198, 0.88);
    background: rgba(238, 239, 233, 0.92);
    box-shadow: 0 8px 24px rgba(51, 57, 47, 0.07);
    backdrop-filter: blur(14px);
  }

  .sidebar-nav,
  .sidebar-status {
    display: none;
  }

  .mobile-switcher {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 5px;
    padding: 3px 0 0;
  }

  .mobile-switcher button {
    height: 32px;
    border: 1px solid var(--line);
    border-radius: 7px;
    color: var(--muted);
    background: #f7f8f4;
    font-size: 10px;
    cursor: pointer;
  }

  .mobile-switcher button.active {
    color: #fff;
    border-color: var(--accent);
    background: var(--accent);
    box-shadow: 0 4px 12px rgba(183, 71, 56, 0.17);
  }

  .mobile-switcher button:focus-visible {
    outline: 0;
    box-shadow: inset 0 0 0 2px rgba(216, 91, 75, 0.35);
  }

  .mobile-switcher button.active:focus-visible {
    box-shadow: inset 0 0 0 2px rgba(255, 255, 255, 0.46), 0 4px 12px rgba(183, 71, 56, 0.17);
  }

  .brand {
    min-height: 43px;
    padding-bottom: 8px;
    border-bottom: 0;
  }

  .brand-mark {
    width: 34px;
    height: 34px;
    font-size: 16px;
  }

  .workspace {
    padding: 21px 14px 40px;
  }

  .home-hero {
    grid-template-columns: 112px minmax(0, 1fr);
    gap: 18px;
    padding: 21px 20px;
  }

  .today-date-block {
    padding: 0 16px 0 0;
    border-right: 1px solid var(--line);
    border-bottom: 0;
  }

  .today-date-block span {
    font-size: 8px;
    white-space: nowrap;
  }

  .today-date-block strong {
    font-size: 48px;
  }

  .home-stats {
    grid-column: 1 / -1;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 0;
    padding-top: 10px;
  }

  .home-stats > div {
    grid-template-columns: minmax(0, 1fr) 28px;
    padding: 7px 9px;
    border-right: 1px solid var(--line);
    border-bottom: 0;
  }

  .home-stats > div:last-child {
    border-right: 0;
  }

  .home-stats strong {
    font-size: 21px;
  }

  .home-task-list li {
    grid-template-columns: 42px 12px minmax(0, 1fr);
    padding: 10px 0;
  }

  .agenda-actions {
    grid-column: 3;
  }

  .header-tag,
  .month-summary {
    display: none;
  }

  .workspace-header > p {
    margin-left: 0;
    line-height: 1.6;
  }

  .calendar-panel {
    overflow-x: auto;
  }

  .weekday-row,
  .calendar-grid {
    min-width: 680px;
  }

  .capture-form {
    grid-template-columns: 1fr;
  }

  .capture-submit p {
    position: static;
    width: auto;
    margin: 0 0 7px;
    text-align: left;
  }

  .capture-submit button {
    width: 100%;
  }

  .inbox-panel-heading,
  .trash-panel-heading {
    align-items: flex-start;
    flex-direction: column;
  }

  .inbox-panel-heading p,
  .trash-panel-heading p {
    text-align: left;
  }

  .inbox-list li,
  .trash-list li {
    grid-template-columns: 28px minmax(0, 1fr);
  }

  .inbox-actions,
  .trash-actions {
    grid-column: 2;
  }

  .form-row.schedule-row {
    grid-template-columns: 1fr 1fr;
  }

  .schedule-row label:first-child {
    grid-column: 1 / -1;
  }

  .inbox-capture {
    padding: 21px 20px;
  }

  .inbox-empty,
  .trash-empty {
    min-height: 220px;
  }

  .task-modal {
    padding: 18px;
  }
}

@media (max-width: 480px) {
  .workspace {
    padding-right: 11px;
    padding-left: 11px;
  }

  .home-hero {
    grid-template-columns: 1fr;
  }

  .today-date-block {
    padding: 0 0 13px;
    border-right: 0;
    border-bottom: 1px solid var(--line);
  }

  .home-stats small {
    font-size: 8px;
  }

  .home-stats span {
    display: none;
  }

  .title-row {
    flex-wrap: wrap;
  }

  .quick-add {
    margin-left: 0;
  }
}
</style>
