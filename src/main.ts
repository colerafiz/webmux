import { createApp } from 'vue'
import { VueQueryPlugin } from '@tanstack/vue-query'
import './style.css'
import App from './App.vue'

const app = createApp(App)

app.use(VueQueryPlugin)

app.mount('#app')

// Register service worker for PWA support
if ('serviceWorker' in navigator && window.location.protocol === 'https:') {
  window.addEventListener('load', () => {
    navigator.serviceWorker.register('/service-worker.js')
      .then((registration: ServiceWorkerRegistration) => {
        console.log('ServiceWorker registration successful:', registration.scope);
      })
      .catch((err: Error) => {
        console.log('ServiceWorker registration failed:', err);
      });
  });
}