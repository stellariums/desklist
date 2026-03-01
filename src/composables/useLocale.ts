import { reactive, watch, computed } from 'vue';
import { LazyStore } from '@tauri-apps/plugin-store';
import { zhCN } from '../i18n/zh-CN';
import { en } from '../i18n/en';

export type Locale = 'zh-CN' | 'en';

const messages = { 'zh-CN': zhCN, en } as const;

const state = reactive<{ locale: Locale }>({ locale: 'zh-CN' });
const store = new LazyStore('locale.json');
let initPromise: Promise<void> | null = null;

export function useLocale() {
  async function init() {
    if (!initPromise) {
      initPromise = (async () => {
        const saved = await store.get<Locale>('locale');
        if (saved && saved in messages) {
          state.locale = saved;
        }
        watch(() => state.locale, (locale) => {
          store.set('locale', locale);
          store.save();
        });
      })();
    }
    return initPromise;
  }

  const t = computed(() => messages[state.locale]);

  function setLocale(locale: Locale) {
    state.locale = locale;
  }

  return { locale: state, t, init, setLocale };
}
