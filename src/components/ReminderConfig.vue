<script setup lang="ts">
import { computed } from 'vue';
import { useLocale } from '../composables/useLocale';

defineProps<{
  remindOnTime: number;
  advanceMinutes: number;
}>();

const emit = defineEmits<{
  'update:remindOnTime': [value: number];
  'update:advanceMinutes': [value: number];
}>();

const { t } = useLocale();

const advanceOptions = computed(() => [
  { label: t.value.noReminder, value: 0 },
  { label: t.value.min5Before, value: 5 },
  { label: t.value.min15Before, value: 15 },
  { label: t.value.min30Before, value: 30 },
  { label: t.value.hour1Before, value: 60 },
  { label: t.value.day1Before, value: 1440 },
]);
</script>

<template>
  <div class="reminder-config">
    <label class="reminder-row">
      <input
        type="checkbox"
        :checked="remindOnTime === 1"
        @change="emit('update:remindOnTime', ($event.target as HTMLInputElement).checked ? 1 : 0)"
      />
      <span>{{ t.remindOnTime }}</span>
    </label>
    <label class="reminder-row">
      <span class="reminder-label">{{ t.advanceReminder }}</span>
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
  accent-color: var(--dl-accent);
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
  border: 1px solid var(--dl-border-subtle);
  border-radius: 8px;
  font-size: 13px;
  color: rgba(255, 255, 255, 0.9);
  background: var(--dl-surface);
  outline: none;
  transition: border-color 0.15s;
}
.reminder-select:focus {
  border-color: var(--dl-accent);
}
.reminder-select option {
  background: var(--dl-dropdown-bg);
  color: rgba(255, 255, 255, 0.9);
}
</style>
