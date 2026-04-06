import path from 'node:path';

import { defineConfig } from '@playwright/test';

const chromeExecutablePath = process.env.IKB_PLAYWRIGHT_CHROME_PATH;
const baseURL = process.env.IKB_WEBUI_BASE_URL || 'http://127.0.0.1:0';
const authUser = process.env.IKB_WEBUI_USER;
const authPass = process.env.IKB_WEBUI_PASS;
const outputDir = process.env.IKB_PLAYWRIGHT_OUTPUT_DIR
  || (process.env.IKB_WEBUI_ARTIFACT_DIR
    ? path.join(process.env.IKB_WEBUI_ARTIFACT_DIR, 'playwright')
    : path.resolve(process.cwd(), '../../apps/integration-tests/.artifacts/frontends-app-playwright'));

export default defineConfig({
  testDir: './tests/e2e',
  fullyParallel: false,
  outputDir,
  retries: process.env.CI ? 1 : 0,
  timeout: 60_000,
  workers: 1,
  reporter: 'list',
  use: {
    baseURL,
    browserName: 'chromium',
    channel: chromeExecutablePath ? undefined : 'chrome',
    headless: process.env.PLAYWRIGHT_HEADLESS !== '0',
    httpCredentials: authUser && authPass ? {
      username: authUser,
      password: authPass,
    } : undefined,
    launchOptions: chromeExecutablePath ? {
      executablePath: chromeExecutablePath,
    } : undefined,
    screenshot: 'only-on-failure',
    trace: 'retain-on-failure',
  },
});
