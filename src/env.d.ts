/// <reference types="vite/client" />

declare module '*.vue' {
  import type { DefineComponent } from 'vue'
  const component: DefineComponent<{}, {}, any>
  export default component
}

// Environment variable types
interface ImportMetaEnv {
  readonly VITE_API_URL: string
  // Add more env vars as needed
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}