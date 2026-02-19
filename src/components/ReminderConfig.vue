<script setup lang="ts">
defineProps<{
  remindOnTime: number;
  advanceMinutes: number;
}>();

const emit = defineEmits<{
  'update:remindOnTime': [value: number];
  'update:advanceMinutes': [value: number];
}>();

const advanceOptions = [
  { label: '不提醒', value: 0 },
  { label: '5分钟前', value: 5 },
  { label: '15分钟前', value: 15 },
  { label: '30分钟前', value: 30 },
  { label: '1小时前', value: 60 },
  { label: '1天前', value: 1440 },
];
</script>

<template>
  <div class="reminder-config">
    <label class="reminder-row">
      <input
        type="checkbox"
        :checked="remindOnTime === 1"
        @change="emit('update:remindOnTime', ($event.target as HTMLInputElement).checked ? 1 : 0)"
      />
      <span>到期提醒</span>
    </label>
    <label class="reminder-row">
      <span class="reminder-label">提前提醒</span>
      <select
        class="reminder-select"
        :value="advanceMinutes"
        @change="emit('update:advanceMinutes', Number(($event.target as HTMLSelectElement).value))"
      >
        <option v-for="opt in advanceOptions" :key="opt.value" :value="opt.value">{{ opt.label }}</option>
      </select>
    </label>
  </div>
</template>

<style scoped>
.reminder-config {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.reminder-row {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: rgba(255, 255, 255, 0.7);
  cursor: pointer;
}
.reminder-row input[type="checkbox"] {
  width: 16px;
  height: 16px;
  accent-color: #4F46E5;
  cursor: pointer;
}
.reminder-label {
  font-size: 13px;
  color: rgba(255, 255, 255, 0.6);
  min-width: 60px;
}
.reminder-select {
  flex: 1;
  padding: 6px 8px;
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 8px;
  font-size: 13px;
  color: rgba(255, 255, 255, 0.9);
  background: rgba(255, 255, 255, 0.08);
  outline: none;
  transition: border-color 0.15s;
}
.reminder-select:focus {
  border-color: #667eea;
}
.reminder-select option {
  background: #1a1a2e;
  color: rgba(255, 255, 255, 0.9);
}
</style>
