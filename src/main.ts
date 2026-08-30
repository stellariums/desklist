import { createApp } from 'vue';
import { isTauri } from '@tauri-apps/api/core';
import App from './App.vue';
import './styles/global.css';

document.documentElement.classList.toggle('browser-mode', !isTauri());

createApp(App).mount('#app');
