import { readFile } from 'node:fs/promises';

import { expect, test } from '@playwright/test';

const configPath = requiredEnv('IKB_WEBUI_CONFIG_PATH');

test('WebUI 浏览器 smoke 覆盖登录、运行停止与配置保存', async ({ page }) => {
  await page.goto('/');

  await expect(page.locator('#mainTabGrid')).toBeVisible();
  await expect(page.locator('#btnRunAction')).toHaveAttribute('data-action', 'run', {
    timeout: 15_000,
  });

  await page.locator('[data-module="ispdomain"]').click();
  await page.locator('#btnRunAction').click();
  await expect(page.locator('#btnRunAction')).toHaveAttribute('data-action', 'stop');

  await page.locator('#btnRunAction').click();
  await expect(page.locator('#btnRunAction')).toHaveAttribute('data-action', 'run', {
    timeout: 10_000,
  });

  await page.locator('[data-tab="config"]').click();
  const basicConfigToggle = page.locator('#btnToggleBasicConfig');
  if ((await basicConfigToggle.getAttribute('aria-expanded')) !== 'true') {
    await basicConfigToggle.click();
  }

  const ikuaiUrlInput = page.locator('#cfgIkuaiUrl');
  const cronInput = page.locator('#cfgCronInline');
  await expect(ikuaiUrlInput).toBeVisible();
  await expect(cronInput).toBeVisible();

  const nextIkuaiUrl = 'http://127.0.0.1:29999';
  const nextCron = '*/5 * * * * *';
  await ikuaiUrlInput.fill(nextIkuaiUrl);
  await cronInput.fill(nextCron);
  await page.locator('#btnSaveNoComments').click();

  await expect(page.locator('#toastText')).toContainText('配置已保存', {
    timeout: 10_000,
  });
  await expect.poll(readConfigText).toContain(nextIkuaiUrl);
  await expect.poll(readConfigText).toContain(nextCron);
});

async function readConfigText(): Promise<string> {
  return await readFile(configPath, 'utf8');
}

function requiredEnv(key: string): string {
  const value = process.env[key];
  if (!value) {
    throw new Error(`missing required env: ${key}`);
  }
  return value;
}
