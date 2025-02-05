import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'

// https://vite.dev/config/
export default defineConfig({
  root: 'frontend',
  plugins: [svelte()],
  build: {
    outDir: '../dist',
    emptyOutDir: true,
  },
  server: {
    host: '127.0.0.1',
    port: 4000,
    proxy: {
      '/api/': {
        target: 'http://127.0.0.1:3030',
      }
    }
  }
})
