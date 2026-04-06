import { defineConfig } from '@playwright/test';

const chromeExecutablePath = process.env.IKB_PLAYWRIGHT_CHROME_PATH;
const baseURL = process.env.IKB_WEBUI_BASE_URL || 'http://127.0.0.1:0';
const authUser = process.env.IKB_WEBUI_USER;
const authPass = process.env.IKB_WEBUI_PASS;

export default defineConfig({
  testDir: './tests/e2e',
  fullyParallel: false,
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
