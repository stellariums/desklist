<script setup lang="ts">
import { ref, watch } from 'vue';
import type { DeskEvent } from '../types';
import ReminderConfig from './ReminderConfig.vue';

const props = defineProps<{
  visible: boolean;
  editEvent: DeskEvent | null;
}>();

const emit = defineEmits<{
  close: [];
  save: [data: Omit<DeskEvent, 'id' | 'created_at' | 'updated_at'>];
  update: [id: string, data: Partial<DeskEvent>];
}>();

const title = ref('');
const description = ref('');
const eventTime = ref('');
const remindOnTime = ref(1);
const advanceMinutes = ref(0);
const recurrence = ref('none');
const recurrenceEnd = ref('');

watch(() => props.visible, (val) => {
  if (val && props.editEvent) {
    title.value = props.editEvent.title;
    description.value = props.editEvent.description;
    eventTime.value = toLocalDatetime(props.editEvent.event_time);
    remindOnTime.value = props.editEvent.remind_on_time;
    advanceMinutes.value = calcAdvanceMinutes(props.editEvent.event_time, props.editEvent.remind_at);
    recurrence.value = props.editEvent.recurrence;
    recurrenceEnd.value = props.editEvent.recurrence_end ? toLocalDatetime(props.editEvent.recurrence_end) : '';
  } else if (val) {
    resetForm();
  }
});

function resetForm() {
  title.value = '';
  description.value = '';
  const now = new Date();
  now.setMinutes(now.getMinutes() + 30);
  now.setSeconds(0, 0);
  eventTime.value = toLocalDatetime(now.toISOString());
  remindOnTime.value = 1;
  advanceMinutes.value = 0;
  recurrence.value = 'none';
  recurrenceEnd.value = '';
}

function toLocalDatetime(iso: string): string {
  const d = new Date(iso);
  const pad = (n: number) => n.toString().padStart(2, '0');
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

function calcAdvanceMinutes(eventTime: string, remindAt: string | null): number {
  if (!remindAt) return 0;
  const diff = (new Date(eventTime).getTime() - new Date(remindAt).getTime()) / 60000;
  const options = [5, 15, 30, 60, 1440];
  return options.includes(Math.round(diff)) ? Math.round(diff) : 0;
}

function calcRemindAt(eventTimeLocal: string, minutes: number): string | null {
  if (minutes <= 0) return null;
  const d = new Date(eventTimeLocal);
  d.setMinutes(d.getMinutes() - minutes);
  return d.toISOString();
}

function handleSave() {
  if (!title.value.trim() || !eventTime.value) return;

  const eventTimeISO = new Date(eventTime.value).toISOString();
  const remindAt = calcRemindAt(eventTime.value, advanceMinutes.value);
  const recEnd = recurrence.value !== 'none' && recurrenceEnd.value
    ? new Date(recurrenceEnd.value).toISOString()
    : null;

  if (props.editEvent) {
    emit('update', props.editEvent.id, {
      title: title.value.trim(),
      description: description.value.trim(),
      event_time: eventTimeISO,
      remind_on_time: remindOnTime.value,
      remind_at: remindAt,
      recurrence: recurrence.value,
      recurrence_end: recEnd,
    });
  } else {
    emit('save', {
      title: title.value.trim(),
      description: description.value.trim(),
      event_time: eventTimeISO,
      completed: 0,
      remind_on_time: remindOnTime.value,
      remind_at: remindAt,
      recurrence: recurrence.value,
      recurrence_end: recEnd,
    });
  }
}
</script>

<template>
  <Transition name="slide">
    <div v-if="visible" class="form-overlay" @click.self="emit('close')">
      <div class="form-panel">
        <div class="form-header">
          <span>{{ editEvent ? '编辑事件' : '新建事件' }}</span>
          <button class="form-close" @click="emit('close')" aria-label="关闭">
            <svg width="14" height="14" viewBox="0 0 14 14">
              <line x1="3" y1="3" x2="11" y2="11" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
              <line x1="11" y1="3" x2="3" y2="11" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
            </svg>
          </button>
        </div>

        <div class="form-body">
          <div class="form-group">
            <label>标题</label>
            <input v-model="title" type="text" placeholder="事件标题" class="form-input" />
          </div>

          <div class="form-group">
            <label>描述</label>
            <textarea v-model="description" placeholder="事件描述（可选）" class="form-textarea" rows="2"></textarea>
          </div>

          <div class="form-group">
            <label>时间</label>
            <input v-model="eventTime" type="datetime-local" class="form-input" />
          </div>

          <div class="form-group">
            <label>提醒</label>
            <ReminderConfig
              :remind-on-time="remindOnTime"
              :advance-minutes="advanceMinutes"
              @update:remind-on-time="remindOnTime = $event"
              @update:advance-minutes="advanceMinutes = $event"
            />
          </div>

          <div class="form-group">
            <label>重复</label>
            <select v-model="recurrence" class="form-input">
              <option value="none">不重复</option>
              <option value="daily">每天</option>
              <option value="weekly">每周</option>
              <option value="monthly">每月</option>
            </select>
          </div>

          <div v-if="recurrence !== 'none'" class="form-group">
            <label>重复截止</label>
            <input v-model="recurrenceEnd" type="datetime-local" class="form-input" />
          </div>
        </div>

        <div class="form-footer">
          <button class="btn btn-cancel" @click="emit('close')">取消</button>
          <button class="btn btn-save" @click="handleSave" :disabled="!title.trim() || !eventTime">保存</button>
        </div>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.form-overlay {
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.3);
  backdrop-filter: blur(4px);
  z-index: 100;
  display: flex;
  justify-content: flex-end;
}
.form-panel {
  width: 100%;
  max-width: 320px;
  background: rgba(20, 20, 35, 0.85);
  backdrop-filter: blur(20px);
  display: flex;
  flex-direction: column;
  height: 100%;
  box-shadow: -4px 0 20px rgba(0, 0, 0, 0.15);
  border: 1px solid rgba(255, 255, 255, 0.1);
}
.form-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  background: transparent;
  color: rgba(255, 255, 255, 0.9);
  font-size: 16px;
  font-weight: 600;
  letter-spacing: 0.3px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
}
.form-close {
  width: 28px;
  height: 28px;
  border: none;
  background: rgba(255, 255, 255, 0.1);
  color: rgba(255, 255, 255, 0.6);
  border-radius: 8px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  transition: all 0.2s;
}
.form-close:hover {
  background: rgba(255, 255, 255, 0.18);
  color: rgba(255, 255, 255, 0.9);
  transform: scale(1.05);
}
.form-body {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}
.form-group {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.form-group label {
  font-size: 12px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.6);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}
.form-input, .form-textarea {
  padding: 10px 12px;
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 10px;
  font-size: 13px;
  color: rgba(255, 255, 255, 0.9);
  background: rgba(255, 255, 255, 0.08);
  outline: none;
  transition: all 0.2s;
  font-family: inherit;
}
.form-input:focus, .form-textarea:focus {
  border-color: #667eea;
  box-shadow: 0 0 0 3px rgba(102, 126, 234, 0.2);
}
.form-input::placeholder, .form-textarea::placeholder {
  color: rgba(255, 255, 255, 0.3);
}
.form-textarea {
  resize: none;
}
.form-footer {
  display: flex;
  gap: 8px;
  padding: 12px 16px;
  border-top: 1px solid rgba(255, 255, 255, 0.08);
}
.btn {
  flex: 1;
  padding: 10px 0;
  border: none;
  border-radius: 10px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}
.btn-cancel {
  background: rgba(255, 255, 255, 0.1);
  color: rgba(255, 255, 255, 0.7);
}
.btn-cancel:hover {
  background: rgba(255, 255, 255, 0.18);
  color: rgba(255, 255, 255, 0.9);
  transform: translateY(-1px);
}
.btn-save {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  box-shadow: 0 2px 8px rgba(102, 126, 234, 0.3);
}
.btn-save:hover {
  box-shadow: 0 4px 12px rgba(102, 126, 234, 0.4);
  transform: translateY(-1px);
}
.btn-save:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  transform: none;
}

.slide-enter-active, .slide-leave-active {
  transition: opacity 0.2s ease;
}
.slide-enter-active .form-panel, .slide-leave-active .form-panel {
  transition: transform 0.25s ease;
}
.slide-enter-from {
  opacity: 0;
}
.slide-enter-from .form-panel {
  transform: translateX(100%);
}
.slide-leave-to {
  opacity: 0;
}
.slide-leave-to .form-panel {
  transform: translateX(100%);
}
</style>
