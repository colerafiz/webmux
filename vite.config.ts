import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import path from 'node:path'
import fs from 'node:fs'
import { fileURLToPath, URL } from 'node:url'

const __dirname = path.dirname(fileURLToPath(import.meta.url))

// Try to load HTTPS certificates
let httpsConfig: any = false
try {
  httpsConfig = {
    key: fs.readFileSync(path.join(__dirname, 'certs', 'key.pem')),
    cert: fs.readFileSync(path.join(__dirname, 'certs', 'cert.pem'))
  }
} catch (error) {
  console.log('HTTPS certificates not found, running in HTTP mode')
}

export default defineConfig({
  plugins: [vue()],
  server: {
    host: '0.0.0.0', // Bind to all network interfaces
    port: 5174, // Dev uses 5174, main uses 5173
    https: httpsConfig,
    proxy: {
      '/api': {
        target: httpsConfig ? 'https://0.0.0.0:4443' : 'http://0.0.0.0:4000', // Dev backend uses 4443/4000
        changeOrigin: true,
        secure: false, // Accept self-signed certificates
        configure: (proxy) => {
          proxy.on('error', (err) => {
            console.log('proxy error', err);
          });
        }
      },
      '/ws': {
        target: httpsConfig ? 'wss://0.0.0.0:4443' : 'ws://0.0.0.0:4000', // Dev backend uses 4443/4000
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