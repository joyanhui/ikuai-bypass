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

function readTauriGlobal(): TauriGlobal | undefined {
  return (globalThis as typeof globalThis & { __TAURI__?: TauriGlobal }).__TAURI__;
}

function isLikelyTauriContext(): boolean {
  if (readTauriGlobal()?.core?.invoke) return true;
  const { protocol, hostname } = globalThis.location;
  return protocol === 'tauri:' || hostname === 'tauri.localhost' || hostname.endsWith('.tauri.localhost');
}

let tauriReadyPromise: Promise<TauriGlobal | null> | null = null;

async function waitForTauriGlobal(timeoutMs = 6000, intervalMs = 50): Promise<TauriGlobal | null> {
  const current = readTauriGlobal();
  if (current?.core?.invoke) return current;
  if (!isLikelyTauriContext()) return null;
  if (tauriReadyPromise) return await tauriReadyPromise;

  tauriReadyPromise = (async () => {
    const deadline = Date.now() + timeoutMs;
    while (Date.now() < deadline) {
      const tauri = readTauriGlobal();
      if (tauri?.core?.invoke) return tauri;
      await new Promise((resolve) => globalThis.setTimeout(resolve, intervalMs));
    }
    console.warn('[IKB] waitForTauriGlobal: timed out after', timeoutMs, 'ms');
    return null;
  })();

  try {
    return await tauriReadyPromise;
  } finally {
    tauriReadyPromise = null;
  }
}

async function isTauriReady(): Promise<boolean> {
  return !!(await waitForTauriGlobal());
}

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

export type TestResult = {
  ok: boolean;
  message: string;
};

export type DiagnosticsReport = {
  generated_at: string;
  text: string;
};

export type GithubRelease = {
  tag_name: string;
  name?: string | null;
  prerelease: boolean;
  draft: boolean;
  html_url: string;
  published_at?: string | null;
  created_at?: string | null;
};

export type ProxyConfig = {
  mode: 'custom' | 'system' | 'smart';
  url: string;
  user: string;
  pass: string;
};

export type ConfigMeta = {
  conf_path: string;
  raw_yaml?: string;
  // Backend may include extra keys; allow missing values.
  // 后端可能包含额外字段；这里允许缺省值。
  [k: string]: JsonValue | undefined;
};

type UnlistenFn = () => void;

function isTauri(): boolean {
  return !!readTauriGlobal()?.core?.invoke;
}

async function tauriInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const t = await waitForTauriGlobal();
  if (!t?.core?.invoke) throw new Error(`Tauri IPC not available (cmd: ${cmd})`);
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

  async isTauriReady(): Promise<boolean> {
    return await isTauriReady();
  },

  async getConfigMeta(): Promise<ConfigMeta> {
    if (await isTauriReady()) return await tauriInvoke<ConfigMeta>('get_config_meta');
    return await fetchJson<ConfigMeta>('/api/config');
  },

  async saveRawYaml(yamlText: string): Promise<void> {
    if (await isTauriReady()) {
      await tauriInvoke<void>('save_raw_yaml', {
        yamlText,
      });
      return;
    }
    await fetchJson('/api/save-raw', {
      method: 'POST',
      body: JSON.stringify({ yaml_text: yamlText }),
    });
  },

  async runtimeStatus(): Promise<RuntimeStatus> {
    if (await isTauriReady()) return await tauriInvoke<RuntimeStatus>('runtime_status');
    return await fetchJson<RuntimeStatus>('/api/runtime/status');
  },

  async runtimeRunOnce(module: string): Promise<boolean> {
    if (await isTauriReady()) return await tauriInvoke<boolean>('runtime_run_once', { module });
    const r = await fetchJson<{ started: boolean }>('/api/runtime/run-once', {
      method: 'POST',
      body: JSON.stringify({ module }),
    });
    return !!r.started;
  },

  async runtimeCronStart(expr: string, module: string): Promise<void> {
    if (await isTauriReady()) {
      await tauriInvoke<void>('runtime_cron_start', { expr, module });
      return;
    }
    await fetchJson('/api/runtime/cron/start', {
      method: 'POST',
      body: JSON.stringify({ expr, module }),
    });
  },

  async runtimeCronStop(): Promise<void> {
    if (await isTauriReady()) {
      await tauriInvoke<void>('runtime_cron_stop');
      return;
    }
    await fetchJson('/api/runtime/cron/stop', { method: 'POST' });
  },

  async runtimeStop(): Promise<void> {
    if (await isTauriReady()) {
      await tauriInvoke<void>('runtime_stop');
      return;
    }
    await fetchJson('/api/runtime/stop', { method: 'POST' });
  },

  async runtimeClean(clean_tag: string): Promise<void> {
    if (await isTauriReady()) {
      await tauriInvoke<void>('runtime_clean', { cleanTag: clean_tag });
      return;
    }
    await fetchJson('/api/runtime/clean', {
      method: 'POST',
      body: JSON.stringify({ clean_tag }),
    });
  },

  async runtimeTailLogs(tail: number): Promise<LogRecord[]> {
    if (await isTauriReady()) return await tauriInvoke<LogRecord[]>('runtime_tail_logs', { tail });
    return await fetchJson<LogRecord[]>(`/api/runtime/logs?tail=${tail}`);
  },

  async testIkuaiLogin(baseUrl: string, username: string, password: string): Promise<TestResult> {
    if (await isTauriReady()) {
      return await tauriInvoke<TestResult>('test_ikuai_login', {
        req: {
          baseUrl,
          username,
          password,
        },
      });
    }
    return await fetchJson<TestResult>('/api/test/ikuai-login', {
      method: 'POST',
      body: JSON.stringify({ baseUrl, username, password }),
    });
  },

  async testGithubProxy(githubProxy: string): Promise<TestResult> {
    if (await isTauriReady()) {
      return await tauriInvoke<TestResult>('test_github_proxy', {
        req: {
          githubProxy,
        },
      });
    }
    return await fetchJson<TestResult>('/api/test/github-proxy', {
      method: 'POST',
      body: JSON.stringify({ githubProxy }),
    });
  },

  async fetchGithubReleases(proxy: ProxyConfig): Promise<GithubRelease[]> {
    if (await isTauriReady()) {
      return await tauriInvoke<GithubRelease[]>('fetch_github_releases', { proxy });
    }
    return await fetchJson<GithubRelease[]>('/api/github/releases', {
      method: 'POST',
      body: JSON.stringify({ proxy }),
    });
  },

  async diagnosticsReport(): Promise<DiagnosticsReport> {
    if (await isTauriReady()) {
      return await tauriInvoke<DiagnosticsReport>('diagnostics_report');
    }
    return await fetchJson<DiagnosticsReport>('/api/diagnostics/report');
  },

  async listenLogs(onRecord: (rec: LogRecord) => void, onError?: (err?: unknown) => void): Promise<UnlistenFn> {
    if (await isTauriReady()) {
      const t = await waitForTauriGlobal();
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

  async fetchRemoteConfig(url: string, proxy: ProxyConfig, githubProxy: string): Promise<string> {
    if (await isTauriReady()) {
      return await tauriInvoke<string>('fetch_remote_config', {
        url,
        proxy,
        githubProxy,
      });
    }
    const r = await fetch('/api/remote/fetch', {
      method: 'POST',
      headers: {
        'content-type': 'application/json',
      },
      body: JSON.stringify({ url, proxy, githubProxy }),
    });
    const text = await r.text().catch(() => '');
    if (!r.ok) throw new Error(text || 'HTTP ' + r.status + ' ' + r.statusText);
    return text;
  },
};
