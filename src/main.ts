import { createApp } from "vue";
import App from "./App.vue";
import './index.css';

import { createMemoryHistory, createRouter } from 'vue-router'
import { createPinia } from 'pinia'
import IndexView from './pages/IndexView.vue'
import AboutView from './pages/AboutView.vue'
import SettingView from "./pages/SettingView.vue";
const pinia = createPinia()
const routes = [
  { path: '/', component: IndexView },
  { path: '/about', component: AboutView },
  { path: '/settings', component: SettingView },
]

const router = createRouter({
  history: createMemoryHistory(),
  routes,
})

createApp(App).use(router).use(pinia).mount("#app");
