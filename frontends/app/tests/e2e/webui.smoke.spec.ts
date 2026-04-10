import { readFile } from 'node:fs/promises';

import { expect, test, type Page } from '@playwright/test';

const configPath = requiredEnv('IKB_WEBUI_CONFIG_PATH');
const remoteConfigUrl = requiredEnv('IKB_WEBUI_REMOTE_CONFIG_URL');
const remoteTemplateIkuaiUrl = 'http://127.0.0.1:29999';
const remoteTemplateCron = '*/9 * * * * *';

test.describe('WebUI 浏览器 smoke', () => {
  test.describe.configure({ mode: 'serial' });

  test('ipgroup 支持稳定的运行与停止交互', async ({ page }) => {
    await openHome(page);

    const runButton = page.locator('#btnRunAction');
    await page.locator('[data-module="ipgroup"]').click();
    await runButton.click();

    await expect(runButton).toHaveAttribute('data-action', 'stop', {
      timeout: 10_000,
    });
    await expect(page.locator('#subStatus')).toContainText('ipgroup');
    await expect(page.locator('#logContainer')).toContainText('module=ipgroup');

    await runButton.click();
    await expect(page.locator('#toastText')).toContainText('任务已停止', {
      timeout: 10_000,
    });
    await expect(runButton).toHaveAttribute('data-action', 'run', {
      timeout: 10_000,
    });
    await expect(page.locator('#logContainer')).toContainText('TASK:任务停止', {
      timeout: 10_000,
    });
  });

  test('cronAft 可以通过停止定时任务 modal 正常收口', async ({ page }) => {
    await openHome(page);

    const runButton = page.locator('#btnRunAction');
    await page.locator('[data-module="ispdomain"]').click();
    await page.locator('[data-run-mode="cronAft"]').click();
    await page.locator('#cronInput').fill('*/2 * * * * *');
    await runButton.click();

    await expect(runButton).toHaveAttribute('data-action', 'stop', {
      timeout: 10_000,
    });
    await expect(runButton).toContainText('停止定时任务');
    await expect(page.locator('#statusBadge')).toContainText('定时运行', {
      timeout: 10_000,
    });

    await runButton.click();
    await expect(page.locator('#btnStopCronConfirm')).toBeVisible();
    await expect(page.locator('#stopCronModule')).toContainText('ispdomain');
    await expect(page.locator('#stopCronExpr')).toContainText('*/2 * * * * *');

    await page.locator('#btnStopCronConfirm').click();
    await expect(page.locator('#toastText')).toContainText('定时任务已停止', {
      timeout: 10_000,
    });
    await expect(runButton).toHaveAttribute('data-action', 'run', {
      timeout: 10_000,
    });
    await expect(runButton).toContainText('启动 cronAft');
  });

  test('配置页支持测试连接、远程载入和保存', async ({ page }) => {
    await openHome(page);
    await openBasicConfig(page);

    await page.locator('#btnTestIkuaiLogin').click();
    await expect(page.locator('#ikuaiTestHint')).toContainText('连接成功', {
      timeout: 10_000,
    });

    await page.locator('#btnOpenRemoteConfig').click();
    await expect(page.locator('#remoteUrl')).toBeVisible();
    await page.locator('#remoteUrl').fill(remoteConfigUrl);
    await page.locator('#btnLoadRemote').click();
    await expect(page.locator('#remoteHint')).toContainText('加载成功', {
      timeout: 10_000,
    });
    await expect(page.locator('#toastText')).toContainText('远程配置已加载', {
      timeout: 10_000,
    });
    await page.locator('#btnCloseRemoteConfig').click();

    await expect(page.locator('#cfgIkuaiUrl')).toHaveValue(remoteTemplateIkuaiUrl);
    await expect(page.locator('#cfgCronInline')).toHaveValue(remoteTemplateCron);

    await page.locator('#btnSaveConfig').click();
    await expect(page.locator('#toastText')).toContainText('配置已保存', {
      timeout: 10_000,
    });
    await expect.poll(readConfigText).toContain(remoteTemplateIkuaiUrl);
    await expect.poll(readConfigText).toContain(remoteTemplateCron);
  });
});

async function readConfigText(): Promise<string> {
  return await readFile(configPath, 'utf8');
}

async function openHome(page: Page): Promise<void> {
  await page.goto('/');
  await expect(page.locator('#mainTabGrid')).toBeVisible();
  await expect(page.locator('#btnRunAction')).toHaveAttribute('data-action', 'run', {
    timeout: 15_000,
  });
}

async function openBasicConfig(page: Page): Promise<void> {
  await page.locator('[data-tab="config"]').click();
  const basicConfigToggle = page.locator('#btnToggleBasicConfig');
  if ((await basicConfigToggle.getAttribute('aria-expanded')) !== 'true') {
    await basicConfigToggle.click();
  }
  await expect(page.locator('#cfgIkuaiUrl')).toBeVisible();
  await expect(page.locator('#cfgCronInline')).toBeVisible();
}

function requiredEnv(key: string): string {
  const value = process.env[key];
  if (!value) {
    throw new Error(`missing required env: ${key}`);
  }
  return value;
}
