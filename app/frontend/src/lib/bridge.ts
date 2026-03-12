type JsonValue = null | boolean | number | string | JsonValue[] | { [k: string]: JsonValue };
type TauriInvoke = <T>(cmd: string, args?: Record<string, unknown>) => Promise<T>;
type TauriEvent = {
  listen?: (event: string, cb: (ev: { payload?: unknown }) => void) => Promise<() => void>;
};
type TauriCore = {
  invoke?: TauriInvoke;
};
type TauriGlobal = {
  core?: TauriCore;
  event?: TauriEvent;
};

export type RuntimeStatus = {
  running: boolean;
  cron_running: boolean;
  cron_expr: string;
  module: string;
  last_run_at: string;
  next_run_at: string;
};

export type LogRecord = {
  ts: string;
  module: string;
  tag: string;
  level: string;
  detail: string;
};

export type ConfigMeta = {
  conf_path: string;
  raw_yaml?: string;
  top_level_comments?: Record<string, string>;
  item_comments?: Record<string, string>;
  webui_comments?: Record<string, string>;
  max_number_of_one_records_comments?: Record<string, string>;
  [k: string]: JsonValue;
};

type UnlistenFn = () => void;

function isTauri(): boolean {
  const t = (globalThis as typeof globalThis & { __TAURI__?: TauriGlobal }).__TAURI__;
  return !!t?.core?.invoke;
}

async function tauriInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const t = (globalThis as typeof globalThis & { __TAURI__?: TauriGlobal }).__TAURI__;
  if (!t?.core?.invoke) throw new Error('Tauri bridge is not available');
  return await t.core.invoke(cmd, args || {});
}

async function fetchJson<T>(url: string, options?: RequestInit): Promise<T> {
  const r = await fetch(url, {
    ...options,
    headers: {
      'content-type': 'application/json',
      ...(options?.headers || {}),
    },
  });
  if (!r.ok) throw new Error(await r.text());
  return await r.json();
}

export const bridge = {
  isTauri,

  async getConfigMeta(): Promise<ConfigMeta> {
    if (isTauri()) return await tauriInvoke<ConfigMeta>('get_config_meta');
    return await fetchJson<ConfigMeta>('/api/config');
  },

  async saveConfig(payload: Record<string, unknown>, withComments: boolean): Promise<void> {
    if (isTauri()) {
      await tauriInvoke<void>('save_config_with_comments', {
        config: payload,
        withComments: withComments,
      });
      return;
    }
    await fetchJson('/api/save', {
      method: 'POST',
      body: JSON.stringify({ ...payload, with_comments: withComments }),
    });
  },

  async saveRawYaml(yamlText: string, withComments: boolean): Promise<void> {
    if (isTauri()) {
      await tauriInvoke<void>('save_raw_yaml', {
        yamlText,
        withComments,
      });
      return;
    }
    await fetchJson('/api/save-raw', {
      method: 'POST',
      body: JSON.stringify({ yaml_text: yamlText, with_comments: withComments }),
    });
  },

  async runtimeStatus(): Promise<RuntimeStatus> {
    if (isTauri()) return await tauriInvoke<RuntimeStatus>('runtime_status');
    return await fetchJson<RuntimeStatus>('/api/runtime/status');
  },

  async runtimeRunOnce(module: string): Promise<boolean> {
    if (isTauri()) return await tauriInvoke<boolean>('runtime_run_once', { module });
    const r = await fetchJson<{ started: boolean }>('/api/runtime/run-once', {
      method: 'POST',
      body: JSON.stringify({ module }),
    });
    return !!r.started;
  },

  async runtimeCronStart(expr: string, module: string): Promise<void> {
    if (isTauri()) {
      await tauriInvoke<void>('runtime_cron_start', { expr, module });
      return;
    }
    await fetchJson('/api/runtime/cron/start', {
      method: 'POST',
      body: JSON.stringify({ expr, module }),
    });
  },

  async runtimeCronStop(): Promise<void> {
    if (isTauri()) {
      await tauriInvoke<void>('runtime_cron_stop');
      return;
    }
    await fetchJson('/api/runtime/cron/stop', { method: 'POST' });
  },

  async runtimeStop(): Promise<void> {
    if (isTauri()) {
      await tauriInvoke<void>('runtime_stop');
      return;
    }
    await fetchJson('/api/runtime/stop', { method: 'POST' });
  },

  async runtimeClean(clean_tag: string): Promise<void> {
    if (isTauri()) {
      await tauriInvoke<void>('runtime_clean', { cleanTag: clean_tag });
      return;
    }
    await fetchJson('/api/runtime/clean', {
      method: 'POST',
      body: JSON.stringify({ clean_tag }),
    });
  },

  async runtimeTailLogs(tail: number): Promise<LogRecord[]> {
    if (isTauri()) return await tauriInvoke<LogRecord[]>('runtime_tail_logs', { tail });
    return await fetchJson<LogRecord[]>(`/api/runtime/logs?tail=${tail}`);
  },

  async listenLogs(onRecord: (rec: LogRecord) => void, onError?: (err?: unknown) => void): Promise<UnlistenFn> {
    if (isTauri()) {
      const t = (globalThis as typeof globalThis & { __TAURI__?: TauriGlobal }).__TAURI__;
      if (!t?.event?.listen) throw new Error('Tauri event bridge is not available');
      const unlisten = await t.event.listen('ikb://log', (ev) => {
        const rec = ev?.payload;
        if (rec && typeof rec === 'object') onRecord(rec as LogRecord);
      });
      return () => {
        try {
          unlisten();
        } catch (err) {
          console.warn('[IKB] Failed to unlisten Tauri logs', err);
        }
      };
    }

    const es = new EventSource('/api/runtime/logs/stream');
    es.onmessage = (ev) => {
      try {
        const rec = JSON.parse(ev.data) as LogRecord;
        onRecord(rec);
      } catch (err) {
        console.warn('[IKB] Failed to parse log event', err);
      }
    };
    es.onerror = (err) => {
      if (onError) onError(err);
    };

    return () => {
      try {
        es.close();
      } catch (err) {
        console.warn('[IKB] Failed to close EventSource', err);
      }
    };
  },

  async fetchRemoteConfig(url: string, githubProxy: string): Promise<string> {
    if (isTauri()) {
      return await tauriInvoke<string>('fetch_remote_config', {
        url,
        githubProxy,
      });
    }
    const r = await fetch(url);
    if (!r.ok) throw new Error('HTTP ' + r.status + ' ' + r.statusText);
    return await r.text();
  },
};
