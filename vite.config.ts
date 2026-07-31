import path from 'node:path';

import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';

// Tauri drives the dev server; these settings keep Vite and the Rust core in sync.
const DEV_SERVER_PORT = 1420;

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@app': path.resolve(import.meta.dirname, 'src/app'),
      '@modules': path.resolve(import.meta.dirname, 'src/modules'),
      '@shared': path.resolve(import.meta.dirname, 'src/shared'),
    },
  },
  clearScreen: false,
  server: {
    port: DEV_SERVER_PORT,
    strictPort: true,
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },
  build: {
    target: 'es2022',
    sourcemap: true,
  },
});
