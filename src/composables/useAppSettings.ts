import { reactive, watch } from 'vue';
import { LazyStore } from '@tauri-apps/plugin-store';
import type { AppSettings } from '../types';

const DEFAULTS: AppSettings = { defaultRemindOnTime: 1 };
const settings = reactive<AppSettings>({ ...DEFAULTS });
const store = new LazyStore('app-settings.json');
let initPromise: Promise<void> | null = null;

export function useAppSettings() {
  async function init() {
    if (!initPromise) {
      initPromise = (async () => {
        const saved = await store.get<AppSettings>('app');
        if (saved) Object.assign(settings, saved);
        watch(settings, () => {
          store.set('app', { ...settings });
          store.save();
        }, { deep: true });
      })();
    }
    return initPromise;
  }

  function resetDefaults() {
    Object.assign(settings, DEFAULTS);
  }

  return { settings, init, resetDefaults };
}
