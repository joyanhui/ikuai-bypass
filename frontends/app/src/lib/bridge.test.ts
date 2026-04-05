import { beforeEach, describe, expect, it, vi } from 'vitest';

import { bridge } from './bridge';

describe('bridge transport routing', () => {
  beforeEach(() => {
    vi.restoreAllMocks();
    Object.defineProperty(globalThis, '__TAURI__', {
      value: undefined,
      configurable: true,
      writable: true,
    });
    Object.defineProperty(globalThis, 'location', {
      value: {
        protocol: 'http:',
        hostname: 'localhost',
      },
      configurable: true,
      writable: true,
    });
  });

  it('uses web fetch path when tauri is unavailable', async () => {
    const fetchMock = vi.fn(async () => {
      return new Response(
        JSON.stringify({
          running: false,
          cron_running: false,
          cron_expr: '',
          module: 'iip',
          last_run_at: '',
          next_run_at: '',
        }),
        { status: 200 },
      );
    });
    vi.stubGlobal('fetch', fetchMock);

    const st = await bridge.runtimeStatus();
    expect(st.running).toBe(false);
    expect(fetchMock).toHaveBeenCalledWith('/api/runtime/status', expect.any(Object));
  });

  it('uses tauri invoke path when tauri global is ready', async () => {
    const invoke = vi.fn(async () => ({
      running: true,
      cron_running: true,
      cron_expr: '*/5 * * * *',
      module: 'ii',
      last_run_at: 'x',
      next_run_at: 'y',
    }));

    Object.defineProperty(globalThis, '__TAURI__', {
      value: {
        core: { invoke },
      },
      configurable: true,
      writable: true,
    });

    const st = await bridge.runtimeStatus();
    expect(st.running).toBe(true);
    expect(invoke).toHaveBeenCalledWith('runtime_status', {});
  });
});
