import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

// Phase 1: frontend runs inside Tauri WebView2 on Windows.
// Dev server is fixed port so Tauri can attach reliably.
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
  },
  build: {
    target: 'es2022',
  },
  test: {
    environment: 'jsdom',
    globals: true,
    setupFiles: ['./src/test/setup.ts'],
    exclude: ['tests/**', 'node_modules/**'],
  },
} as never);
