import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import path from 'path'
import fs from 'fs'

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
        target: 'https://localhost:3443',
        changeOrigin: true,
        secure: false // Accept self-signed certificates
      },
      '/ws': {
        target: 'wss://localhost:3443',
        ws: true,
        changeOrigin: true,
        secure: false // Accept self-signed certificates
      }
    }
  },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src')
    }
  }
})