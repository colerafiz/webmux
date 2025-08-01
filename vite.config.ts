import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import path from 'node:path'
import fs from 'node:fs'
import { fileURLToPath, URL } from 'node:url'

const __dirname = path.dirname(fileURLToPath(import.meta.url))

export default defineConfig({
  plugins: [vue()],
  server: {
    host: '0.0.0.0', // Bind to all network interfaces
    port: 5173,
    https: {
      key: fs.readFileSync(path.join(__dirname, 'certs', 'key.pem')),
      cert: fs.readFileSync(path.join(__dirname, 'certs', 'cert.pem'))
    },
    proxy: {
      '/api': {
        target: 'https://0.0.0.0:3443',
        changeOrigin: true,
        secure: false, // Accept self-signed certificates
        configure: (proxy) => {
          proxy.on('error', (err) => {
            console.log('proxy error', err);
          });
        }
      },
      '/ws': {
        target: 'wss://0.0.0.0:3443',
        ws: true,
        changeOrigin: true,
        secure: false // Accept self-signed certificates
      }
    }
  },
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url))
    }
  }
})