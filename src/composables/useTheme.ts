import { reactive, watch } from 'vue';
import { LazyStore } from '@tauri-apps/plugin-store';
import type { ThemeSettings } from '../types';

const DEFAULTS: ThemeSettings = { windowOpacity: 0.65, accentColor: '#667eea' };
const settings = reactive<ThemeSettings>({ ...DEFAULTS });
const store = new LazyStore('theme.json');
let initPromise: Promise<void> | null = null;

function hexToRgb(hex: string) {
  const n = parseInt(hex.slice(1), 16);
  return { r: (n >> 16) & 255, g: (n >> 8) & 255, b: n & 255 };
}

function hexToHsl(hex: string) {
  const { r, g, b } = hexToRgb(hex);
  const r1 = r / 255, g1 = g / 255, b1 = b / 255;
  const max = Math.max(r1, g1, b1), min = Math.min(r1, g1, b1);
  let h = 0, s = 0;
  const l = (max + min) / 2;
  if (max !== min) {
    const d = max - min;
    s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
    if (max === r1) h = ((g1 - b1) / d + (g1 < b1 ? 6 : 0)) / 6;
    else if (max === g1) h = ((b1 - r1) / d + 2) / 6;
    else h = ((r1 - g1) / d + 4) / 6;
  }
  return { h: h * 360, s: s * 100, l: l * 100 };
}

function deriveGradientEnd(hex: string) {
  const { h, s, l } = hexToHsl(hex);
  return `hsl(${(h + 45) % 360}, ${s}%, ${Math.max(l - 10, 20)}%)`;
}

function lighten(hex: string, amount: number) {
  const { h, s, l } = hexToHsl(hex);
  return `hsl(${h}, ${s}%, ${Math.min(l + amount, 95)}%)`;
}

function applyTheme() {
  const { r, g, b } = hexToRgb(settings.accentColor);
  const end = deriveGradientEnd(settings.accentColor);
  const s = document.documentElement.style;
  s.setProperty('--dl-bg', `rgba(15,15,25,${settings.windowOpacity})`);
  s.setProperty('--dl-accent', settings.accentColor);
  s.setProperty('--dl-accent-end', end);
  s.setProperty('--dl-accent-gradient', `linear-gradient(135deg, ${settings.accentColor}, ${end})`);
  s.setProperty('--dl-accent-shadow', `rgba(${r},${g},${b},0.3)`);
  s.setProperty('--dl-accent-shadow-strong', `rgba(${r},${g},${b},0.4)`);
  s.setProperty('--dl-accent-ring', `rgba(${r},${g},${b},0.2)`);
  s.setProperty('--dl-accent-subtle', `rgba(${r},${g},${b},0.15)`);
  s.setProperty('--dl-accent-border-hover', `rgba(${r},${g},${b},0.25)`);
  s.setProperty('--dl-accent-light', lighten(settings.accentColor, 30));
}

export function useTheme() {
  async function init() {
    if (!initPromise) {
      initPromise = (async () => {
        const saved = await store.get<ThemeSettings>('theme');
        if (saved) Object.assign(settings, saved);
        applyTheme();
        watch(settings, () => {
          applyTheme();
          store.set('theme', { ...settings });
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
