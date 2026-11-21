import { mkdtempSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { defineConfig, devices } from '@playwright/test';
import { APP_PORT, MANAGER_ID, PREFIX, PROXY_ORIGIN, PROXY_PORT } from './constants.ts';

// Fresh /data for every run.
const dataDir = mkdtempSync(join(tmpdir(), 'memory-card-e2e-'));

export default defineConfig({
  testDir: './specs',
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  reporter: process.env.CI ? [['github'], ['html', { open: 'never' }]] : 'list',
  use: {
    baseURL: PROXY_ORIGIN,
    trace: 'retain-on-failure',
  },
  projects: [
    { name: 'desktop', use: { ...devices['Desktop Chrome'] } },
    { name: 'mobile', use: { ...devices['Pixel 7'] } },
  ],
  webServer: [
    {
      // Built by `cargo build` with the web app built first (see README).
      command: '../target/debug/memory-card serve',
      url: `http://127.0.0.1:${APP_PORT}/api/status`,
      env: {
        MEMORY_CARD_DATA_DIR: join(dataDir, 'data'),
        MEMORY_CARD_TMP_DIR: join(dataDir, 'tmp'),
        MEMORY_CARD_LISTEN: `127.0.0.1:${APP_PORT}`,
        MEMORY_CARD_TRUSTED_PEER: '127.0.0.1',
        MEMORY_CARD_LOG_LEVEL: 'warning',
        MEMORY_CARD_MANAGERS: MANAGER_ID,
      },
      reuseExistingServer: false,
    },
    {
      command: 'node ingress-proxy.ts',
      port: PROXY_PORT,
      env: {
        INGRESS_PREFIX: PREFIX,
        UPSTREAM: `http://127.0.0.1:${APP_PORT}`,
        PORT: String(PROXY_PORT),
        DEFAULT_USER: MANAGER_ID,
      },
      reuseExistingServer: false,
    },
  ],
});
