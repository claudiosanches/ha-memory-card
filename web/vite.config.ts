/// <reference types="vitest/config" />
import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';

export default defineConfig({
  // Relative asset URLs so the app works under any ingress prefix.
  base: './',
  plugins: [react()],
  server: {
    // Signs in as the user `make run` names a manager, like ingress would.
    proxy: Object.fromEntries(
      ['/api', '/covers'].map((path) => [
        path,
        { target: 'http://127.0.0.1:8099', headers: { 'X-Remote-User-Id': process.env.DEV_USER ?? 'dev' } },
      ]),
    ),
  },
  test: {
    environment: 'jsdom',
    setupFiles: ['./src/test/setup.ts'],
    css: false,
  },
});
