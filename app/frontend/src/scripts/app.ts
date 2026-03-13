import { bridge } from '../lib/bridge.ts';
import type { RuntimeStatus } from '../lib/bridge.ts';
import { defaultUiConfig, fromBackendMeta, toBackendPayload, yamlDumpWithComments, yamlParse } from '../lib/config_model.ts';
import type { UiConfig } from '../lib/config_model.ts';
import { loadJson, saveJson } from '../lib/storage.ts';
import { removeYamlSeqItem, updateYamlPaths, upsertYamlSeqItem } from '../lib/yaml_ast.ts';

type MonacoModule = typeof import('monaco-editor/esm/vs/editor/editor.api');
type MonacoEditor = import('monaco-editor/esm/vs/editor/editor.api').editor.IStandaloneCodeEditor;
type YamlLanguageModule = typeof import('monaco-editor/esm/vs/basic-languages/yaml/yaml.js');

// ============================================
// 全局状态
// ============================================
const state = {
  cfg: defaultUiConfig(),
  comments: { top: {}, item: {}, webui: {}, maxNumberOfOneRecords: {} },
  rawYaml: '',
  confPath: '',
  selectedModule: 'ispdomain',
  selectedRunMode: 'once' as 'cron' | 'cronAft' | 'once' | 'clean',
  selectedConfigTab: 'visual' as 'visual' | 'raw',
  isRunning: false,
  isCronRunning: false,
  lastRuntimeStatus: null as RuntimeStatus | null,
  unlistenLogs: null as (() => void) | null,
  streamReconnectTimer: null as ReturnType<typeof setTimeout> | null,
  ruleEditor: null as null | {
    listKey: RuleListKey;
    index: number;
  },
  rawEditor: null as MonacoEditor | null,
  rawEditorTextarea: null as HTMLTextAreaElement | null,
};

let monaco: MonacoModule | null = null;
let yamlLanguageModule: YamlLanguageModule | null = null;
let monacoLoading: Promise<void> | null = null;

const RECONNECT_DELAY = 3000;
let yamlLanguageRegistered = false;
const MODAL_IDS = ['remoteConfigModal', 'ruleEditorModal', 'stopCronModal'] as const;

let configMissingDetected = false;
let configMissingPrompted = false;

const getErrorMessage = (err: unknown): string => {
  if (typeof err === 'string') return err;
  if (err instanceof Error) return err.message || err.toString();
  if (err && typeof err === 'object' && 'message' in err) {
    const value = (err as { message?: unknown }).message;
    if (typeof value === 'string') return value;
  }
  return '未知错误';
};

const getRawEditorValue = () => {
  if (state.rawEditor) return state.rawEditor.getValue();
  if (state.rawEditorTextarea) return state.rawEditorTextarea.value;
  return '';
};

const setRawEditorValue = (value: string) => {
  if (state.rawEditor) {
    state.rawEditor.setValue(value);
    return;
  }
  if (state.rawEditorTextarea) {
    state.rawEditorTextarea.value = value;
  }
};

const refreshEditorFromRawYaml = () => {
  if (state.selectedConfigTab !== 'raw') return;
  const current = getRawEditorValue();
  if (current !== state.rawYaml) {
    setRawEditorValue(state.rawYaml);
  }
};

const applyStateFromRawYaml = () => {
  const doc = yamlParse(state.rawYaml);
  const parsed = fromBackendMeta(doc);
  state.cfg = parsed.cfg;
  if (Object.keys(parsed.comments.top).length > 0) {
    state.comments = parsed.comments;
  }
};

// ============================================
// Toast 提示系统
// ============================================
const showToast = (message: string, duration = 2000) => {
  const toast = document.getElementById('toast');
  const toastText = document.getElementById('toastText');
  if (!toast || !toastText) return;
  
  toastText.textContent = message;
  toast.classList.remove('opacity-0', 'translate-y-4');
  toast.classList.add('opacity-100', 'translate-y-0');
  
  setTimeout(() => {
    toast.classList.add('opacity-0', 'translate-y-4');
    toast.classList.remove('opacity-100', 'translate-y-0');
  }, duration);
};

// ============================================
// GitHub Proxy (ghproxy) 快速选择
// ============================================
const DEFAULT_REMOTE_TEMPLATE_URL =
  'https://raw.githubusercontent.com/joyanhui/ikuai-bypass/refs/heads/main/config.yml';

const COMMON_GITHUB_PROXIES = [
  'https://gh-proxy.com/',
  'https://ghproxy.net/',
  'https://ghproxy.vip/',
  'https://github.akams.cn/',
  'https://gh.llkk.cc/',
  'https://ghproxy.wujiyan.cc/',
] as const;

const ghProxyPickerRefresh = new Map<string, () => void>();

const normalizeGhProxy = (proxy: string): string => {
  const p = proxy.trim();
  if (!p) return '';
  return p.endsWith('/') ? p : p + '/';
};

const isGithubUrlForProxy = (url: string): boolean => {
  const u = url.trim();
  return u.startsWith('https://raw.githubusercontent.com/') || u.startsWith('https://github.com/');
};

const applyGhProxy = (proxy: string, url: string): string => {
  const p = normalizeGhProxy(proxy);
  if (!p) return url;
  return p + url;
};

const uniqueStrings = (items: string[]): string[] => {
  const out: string[] = [];
  const seen = new Set<string>();
  let hasEmpty = false;
  for (const it of items) {
    if (it === '' && !hasEmpty) {
      hasEmpty = true;
      out.push('');
      continue;
    }
    const v = it.trim();
    if (!v) continue;
    if (seen.has(v)) continue;
    seen.add(v);
    out.push(v);
  }
  return out;
};

const buildGhProxyCandidates = (url: string, enabled: boolean, preferredProxy: string, fallbackProxy: string): string[] => {
  if (!enabled) return [''];
  if (!isGithubUrlForProxy(url)) return [''];

  const list: string[] = [];
  const p1 = normalizeGhProxy(preferredProxy);
  const p2 = normalizeGhProxy(fallbackProxy);
  if (p1) list.push(p1);
  if (p2) list.push(p2);
  list.push(...COMMON_GITHUB_PROXIES.map((p) => normalizeGhProxy(p)));

  const proxies = uniqueStrings(list);
  // 最后追加一次直连尝试，避免某些环境下代理不可用。
  // Add direct attempt as a last resort.
  proxies.push('');
  return proxies;
};

const mountGhProxyQuickPick = (inputId: string, containerId: string) => {
  const input = document.getElementById(inputId) as HTMLInputElement | null;
  const container = document.getElementById(containerId);
  if (!input || !container) return;

  const proxies = [...COMMON_GITHUB_PROXIES];
  const normalize = (v: string) => normalizeGhProxy(v);

  container.innerHTML = '';

  const wrap = document.createElement('div');
  wrap.className = 'flex flex-wrap items-center gap-2';

  const syncActive = () => {
    const current = normalize(input.value);
    wrap.querySelectorAll<HTMLButtonElement>('button[data-proxy]')?.forEach((btn) => {
      const val = normalize(btn.dataset.proxy || '');
      const active = !!val && val === current;
      btn.classList.toggle('bg-primary-100', active);
      btn.classList.toggle('text-primary-700', active);
      btn.classList.toggle('dark:bg-primary-900/30', active);
      btn.classList.toggle('dark:text-primary-300', active);
      btn.classList.toggle('bg-white/70', !active);
      btn.classList.toggle('text-gray-700', !active);
      btn.classList.toggle('dark:bg-gray-800/50', !active);
      btn.classList.toggle('dark:text-gray-300', !active);
    });
  };

  ghProxyPickerRefresh.set(inputId, syncActive);

  const mkTag = (proxy: string) => {
    const btn = document.createElement('button');
    btn.type = 'button';
    btn.dataset.proxy = proxy;
    btn.className =
      'rounded-full border border-gray-200 px-3 py-1 text-xs font-medium transition hover:bg-gray-50 dark:border-gray-700 dark:hover:bg-gray-800';
    btn.textContent = proxy.replace(/^https?:\/\//, '').replace(/\/$/, '');
    btn.addEventListener('click', () => {
      input.value = proxy;
      input.dispatchEvent(new Event('input', { bubbles: true }));
      input.focus();
      syncActive();
    });
    return btn;
  };

  proxies.forEach((p) => wrap.appendChild(mkTag(p)));

  const help = document.createElement('button');
  help.type = 'button';
  help.className =
    'ml-1 flex h-8 w-8 items-center justify-center rounded-full border border-gray-200 bg-white/70 text-xs font-bold text-gray-600 transition hover:bg-gray-50 dark:border-gray-700 dark:bg-gray-800/50 dark:text-gray-300 dark:hover:bg-gray-800';
  help.textContent = '?';
  help.title = '获取更多可用 ghproxy 站点';
  help.addEventListener('click', () => {
    showToast('提示：可用 Bing/Google 搜索关键词 ghproxy 获取更多可用站点', 3200);
  });
  wrap.appendChild(help);

  const testBtn = document.createElement('button');
  testBtn.type = 'button';
  testBtn.className =
    'flex h-8 items-center justify-center rounded-full border border-gray-200 bg-white/70 px-3 text-xs font-medium text-gray-700 transition hover:bg-gray-50 disabled:opacity-60 dark:border-gray-700 dark:bg-gray-800/50 dark:text-gray-300 dark:hover:bg-gray-800';
  testBtn.textContent = '测试';
  testBtn.title = '测试该 GitHub Proxy 是否可用';
  testBtn.addEventListener('click', async () => {
    const proxy = input.value.trim();
    if (!proxy) {
      showToast('请先填写 GitHub Proxy 地址', 2600);
      return;
    }
    const old = testBtn.textContent;
    testBtn.disabled = true;
    testBtn.textContent = '测试中...';
    try {
      const r = await bridge.testGithubProxy(proxy);
      if (r.ok) {
        showToast('GitHub Proxy 可用');
      } else {
        showToast('GitHub Proxy 不可用: ' + (r.message || 'unknown error'), 3600);
      }
    } catch (err) {
      showToast('测试失败: ' + getErrorMessage(err), 3600);
    } finally {
      testBtn.disabled = false;
      testBtn.textContent = old || '测试';
    }
  });
  wrap.appendChild(testBtn);

  container.appendChild(wrap);
  input.addEventListener('input', () => syncActive());
  syncActive();
};

// ============================================
// Modal 系统
// ============================================
const openModal = (modalId: string) => {
  const modal = document.getElementById(modalId);
  if (!modal) return;
  modal.classList.remove('hidden');
  // 触发重排以启动动画
  void modal.offsetWidth;
  modal.classList.add('modal-open');
  document.body.style.overflow = 'hidden';
};

const closeModal = (modalId: string) => {
  const modal = document.getElementById(modalId);
  if (!modal) return;
  modal.classList.remove('modal-open');
  setTimeout(() => {
    modal.classList.add('hidden');
    const hasOpenModal = MODAL_IDS.some((id) => {
      const el = document.getElementById(id);
      return el && !el.classList.contains('hidden');
    });
    document.body.style.overflow = hasOpenModal ? 'hidden' : '';
  }, 300);
};

const initModalEscape = () => {
  document.addEventListener('keydown', (event) => {
    if (event.key !== 'Escape') return;
    const visible = [...MODAL_IDS].reverse().find((id) => {
      const el = document.getElementById(id);
      return el && !el.classList.contains('hidden');
    });
    if (visible) {
      closeModal(visible);
    }
  });
};

// ============================================
// 停止定时任务 Modal
// ============================================
const syncStopCronModalFromStatus = (st: RuntimeStatus | null) => {
  const moduleEl = document.getElementById('stopCronModule');
  const exprEl = document.getElementById('stopCronExpr');
  const nextEl = document.getElementById('stopCronNext');
  const hintEl = document.getElementById('stopCronHint');

  const safeText = (v: string | null | undefined) => {
    const s = (v || '').trim();
    return s ? s : '--';
  };

  if (moduleEl) moduleEl.textContent = safeText(st?.module);
  if (exprEl) exprEl.textContent = safeText(st?.cron_expr);
  if (nextEl) nextEl.textContent = safeText(st?.next_run_at);

  if (hintEl) {
    if (st?.running) {
      hintEl.textContent =
        '当前任务正在执行中。本操作仅停止定时任务，不会中断当前执行；停止后计划任务将不会再按 Cron 自动执行。若要立刻切换到 once / clean，请先在主界面点击“停止执行”中断当前任务。';
    } else {
      hintEl.textContent =
        '停止后可能会导致计划任务不会再执行。停止后你可以切换到 once / clean 模式执行；如需再次启动定时任务，请选择 cron 或 cronAft 模式并点击启动。';
    }
  }
};

const openStopCronModal = async () => {
  try {
    const st = await bridge.runtimeStatus();
    state.lastRuntimeStatus = st;
    syncStopCronModalFromStatus(st);
    openModal('stopCronModal');
  } catch (err) {
    showToast('无法获取运行状态: ' + getErrorMessage(err), 3200);
  }
};

const initStopCronModal = () => {
  const close = () => closeModal('stopCronModal');

  document.getElementById('stopCronBackdrop')?.addEventListener('click', close);
  document.getElementById('btnStopCronCancelTop')?.addEventListener('click', close);
  document.getElementById('btnStopCronCancel')?.addEventListener('click', close);

  const confirmBtn = document.getElementById('btnStopCronConfirm') as HTMLButtonElement | null;
  confirmBtn?.addEventListener('click', async () => {
    if (!confirmBtn) return;
    const old = confirmBtn.textContent;
    confirmBtn.disabled = true;
    confirmBtn.textContent = '停止中...';
    try {
      await bridge.runtimeCronStop();
      state.isCronRunning = false;
      showToast('定时任务已停止');
      close();
      void updateStatus();
    } catch (err) {
      showToast('停止失败: ' + getErrorMessage(err), 3200);
    } finally {
      confirmBtn.disabled = false;
      confirmBtn.textContent = old || '停止定时任务';
    }
  });
};

const setRemoteTemplateTip = (message: string | null) => {
  const el = document.getElementById('remoteTemplateTip');
  if (!el) return;
  if (!message) {
    el.textContent = '';
    el.classList.add('hidden');
    return;
  }
  el.textContent = message;
  el.classList.remove('hidden');
};

const loadMonaco = async () => {
  if (monaco) return;
  if (monacoLoading) {
    await monacoLoading;
    return;
  }
  monacoLoading = (async () => {
    const [monacoModule, yamlModule, workerModule] = await Promise.all([
      import('monaco-editor/esm/vs/editor/editor.api'),
      import('monaco-editor/esm/vs/basic-languages/yaml/yaml.js'),
      import('monaco-editor/esm/vs/editor/editor.worker?worker'),
    ]);

    monaco = monacoModule;
    yamlLanguageModule = yamlModule;
    const EditorWorker = workerModule.default;

    (globalThis as typeof globalThis & {
      MonacoEnvironment?: {
        getWorker: (_workerId: string, _label: string) => Worker;
      };
    }).MonacoEnvironment = {
      getWorker() {
        return new EditorWorker();
      },
    };
  })();

  try {
    await monacoLoading;
  } finally {
    monacoLoading = null;
  }
};

const shouldUseLightweightEditor = () => {
  if (/Android|iPhone|iPad|iPod/i.test(navigator.userAgent)) return true;
  if (typeof navigator.deviceMemory === 'number' && navigator.deviceMemory <= 4) return true;
  return false;
};

const ensureRawEditor = async () => {
  if (state.rawEditor || state.rawEditorTextarea) return;
  const container = document.getElementById('rawEditorContainer');
  if (!container) return;

  if (shouldUseLightweightEditor()) {
    const textarea = document.createElement('textarea');
    textarea.className = 'h-[32rem] w-full resize-none bg-transparent p-4 text-sm text-gray-800 outline-none dark:text-gray-100';
    textarea.spellcheck = false;
    container.innerHTML = '';
    container.appendChild(textarea);
    state.rawEditorTextarea = textarea;
    return;
  }

  await loadMonaco();
  if (!monaco || !yamlLanguageModule) return;

  if (!yamlLanguageRegistered) {
    monaco.languages.register({ id: 'yaml', extensions: ['.yaml', '.yml'], aliases: ['YAML', 'yaml'] });
    monaco.languages.setMonarchTokensProvider('yaml', yamlLanguageModule.language as monaco.languages.IMonarchLanguage);
    monaco.languages.setLanguageConfiguration('yaml', yamlLanguageModule.conf);
    yamlLanguageRegistered = true;
  }

  state.rawEditor = monaco.editor.create(container, {
    value: '',
    language: 'yaml',
    theme: document.documentElement.classList.contains('dark') ? 'vs-dark' : 'vs',
    automaticLayout: true,
    minimap: { enabled: false },
    fontSize: 14,
    lineNumbers: 'on',
    roundedSelection: false,
    scrollBeyondLastLine: false,
    wordWrap: 'on',
    tabSize: 2,
    insertSpaces: true,
    glyphMargin: false,
    folding: true,
    padding: { top: 16, bottom: 16 },
  });

  const model = state.rawEditor.getModel();
  if (model) {
    monaco.editor.setModelLanguage(model, 'yaml');
  }
};

// ============================================
// 主题管理
// ============================================
const initTheme = () => {
  const savedTheme = loadJson<'auto' | 'dark' | 'light'>('ikb_theme', 'auto');
  applyTheme(savedTheme);
  
  document.getElementById('themeToggle')?.addEventListener('click', () => {
    const current = document.documentElement.dataset.theme as 'auto' | 'dark' | 'light' || 'auto';
    const next = current === 'dark' ? 'light' : current === 'light' ? 'auto' : 'dark';
    applyTheme(next);
    showToast(next === 'auto' ? '已切换到自动主题' : next === 'dark' ? '已切换到深色模式' : '已切换到浅色模式');
  });
};

const applyTheme = (mode: 'auto' | 'dark' | 'light') => {
  const root = document.documentElement;
  root.dataset.theme = mode;
  saveJson('ikb_theme', mode);
  
  const prefersDark = window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches;
  const useDark = mode === 'dark' || (mode === 'auto' && prefersDark);
  
  root.classList.toggle('dark', useDark);
  root.classList.toggle('light', mode === 'light');
  if (monaco) {
    monaco.editor.setTheme(useDark ? 'vs-dark' : 'vs');
  }
};

// ============================================
// 日志系统
// ============================================
const appendLog = (line: string) => {
  const container = document.getElementById('logContainer');
  if (!container) return;
  
  // 解析日志级别并添加颜色
  let className = '';
  if (line.includes('[ERROR]') || line.includes('错误') || line.includes('失败')) {
    className = 'log-error';
  } else if (line.includes('[WARN]') || line.includes('警告')) {
    className = 'log-warn';
  } else if (line.includes('[SUCCESS]') || line.includes('成功') || line.includes('完成')) {
    className = 'log-success';
  } else if (line.includes('[INFO]')) {
    className = 'log-info';
  }
  
  const entry = document.createElement('div');
  entry.className = className;
  entry.textContent = line;
  
  // 移除"等待日志输出..."
  if (container.children.length === 1 && container.children[0].textContent?.includes('等待')) {
    container.innerHTML = '';
  }
  
  container.appendChild(entry);
  
  // 限制日志数量
  while (container.children.length > 200) {
    const first = container.firstChild;
    if (!first) break;
    container.removeChild(first);
  }
  
  // 自动滚动
  container.scrollTop = container.scrollHeight;
};

const clearLogs = () => {
  const container = document.getElementById('logContainer');
  if (!container) return;
  container.innerHTML = '<div class="text-gray-400 dark:text-gray-500 italic">等待日志输出...</div>';
};

const startLogStream = async () => {
  if (state.unlistenLogs) {
    try {
      state.unlistenLogs();
    } catch (err) {
      console.warn('[IKB] Failed to unlisten logs', err);
    }
  }
  
  try {
    state.unlistenLogs = await bridge.listenLogs(
      (rec) => {
        const line = `${rec.ts || ''} [${rec.module || ''}] [${rec.tag || ''}] ${rec.detail || ''}`;
        appendLog(line);
      },
      () => scheduleReconnect()
    );
  } catch (err) {
    console.warn('[IKB] Failed to start log stream', err);
    scheduleReconnect();
  }
};

const scheduleReconnect = () => {
  if (state.streamReconnectTimer) clearTimeout(state.streamReconnectTimer);
  state.streamReconnectTimer = setTimeout(() => {
    startLogStream().catch((err) => {
      console.warn('[IKB] Failed to reconnect log stream', err);
    });
  }, RECONNECT_DELAY);
};

const loadInitialLogs = async () => {
  try {
    const logs = await bridge.runtimeTailLogs(100);
    for (const rec of logs) {
      const line = `${rec.ts || ''} [${rec.module || ''}] [${rec.tag || ''}] ${rec.detail || ''}`;
      appendLog(line);
    }
  } catch (err) {
    console.warn('[IKB] Failed to load initial logs', err);
  }
};

// ============================================
// 状态管理
// ============================================
const updateStatus = async () => {
  try {
    const st = await bridge.runtimeStatus();
    state.isRunning = st.running;
    state.isCronRunning = st.cron_running;
    state.lastRuntimeStatus = st;
    
    const badge = document.getElementById('statusBadge');
    const mainStatus = document.getElementById('mainStatus');
    const subStatus = document.getElementById('subStatus');
    
    if (!badge || !mainStatus || !subStatus) return;
    
    // 更新徽章
    if (st.running) {
      badge.className = 'px-3 py-1 rounded-full text-xs font-semibold status-running flex items-center gap-1.5';
      badge.innerHTML = '<span class="w-2 h-2 rounded-full bg-emerald-500 animate-pulse"></span>运行中';
      mainStatus.textContent = '执行中';
    } else if (st.cron_running) {
      badge.className = 'px-3 py-1 rounded-full text-xs font-semibold status-running flex items-center gap-1.5';
      badge.innerHTML = '<span class="w-2 h-2 rounded-full bg-emerald-500 animate-pulse"></span>定时运行';
      mainStatus.textContent = '待机';
    } else {
      badge.className = 'px-3 py-1 rounded-full text-xs font-semibold status-stopped flex items-center gap-1.5';
      badge.innerHTML = '<span class="w-2 h-2 rounded-full bg-red-500"></span>已停止';
      mainStatus.textContent = '已停止';
    }
    
    // 更新副状态
    if (st.running) {
      subStatus.textContent = `正在执行模块: ${st.module || state.selectedModule}`;
    } else if (st.cron_running && st.next_run_at) {
      subStatus.textContent = `定时模块: ${st.module || state.selectedModule} / 下次执行: ${st.next_run_at}`;
    } else if (st.last_run_at) {
      subStatus.textContent = `上次执行: ${st.last_run_at}`;
    } else {
      subStatus.textContent = '等待启动...';
    }
    
    // 更新 Cron 按钮状态
    updateCronButton();
     
  } catch (e) {
    state.lastRuntimeStatus = null;
    const badge = document.getElementById('statusBadge');
    const mainStatus = document.getElementById('mainStatus');
    const subStatus = document.getElementById('subStatus');
    
    if (badge) {
      badge.className = 'px-3 py-1 rounded-full text-xs font-semibold status-pending flex items-center gap-1.5';
      badge.innerHTML = '<span class="w-2 h-2 rounded-full bg-amber-500 animate-pulse"></span>连接中';
    }
    if (mainStatus) mainStatus.textContent = '--';
    if (subStatus) subStatus.textContent = '无法连接到服务';
  }
};

const updateCronButton = () => {
  const stopBtn = document.getElementById('btnStopAction') as HTMLButtonElement | null;
  const runBtn = document.getElementById('btnRunAction') as HTMLButtonElement | null;
  if (stopBtn) {
    stopBtn.remove();
  }
  if (!runBtn) return;

  if (state.isRunning || state.isCronRunning) {
    runBtn.textContent = state.isCronRunning ? '停止定时任务' : '停止执行';
    runBtn.dataset.action = 'stop';
    runBtn.classList.remove('bg-primary-600', 'hover:bg-primary-700', 'shadow-primary-600/30');
    runBtn.classList.add('bg-red-500', 'hover:bg-red-600', 'shadow-red-500/30');
    return;
  }

  runBtn.dataset.action = 'run';
  runBtn.classList.remove('bg-red-500', 'hover:bg-red-600', 'shadow-red-500/30');
  runBtn.classList.add('bg-primary-600', 'hover:bg-primary-700', 'shadow-primary-600/30');

  const labels: Record<typeof state.selectedRunMode, string> = {
    once: '执行一次',
    cron: '启动 cron',
    cronAft: '启动 cronAft',
    clean: '执行清理',
  };
  runBtn.textContent = labels[state.selectedRunMode];
};

const setRunningPreview = (text: string) => {
  const badge = document.getElementById('statusBadge');
  const mainStatus = document.getElementById('mainStatus');
  const subStatus = document.getElementById('subStatus');

  if (badge) {
    badge.className = 'px-3 py-1 rounded-full text-xs font-semibold status-running flex items-center gap-1.5';
    badge.innerHTML = '<span class="w-2 h-2 rounded-full bg-emerald-500 animate-pulse"></span>运行中';
  }
  if (mainStatus) mainStatus.textContent = '执行中';
  if (subStatus) subStatus.textContent = text;
};

const updateRunModeUI = () => {
  document.querySelectorAll('.run-mode-chip').forEach((chip) => {
    chip.classList.toggle('active', chip.getAttribute('data-run-mode') === state.selectedRunMode);
  });

  const showCron = state.selectedRunMode === 'cron' || state.selectedRunMode === 'cronAft';
  const showClean = state.selectedRunMode === 'clean';
  document.getElementById('cronSection')?.classList.toggle('hidden', !showCron);
  document.getElementById('cleanSection')?.classList.toggle('hidden', !showClean);

  const runAction = document.getElementById('btnRunAction');
  if (runAction) updateCronButton();
};

const initMainTabs = () => {
  const tabs = document.querySelectorAll<HTMLElement>('.main-tab');
  const panels = document.querySelectorAll<HTMLElement>('.tab-panel');

  tabs.forEach((tab) => {
    tab.addEventListener('click', () => {
      const tabName = tab.dataset.tab || 'runtime';
      tabs.forEach((item) => item.classList.toggle('active', item === tab));
      panels.forEach((panel) => panel.classList.toggle('hidden', panel.dataset.tabPanel !== tabName));
    });
  });
};

const applyRawEditorToState = () => {
  const editor = state.rawEditor;
  const hint = document.getElementById('rawEditorHint');
  if (!editor && !state.rawEditorTextarea) return true;

  try {
    state.rawYaml = getRawEditorValue();
    applyStateFromRawYaml();
    if (hint) hint.textContent = 'YAML 已同步到表单。';
    bindConfigFields();
    renderCmd();
    return true;
  } catch (err) {
    if (hint) hint.textContent = `YAML 解析失败: ${getErrorMessage(err)}`;
    showToast('请先修正 YAML');
    return false;
  }
};

const updateConfigSubTabUI = async () => {
  document.querySelectorAll<HTMLElement>('.config-sub-tab').forEach((tab) => {
    tab.classList.toggle('active', tab.dataset.configTab === state.selectedConfigTab);
  });
  document.querySelectorAll<HTMLElement>('.config-sub-panel').forEach((panel) => {
    panel.classList.toggle('hidden', panel.dataset.configPanel !== state.selectedConfigTab);
  });
  if (state.selectedConfigTab === 'raw') {
    await openRawEditor();
    requestAnimationFrame(() => state.rawEditor?.layout());
  }
};

const initConfigSubTabs = () => {
  document.querySelectorAll<HTMLElement>('.config-sub-tab').forEach((tab) => {
    tab.addEventListener('click', () => {
      const nextTab = (tab.dataset.configTab as 'visual' | 'raw') || 'visual';
      if (nextTab === state.selectedConfigTab) return;
      if (state.selectedConfigTab === 'raw' && nextTab === 'visual' && !applyRawEditorToState()) {
        return;
      }
      state.selectedConfigTab = nextTab;
      void updateConfigSubTabUI();
    });
  });
  void updateConfigSubTabUI();
};

const initRunModeSelection = () => {
  document.querySelectorAll('.run-mode-chip').forEach((chip) => {
    chip.addEventListener('click', () => {
      if (state.isRunning || state.isCronRunning) {
        showToast('请先停止任务');
        return;
      }
      state.selectedRunMode = (chip.getAttribute('data-run-mode') as typeof state.selectedRunMode) || 'once';
      updateRunModeUI();
    });
  });
  updateRunModeUI();
};

// ============================================
// 模块选择
// ============================================
const initModuleSelection = () => {
  const grid = document.getElementById('moduleGrid');
  if (!grid) return;
  
  grid.querySelectorAll('.module-chip').forEach(chip => {
    chip.addEventListener('click', () => {
      if (state.isRunning || state.isCronRunning) {
        showToast('请先停止任务');
        return;
      }
      grid.querySelectorAll('.module-chip').forEach(c => c.classList.remove('active'));
      chip.classList.add('active');
      state.selectedModule = chip.getAttribute('data-module') || 'ispdomain';
    });
  });
};

// ============================================
// 快速操作
// ============================================
const initQuickActions = () => {
  document.getElementById('btnRunAction')?.addEventListener('click', async () => {
    try {
      const action = (document.getElementById('btnRunAction') as HTMLButtonElement | null)?.dataset.action || 'run';
      if (action === 'stop') {
        if (state.isCronRunning) {
          await openStopCronModal();
          return;
        }
        await bridge.runtimeStop();
        showToast('任务已停止');
        return;
      }

      if (state.selectedRunMode === 'clean') {
        const cleanTag = (document.getElementById('cleanTagInput') as HTMLInputElement | null)?.value.trim() || '';
        if (!cleanTag) {
          showToast('clean 模式必须填写清理标签');
          return;
        }
        showToast('正在执行清理...');
        await bridge.runtimeClean(cleanTag);
        showToast('清理完成');
        return;
      }

      if (state.selectedRunMode === 'once') {
        showToast('正在启动...');
        await bridge.runtimeRunOnce(state.selectedModule);
        state.isRunning = true;
        setRunningPreview(`正在执行模块: ${state.selectedModule}`);
        showToast('任务已启动');
        return;
      }

      const expr = (document.getElementById('cronInput') as HTMLInputElement | null)?.value || state.cfg.cron;
      if (!expr.trim()) {
        showToast('请先设置 Cron 表达式');
        return;
      }

      if (state.selectedRunMode === 'cron') {
        showToast('正在执行并启动定时...');
        await bridge.runtimeRunOnce(state.selectedModule);
        await bridge.runtimeCronStart(expr, state.selectedModule);
        state.isRunning = true;
        state.isCronRunning = true;
        setRunningPreview(`正在执行模块: ${state.selectedModule}`);
        showToast('已进入 cron 模式');
        return;
      }

      await bridge.runtimeCronStart(expr, state.selectedModule);
      state.isCronRunning = true;
      updateCronButton();
      showToast('已进入 cronAft 模式');
    } catch (err) {
      showToast('启动失败: ' + getErrorMessage(err));
    }
  });

  document.getElementById('btnClearLogs')?.addEventListener('click', () => {
    clearLogs();
    showToast('日志已清空');
  });
};

// ============================================
// 配置 Modal
// ============================================
const initGhProxyPickers = () => {
  mountGhProxyQuickPick('cfgGhProxy', 'cfgGhProxyQuickPick');
  mountGhProxyQuickPick('remoteGhProxy', 'remoteGhProxyQuickPick');

  const use = document.getElementById('remoteUseGhProxy') as HTMLInputElement | null;
  const section = document.getElementById('remoteGhProxySection');
  const proxyInput = document.getElementById('remoteGhProxy') as HTMLInputElement | null;

  const apply = () => {
    if (!use || !section) return;
    section.classList.toggle('hidden', !use.checked);
  };

  if (use) {
    use.checked = loadJson<boolean>('ikb_remote_use_ghproxy', false);
    use.addEventListener('change', () => {
      saveJson('ikb_remote_use_ghproxy', use.checked);
      apply();
    });
  }

  if (proxyInput) {
    const saved = loadJson<string>('ikb_remote_ghproxy', '');
    if (!proxyInput.value.trim() && saved) proxyInput.value = saved;
    proxyInput.addEventListener('input', () => {
      saveJson('ikb_remote_ghproxy', proxyInput.value);
    });
  }

  apply();
};

const initBasicConfigAccordion = () => {
  const btn = document.getElementById('btnToggleBasicConfig') as HTMLButtonElement | null;
  const panel = document.getElementById('basicConfigAccordionPanel');
  const label = document.getElementById('basicConfigToggleLabel');
  const icon = document.getElementById('basicConfigToggleIcon');
  if (!btn || !panel) return;

  // 默认折叠，不持久化展开状态。
  // Default collapsed; do not persist expanded state.
  let open = false;

  const setPanelDisabled = (disabled: boolean) => {
    panel
      .querySelectorAll<HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement>('input, textarea, select')
      .forEach((el) => {
        el.disabled = disabled;
      });
  };

  const apply = (next: boolean) => {
    open = next;
    btn.setAttribute('aria-expanded', open ? 'true' : 'false');
    panel.setAttribute('aria-hidden', open ? 'false' : 'true');
    panel.classList.toggle('grid-rows-[1fr]', open);
    panel.classList.toggle('grid-rows-[0fr]', !open);
    panel.classList.toggle('opacity-100', open);
    panel.classList.toggle('opacity-0', !open);
    panel.classList.toggle('pointer-events-none', !open);
    if (label) label.textContent = open ? '收起' : '展开';
    if (icon) icon.classList.toggle('rotate-180', open);
    setPanelDisabled(!open);

    if (open) {
      bindConfigFields();
      ghProxyPickerRefresh.get('cfgGhProxy')?.();
    }
  };

  apply(open);
  btn.addEventListener('click', () => {
    apply(!open);
  });
};

const initConfigModal = () => {
  initGhProxyPickers();

  initBasicConfigAccordion();

  document.getElementById('btnTestIkuaiLogin')?.addEventListener('click', async () => {
    const url = (document.getElementById('cfgIkuaiUrl') as HTMLInputElement | null)?.value.trim() || '';
    const user = (document.getElementById('cfgUser') as HTMLInputElement | null)?.value.trim() || '';
    const pass = (document.getElementById('cfgPass') as HTMLInputElement | null)?.value || '';
    const hint = document.getElementById('ikuaiTestHint');
    const btn = document.getElementById('btnTestIkuaiLogin') as HTMLButtonElement | null;

    if (!url || !user || !pass) {
      showToast('请填写路由器地址/用户名/密码', 2600);
      if (hint) hint.textContent = '请先补全连接信息';
      return;
    }

    if (btn) {
      btn.disabled = true;
      btn.textContent = '测试中...';
    }
    if (hint) hint.textContent = '正在测试...';

    try {
      const r = await bridge.testIkuaiLogin(url, user, pass);
      if (r.ok) {
        showToast('连接成功');
        if (hint) hint.textContent = '连接成功';
      } else {
        showToast('连接失败: ' + (r.message || 'unknown error'), 3600);
        if (hint) hint.textContent = '连接失败';
      }
    } catch (err) {
      showToast('测试失败: ' + getErrorMessage(err), 3600);
      if (hint) hint.textContent = '测试失败';
    } finally {
      if (btn) {
        btn.disabled = false;
        btn.textContent = '测试连接';
      }
    }
  });

  document.getElementById('btnOpenRemoteConfig')?.addEventListener('click', () => {
    setRemoteTemplateTip(configMissingDetected ? '检测到当前配置缺失/为空，建议通过远程载入模板初始化，然后修改 iKuai 连接信息并保存。' : null);

    const remoteProxy = document.getElementById('remoteGhProxy') as HTMLInputElement | null;
    if (remoteProxy && !remoteProxy.value.trim() && state.cfg.githubProxy.trim()) {
      remoteProxy.value = state.cfg.githubProxy.trim();
      remoteProxy.dispatchEvent(new Event('input', { bubbles: true }));
    }
    openModal('remoteConfigModal');
  });
  document.getElementById('btnCloseRemoteConfig')?.addEventListener('click', () => {
    closeModal('remoteConfigModal');
  });
  document.getElementById('remoteConfigBackdrop')?.addEventListener('click', () => {
    closeModal('remoteConfigModal');
  });

  document.getElementById('btnLoadRemote')?.addEventListener('click', loadRemoteConfig);
  document.getElementById('btnCloseRuleEditor')?.addEventListener('click', () => closeModal('ruleEditorModal'));
  document.getElementById('btnCancelRuleEditor')?.addEventListener('click', () => closeModal('ruleEditorModal'));
  document.getElementById('ruleEditorBackdrop')?.addEventListener('click', () => closeModal('ruleEditorModal'));
  document.getElementById('btnResetRemote')?.addEventListener('click', () => {
    const def = DEFAULT_REMOTE_TEMPLATE_URL;
    const input = document.getElementById('remoteUrl') as HTMLInputElement | null;
    if (input) input.value = def;
    saveJson('ikb_remote_url', def);
    showToast('已恢复默认地址');
  });
  
  document.getElementById('btnSaveNoComments')?.addEventListener('click', () => saveConfig(false));
  document.getElementById('btnSaveWithComments')?.addEventListener('click', () => saveConfig(true));

  const liveSyncIds = [
    'cfgIkuaiUrl',
    'cfgUser',
    'cfgPass',
    'cfgGhProxy',
    'cfgRetryWait',
    'cfgAddWait',
    'cfgCronInline',
    'cfgWebPort',
    'cfgWebCdn',
    'cfgWebUser',
    'cfgWebPass',
    'cfgMaxIsp',
    'cfgMaxIpv4',
    'cfgMaxIpv6',
    'cfgMaxDomain',
  ];

  liveSyncIds.forEach((id) => {
    document.getElementById(id)?.addEventListener('input', () => {
      commitBasicConfigToRawYaml();
    });
  });

  ['cfgWebEnable'].forEach((id) => {
    document.getElementById(id)?.addEventListener('change', () => {
      commitBasicConfigToRawYaml();
    });
  });

  document.getElementById('addCustomIsp')?.addEventListener('click', () => {
    openRuleEditor('customIsp', -1, false);
  });
  document.getElementById('addIpGroup')?.addEventListener('click', () => {
    openRuleEditor('ipGroup', -1, false);
  });
  document.getElementById('addIpv6Group')?.addEventListener('click', () => {
    openRuleEditor('ipv6Group', -1, false);
  });
  document.getElementById('addStreamDomain')?.addEventListener('click', () => {
    openRuleEditor('streamDomain', -1, false);
  });
  document.getElementById('addStreamIpPort')?.addEventListener('click', () => {
    openRuleEditor('streamIpPort', -1, false);
  });
};

const bindConfigFields = () => {
  // 基础字段
  const setValue = (id: string, value: string) => {
    const el = document.getElementById(id) as HTMLInputElement;
    if (el) el.value = value;
  };
  
  setValue('cfgIkuaiUrl', state.cfg.ikuaiUrl);
  setValue('cfgUser', state.cfg.username);
  setValue('cfgPass', state.cfg.password);
  setValue('cfgGhProxy', state.cfg.githubProxy);
  ghProxyPickerRefresh.get('cfgGhProxy')?.();
  setValue('cfgRetryWait', state.cfg.addErrRetryWait);
  setValue('cfgAddWait', state.cfg.addWait);
  setValue('cfgCronInline', state.cfg.cron);
  
  // WebUI
  const webEnable = document.getElementById('cfgWebEnable') as HTMLInputElement;
  if (webEnable) webEnable.checked = state.cfg.webui.enable;
  setValue('cfgWebPort', state.cfg.webui.port);
  setValue('cfgWebCdn', state.cfg.webui.cdnPrefix);
  setValue('cfgWebUser', state.cfg.webui.user);
  setValue('cfgWebPass', state.cfg.webui.pass);
  
  // 数据限制
  setValue('cfgMaxIsp', String(state.cfg.maxNumberOfOneRecords.Isp));
  setValue('cfgMaxIpv4', String(state.cfg.maxNumberOfOneRecords.Ipv4));
  setValue('cfgMaxIpv6', String(state.cfg.maxNumberOfOneRecords.Ipv6));
  setValue('cfgMaxDomain', String(state.cfg.maxNumberOfOneRecords.Domain));
  
  // 远程 URL
  const savedUrl = loadJson('ikb_remote_url', DEFAULT_REMOTE_TEMPLATE_URL);
  setValue('remoteUrl', savedUrl);
  
  // 渲染列表
  renderCustomIspList();
  renderIpGroupList();
  renderIpv6GroupList();
  renderStreamDomainList();
  renderStreamIpPortList();
};

const syncConfigFromInputs = () => {
  const getValue = (id: string) => {
    const el = document.getElementById(id) as HTMLInputElement;
    return el?.value || '';
  };
  
  state.cfg.ikuaiUrl = getValue('cfgIkuaiUrl');
  state.cfg.username = getValue('cfgUser');
  state.cfg.password = getValue('cfgPass');
  state.cfg.githubProxy = getValue('cfgGhProxy');
  state.cfg.addErrRetryWait = getValue('cfgRetryWait');
  state.cfg.addWait = getValue('cfgAddWait');
  state.cfg.cron = getValue('cfgCronInline') || state.cfg.cron;
  
  const webEnable = document.getElementById('cfgWebEnable') as HTMLInputElement;
  state.cfg.webui.enable = webEnable?.checked || false;
  state.cfg.webui.port = getValue('cfgWebPort');
  state.cfg.webui.cdnPrefix = getValue('cfgWebCdn');
  state.cfg.webui.user = getValue('cfgWebUser');
  state.cfg.webui.pass = getValue('cfgWebPass');
  
  state.cfg.maxNumberOfOneRecords.Isp = Number(getValue('cfgMaxIsp')) || 5000;
  state.cfg.maxNumberOfOneRecords.Ipv4 = Number(getValue('cfgMaxIpv4')) || 1000;
  state.cfg.maxNumberOfOneRecords.Ipv6 = Number(getValue('cfgMaxIpv6')) || 1000;
  state.cfg.maxNumberOfOneRecords.Domain = Number(getValue('cfgMaxDomain')) || 5000;
};

const commitBasicConfigToRawYaml = () => {
  syncConfigFromInputs();
  state.rawYaml = updateYamlPaths(state.rawYaml, [
    { path: ['ikuai-url'], value: state.cfg.ikuaiUrl },
    { path: ['username'], value: state.cfg.username },
    { path: ['password'], value: state.cfg.password },
    { path: ['cron'], value: state.cfg.cron },
    { path: ['github-proxy'], value: state.cfg.githubProxy },
    { path: ['AddErrRetryWait'], value: state.cfg.addErrRetryWait },
    { path: ['AddWait'], value: state.cfg.addWait },
    { path: ['webui', 'enable'], value: state.cfg.webui.enable },
    { path: ['webui', 'port'], value: state.cfg.webui.port },
    { path: ['webui', 'user'], value: state.cfg.webui.user },
    { path: ['webui', 'pass'], value: state.cfg.webui.pass },
    { path: ['webui', 'cdn-prefix'], value: state.cfg.webui.cdnPrefix },
    { path: ['MaxNumberOfOneRecords', 'Isp'], value: state.cfg.maxNumberOfOneRecords.Isp },
    { path: ['MaxNumberOfOneRecords', 'Ipv4'], value: state.cfg.maxNumberOfOneRecords.Ipv4 },
    { path: ['MaxNumberOfOneRecords', 'Ipv6'], value: state.cfg.maxNumberOfOneRecords.Ipv6 },
    { path: ['MaxNumberOfOneRecords', 'Domain'], value: state.cfg.maxNumberOfOneRecords.Domain },
  ]);
  applyStateFromRawYaml();
  refreshEditorFromRawYaml();
};

const saveConfig = async (withComments: boolean) => {
  try {
    if (state.selectedConfigTab === 'raw' && !applyRawEditorToState()) {
      return;
    }
    if (state.selectedConfigTab === 'visual') {
      commitBasicConfigToRawYaml();
    }
    await bridge.saveRawYaml(state.rawYaml, withComments);
    showToast(withComments ? '配置已保存(带注释)' : '配置已保存');
    await loadBackend();
  } catch (err) {
    showToast('保存失败: ' + getErrorMessage(err), 3200);
  }
};

const loadRemoteConfig = async () => {
  const input = document.getElementById('remoteUrl') as HTMLInputElement | null;
  const hint = document.getElementById('remoteHint');
  const url = input?.value.trim() || '';

  if (!url) {
    if (hint) hint.textContent = '请输入 URL';
    return;
  }

  const useGhProxy = (document.getElementById('remoteUseGhProxy') as HTMLInputElement | null)?.checked || false;
  const ghProxyInput = document.getElementById('remoteGhProxy') as HTMLInputElement | null;
  const preferredProxy = ghProxyInput?.value || '';

  const candidates = buildGhProxyCandidates(url, useGhProxy, preferredProxy, state.cfg.githubProxy);
  const isTauri = await bridge.isTauriReady();

  if (hint) hint.textContent = '正在加载...';

  try {
    let lastErr: unknown = null;
    for (let i = 0; i < candidates.length; i++) {
      const proxy = candidates[i];
      const label = proxy ? proxy : '直连';
      if (hint) {
        hint.textContent = candidates.length > 1 ? `正在尝试 (${i + 1}/${candidates.length}): ${label}` : '正在加载...';
      }

      try {
        const text = isTauri
          ? await bridge.fetchRemoteConfig(url, proxy)
          : await (async () => {
              const finalUrl = proxy && isGithubUrlForProxy(url) ? applyGhProxy(proxy, url) : url;
              const r = await fetch(finalUrl);
              if (!r.ok) throw new Error('HTTP ' + r.status + ' ' + r.statusText);
              return await r.text();
            })();

        // 成功：写入 rawYaml，并在启用代理时同步覆盖 github-proxy 字段
        state.rawYaml = text;
        if (proxy && isGithubUrlForProxy(url)) {
          const normalized = normalizeGhProxy(proxy);
          state.rawYaml = updateYamlPaths(state.rawYaml, [{ path: ['github-proxy'], value: normalized }]);
          if (ghProxyInput) {
            ghProxyInput.value = normalized;
            ghProxyInput.dispatchEvent(new Event('input', { bubbles: true }));
          }
        }

        applyStateFromRawYaml();
        bindConfigFields();
        renderCmd();
        refreshEditorFromRawYaml();
        saveJson('ikb_remote_url', url);
        if (hint) hint.textContent = '加载成功';
        setRemoteTemplateTip(null);
        showToast('远程配置已加载');
        return;
      } catch (err) {
        lastErr = err;
      }
    }

    throw lastErr || new Error('加载失败');
  } catch (err) {
    if (hint) hint.textContent = '加载失败: ' + getErrorMessage(err);
    showToast('加载失败');
  }
};

// ============================================
// 列表渲染
// ============================================
type RuleField = {
  key: string;
  label: string;
  placeholder?: string;
  fullRow?: boolean;
  type?: 'text' | 'select' | 'toggle';
  options?: Array<{ value: string; label: string }>;
};

type RuleItemByKey = {
  customIsp: UiConfig['customIsp'][number];
  ipGroup: UiConfig['ipGroup'][number];
  ipv6Group: UiConfig['ipv6Group'][number];
  streamDomain: UiConfig['streamDomain'][number];
  streamIpPort: UiConfig['streamIpPort'][number];
};

type RuleListKey = keyof RuleItemByKey;
type RuleDraft = Record<string, string>;
type RuleMetaItem = { label: string; value: string };

const RULE_LIST_META: Record<RuleListKey, {
  title: string;
  fields: RuleField[];
}> = {
  customIsp: {
    title: '自定义运营商',
    fields: [
      { key: 'tag', label: '标签', placeholder: '例如：telegram' },
      { key: 'url', label: '订阅地址', placeholder: 'https://raw.githubusercontent.com/...', fullRow: true },
    ],
  },
  ipGroup: {
    title: 'IPv4 分组',
    fields: [
      { key: 'tag', label: '标签', placeholder: '例如：国内' },
      { key: 'url', label: '订阅地址', placeholder: 'https://raw.githubusercontent.com/...', fullRow: true },
    ],
  },
  ipv6Group: {
    title: 'IPv6 分组',
    fields: [
      { key: 'tag', label: '标签', placeholder: '例如：国内v6' },
      { key: 'url', label: '订阅地址', placeholder: 'https://raw.githubusercontent.com/...', fullRow: true },
    ],
  },
  streamDomain: {
    title: '域名分流',
    fields: [
      { key: 'tag', label: '标签', placeholder: '例如：gfw' },
      { key: 'interface', label: '出站接口', placeholder: '例如：wan2' },
      { key: 'srcAddr', label: '源地址', placeholder: '可选，支持单 IP 或范围' },
      { key: 'srcAddrOptIpGroup', label: '源地址 IP 分组', placeholder: '可选，填写已存在分组名' },
      { key: 'url', label: '域名列表地址', placeholder: 'https://raw.githubusercontent.com/...', fullRow: true },
    ],
  },
  streamIpPort: {
    title: 'IP / 端口分流',
    fields: [
      { key: 'optTagName', label: '规则名称', placeholder: '可选，用于识别这条规则' },
      {
        key: 'type',
        label: '分流类型',
        type: 'select',
        options: [
          { value: '0', label: '0 - 外网线路' },
          { value: '1', label: '1 - 下一跳网关' },
        ],
      },
      { key: 'interface', label: '接口', placeholder: 'type=0 时填写，例如：wan1' },
      { key: 'nexthop', label: '下一跳', placeholder: 'type=1 时填写，例如：192.168.1.2' },
      { key: 'ipGroup', label: '关联 IP 分组', placeholder: '例如：国内流量' },
      { key: 'srcAddr', label: '源地址', placeholder: '可选，支持单 IP 或范围' },
      { key: 'srcAddrOptIpGroup', label: '源地址 IP 分组', placeholder: '可选，填写已存在分组名' },
      {
        key: 'mode',
        label: '负载模式',
        type: 'select',
        fullRow: true,
        options: [
          { value: '0', label: '0 - 新建连接数' },
          { value: '1', label: '1 - 源IP' },
          { value: '2', label: '2 - 源IP+源端口' },
          { value: '3', label: '3 - 源IP+目的IP' },
          { value: '4', label: '4 - 源IP+目的IP+目的端口' },
          { value: '5', label: '5 - 主备模式' },
        ],
      },
      { key: 'ifaceband', label: '线路绑定', type: 'toggle', fullRow: true },
    ],
  },
};

const createEmptyState = (text: string) => {
  const div = document.createElement('div');
  div.className = 'rounded-2xl border border-dashed border-gray-200 bg-white/50 px-4 py-6 text-center text-sm text-gray-400 dark:border-gray-700 dark:bg-gray-900/20 dark:text-gray-500';
  div.textContent = text;
  return div;
};

const getRuleList = <K extends RuleListKey>(listKey: K): RuleItemByKey[K][] => state.cfg[listKey];

const rerenderRuleList = (listKey: RuleListKey) => {
  const renderers: Record<RuleListKey, () => void> = {
    customIsp: renderCustomIspList,
    ipGroup: renderIpGroupList,
    ipv6Group: renderIpv6GroupList,
    streamDomain: renderStreamDomainList,
    streamIpPort: renderStreamIpPortList,
  };
  renderers[listKey]();
};

const getRulePrimaryText = <K extends RuleListKey>(listKey: K, item: RuleItemByKey[K]) => {
  switch (listKey) {
    case 'customIsp':
    case 'ipGroup':
    case 'ipv6Group':
      return item.tag || '--';
    case 'streamDomain':
      return item.tag || '--';
    case 'streamIpPort':
      return item.optTagName || '--';
  }
};

const getRuleSecondaryText = <K extends RuleListKey>(listKey: K, item: RuleItemByKey[K]) => {
  switch (listKey) {
    case 'customIsp':
      return '自定义运营商';
    case 'ipGroup':
      return 'IPv4 分组';
    case 'ipv6Group':
      return 'IPv6 分组';
    case 'streamDomain':
      return `接口 ${item.interface || '--'}`;
    case 'streamIpPort':
      return item.type === '1' ? '下一跳网关' : '外网线路';
  }
};

const getRuleMetaItems = <K extends RuleListKey>(listKey: K, item: RuleItemByKey[K]): RuleMetaItem[] => {
  switch (listKey) {
    case 'customIsp':
    case 'ipGroup':
    case 'ipv6Group':
      return [];
    case 'streamDomain':
      return [
        { label: '源地址/分组', value: item.srcAddrOptIpGroup || item.srcAddr || '--' },
      ];
    case 'streamIpPort':
      return [
        { label: '目标', value: item.type === '1' ? (item.nexthop || '--') : (item.interface || '--') },
        { label: '源地址/分组', value: item.srcAddrOptIpGroup || item.srcAddr || '--' },
      ];
  }
};

const getRuleDetailText = <K extends RuleListKey>(listKey: K, item: RuleItemByKey[K]) => {
  switch (listKey) {
    case 'customIsp':
    case 'ipGroup':
    case 'ipv6Group':
    case 'streamDomain':
      return item.url || '--';
    case 'streamIpPort':
      return item.ipGroup || '--';
  }
};

const createRuleList = <K extends RuleListKey>(listKey: K) => {
  const meta = RULE_LIST_META[listKey];
  const list = getRuleList(listKey);
  if (list.length === 0) {
    return createEmptyState(`暂无${meta.title}数据`);
  }

  const wrap = document.createElement('div');
  wrap.className = 'rule-list';

  list.forEach((item, index) => {
    const row = document.createElement('div');
    row.className = 'rule-list-item';

    const body = document.createElement('div');
    body.className = 'rule-list-body';

    const top = document.createElement('div');
    top.className = 'rule-list-top';

    const heading = document.createElement('div');
    heading.className = 'min-w-0';

    const title = document.createElement('div');
    title.className = 'rule-list-title';
    title.textContent = getRulePrimaryText(listKey, item);

    const subtitle = document.createElement('div');
    subtitle.className = 'rule-list-subtitle';
    subtitle.textContent = getRuleSecondaryText(listKey, item);

    heading.appendChild(title);
    heading.appendChild(subtitle);
    top.appendChild(heading);

    const metaWrap = document.createElement('div');
    metaWrap.className = 'rule-meta-wrap';
    getRuleMetaItems(listKey, item).forEach((metaItem) => {
      const chip = document.createElement('span');
      chip.className = 'rule-meta-chip';
      chip.textContent = `${metaItem.label}: ${metaItem.value}`;
      metaWrap.appendChild(chip);
    });
    if (metaWrap.childElementCount > 0) {
      top.appendChild(metaWrap);
    }

    const detail = document.createElement('div');
    detail.className = 'rule-list-detail';
    detail.textContent = getRuleDetailText(listKey, item);

    const actions = document.createElement('div');
    actions.className = 'rule-list-actions';

    const viewBtn = document.createElement('button');
    viewBtn.type = 'button';
    viewBtn.className = 'rule-inline-btn';
    viewBtn.textContent = '查看';
    viewBtn.addEventListener('click', () => openRuleEditor(listKey, index, true));

    const editBtn = document.createElement('button');
    editBtn.type = 'button';
    editBtn.className = 'rule-inline-btn rule-inline-btn-primary';
    editBtn.textContent = '修改';
    editBtn.addEventListener('click', () => openRuleEditor(listKey, index, false));

    const deleteBtn = document.createElement('button');
    deleteBtn.type = 'button';
    deleteBtn.className = 'rule-inline-btn rule-inline-btn-danger';
    deleteBtn.textContent = '删除';
    deleteBtn.addEventListener('click', () => {
      const pathMap: Record<RuleListKey, string> = {
        customIsp: 'custom-isp',
        ipGroup: 'ip-group',
        ipv6Group: 'ipv6-group',
        streamDomain: 'stream-domain',
        streamIpPort: 'stream-ipport',
      };
      state.rawYaml = removeYamlSeqItem(state.rawYaml, [pathMap[listKey]], index);
      applyStateFromRawYaml();
      bindConfigFields();
      showToast('规则已删除');
    });

    actions.appendChild(viewBtn);
    actions.appendChild(editBtn);
    actions.appendChild(deleteBtn);

    body.appendChild(top);
    body.appendChild(detail);
    row.appendChild(body);
    row.appendChild(actions);
    wrap.appendChild(row);
  });

  return wrap;
};

const openRuleEditor = (listKey: RuleListKey, index: number, readonly: boolean) => {
  const list = getRuleList(listKey);
  const defaults: RuleItemByKey = {
    customIsp: { tag: '', url: '' },
    ipGroup: { tag: '', url: '' },
    ipv6Group: { tag: '', url: '' },
    streamDomain: { interface: 'wan1', srcAddr: '', srcAddrOptIpGroup: '', url: '', tag: '' },
    streamIpPort: { optTagName: '', type: '0', interface: 'wan1', nexthop: '', srcAddr: '', srcAddrOptIpGroup: '', ipGroup: '', mode: '0', ifaceband: '0' },
  };
  const item = index >= 0 ? list[index] : defaults[listKey];
  if (!item) return;

  state.ruleEditor = { listKey, index };
  const meta = RULE_LIST_META[listKey];

  const title = document.getElementById('ruleEditorTitle');
  const subtitle = document.getElementById('ruleEditorSubtitle');
  const form = document.getElementById('ruleEditorForm');
  const saveBtn = document.getElementById('btnSaveRuleEditor') as HTMLButtonElement | null;
  const delBtn = document.getElementById('btnDeleteRuleFromModal') as HTMLButtonElement | null;

  if (!form) return;
  if (title) title.textContent = `${readonly ? '查看' : index >= 0 ? '编辑' : '新增'}${meta.title}`;
  if (subtitle) subtitle.textContent = readonly ? '当前规则的只读视图。' : '修改后会直接写回 YAML。';
  if (saveBtn) saveBtn.classList.toggle('hidden', readonly);
  if (delBtn) delBtn.classList.toggle('hidden', readonly);

  const draft = structuredClone(item) as RuleDraft;
  form.innerHTML = '';

  meta.fields.forEach((field) => {
    const fieldWrap = document.createElement('label');
    fieldWrap.className = field.fullRow ? 'rule-field md:col-span-2' : 'rule-field';

    const label = document.createElement('span');
    label.className = 'rule-label';
    label.textContent = field.label;
    fieldWrap.appendChild(label);

    if (field.type === 'select') {
      const select = document.createElement('select');
      select.className = 'rule-input';
      select.disabled = readonly;
      (field.options || []).forEach((option) => {
        const opt = document.createElement('option');
        opt.value = option.value;
        opt.textContent = option.label;
        select.appendChild(opt);
      });
      select.value = draft[field.key] || '';
      select.addEventListener('change', (e) => {
        draft[field.key] = (e.target as HTMLSelectElement).value;
      });
      fieldWrap.appendChild(select);
    } else if (field.type === 'toggle') {
      const row = document.createElement('div');
      row.className = 'flex items-center justify-between rounded-2xl border border-gray-200/70 bg-white/80 px-4 py-3 dark:border-gray-700 dark:bg-gray-900/60';
      const text = document.createElement('span');
      text.className = 'text-sm text-gray-700 dark:text-gray-300';
      text.textContent = draft[field.key] === '1' ? '已开启' : '已关闭';
      const toggle = document.createElement('label');
      toggle.className = 'toggle-switch';
      const input = document.createElement('input');
      input.type = 'checkbox';
      input.disabled = readonly;
      input.checked = draft[field.key] === '1';
      input.addEventListener('change', (e) => {
        const checked = (e.target as HTMLInputElement).checked;
        draft[field.key] = checked ? '1' : '0';
        text.textContent = checked ? '已开启' : '已关闭';
      });
      const slider = document.createElement('span');
      slider.className = 'toggle-slider';
      toggle.appendChild(input);
      toggle.appendChild(slider);
      row.appendChild(text);
      row.appendChild(toggle);
      fieldWrap.appendChild(row);
    } else {
      const input = document.createElement('input');
      input.type = 'text';
      input.className = 'rule-input';
      input.placeholder = field.placeholder || '';
      input.value = draft[field.key] || '';
      input.readOnly = readonly;
      input.addEventListener('input', (e) => {
        draft[field.key] = (e.target as HTMLInputElement).value;
      });
      fieldWrap.appendChild(input);
    }

    form.appendChild(fieldWrap);
  });

  if (saveBtn) {
    saveBtn.onclick = () => {
      const pathMap: Record<RuleListKey, string> = {
        customIsp: 'custom-isp',
        ipGroup: 'ip-group',
        ipv6Group: 'ipv6-group',
        streamDomain: 'stream-domain',
        streamIpPort: 'stream-ipport',
      };
      const payloadMap: Record<RuleListKey, Record<string, string | number>> = {
        customIsp: draft,
        ipGroup: draft,
        ipv6Group: draft,
        streamDomain: {
          interface: draft.interface,
          'src-addr': draft.srcAddr,
          'src-addr-opt-ipgroup': draft.srcAddrOptIpGroup,
          url: draft.url,
          tag: draft.tag,
        },
        streamIpPort: {
          'opt-tagname': draft.optTagName,
          type: draft.type,
          interface: draft.interface,
          nexthop: draft.nexthop,
          'src-addr': draft.srcAddr,
          'src-addr-opt-ipgroup': draft.srcAddrOptIpGroup,
          'ip-group': draft.ipGroup,
          mode: Number(draft.mode || 0),
          ifaceband: Number(draft.ifaceband || 0),
        },
      };
      state.rawYaml = upsertYamlSeqItem(state.rawYaml, [pathMap[listKey]], index, payloadMap[listKey]);
      applyStateFromRawYaml();
      bindConfigFields();
      closeModal('ruleEditorModal');
      showToast('规则已更新');
    };
  }

  if (delBtn) {
    delBtn.onclick = () => {
      const pathMap: Record<RuleListKey, string> = {
        customIsp: 'custom-isp',
        ipGroup: 'ip-group',
        ipv6Group: 'ipv6-group',
        streamDomain: 'stream-domain',
        streamIpPort: 'stream-ipport',
      };
      state.rawYaml = removeYamlSeqItem(state.rawYaml, [pathMap[listKey]], index);
      applyStateFromRawYaml();
      bindConfigFields();
      closeModal('ruleEditorModal');
      showToast('规则已删除');
    };
  }

  openModal('ruleEditorModal');
};

const openRawEditor = async () => {
  await ensureRawEditor();
  const value = state.rawYaml || yamlDumpWithComments(toBackendPayload(state.cfg), state.comments);
  setRawEditorValue(value);
  if (state.rawEditor) {
    requestAnimationFrame(() => state.rawEditor?.layout());
  }
  const hint = document.getElementById('rawEditorHint');
  if (hint) {
    hint.textContent = state.rawEditorTextarea
      ? '轻量模式已启用（移动端/低内存）。保存时会校验 YAML。'
      : '保存时会校验 YAML 结构。';
  }
};

const renderCustomIspList = () => {
  const container = document.getElementById('listCustomIsp');
  if (!container) return;
  
  container.innerHTML = '';
  if (state.cfg.customIsp.length === 0) {
    container.appendChild(createEmptyState('暂无自定义运营商规则'));
    return;
  }
  container.appendChild(createRuleList('customIsp'));
};

const renderIpGroupList = () => {
  const container = document.getElementById('listIpGroup');
  if (!container) return;
  
  container.innerHTML = '';
  if (state.cfg.ipGroup.length === 0) {
    container.appendChild(createEmptyState('暂无 IPv4 分组规则'));
    return;
  }
  
  container.appendChild(createRuleList('ipGroup'));
};

const renderIpv6GroupList = () => {
  const container = document.getElementById('listIpv6Group');
  if (!container) return;
  
  container.innerHTML = '';
  if (state.cfg.ipv6Group.length === 0) {
    container.appendChild(createEmptyState('暂无 IPv6 分组规则'));
    return;
  }
  
  container.appendChild(createRuleList('ipv6Group'));
};

const renderStreamDomainList = () => {
  const container = document.getElementById('listStreamDomain');
  if (!container) return;
  
  container.innerHTML = '';
  if (state.cfg.streamDomain.length === 0) {
    container.appendChild(createEmptyState('暂无域名分流规则'));
    return;
  }
  
  container.appendChild(createRuleList('streamDomain'));
};

const renderStreamIpPortList = () => {
  const container = document.getElementById('listStreamIpPort');
  if (!container) return;
  
  container.innerHTML = '';
  if (state.cfg.streamIpPort.length === 0) {
    container.appendChild(createEmptyState('暂无 IP/端口分流规则'));
    return;
  }
  
  container.appendChild(createRuleList('streamIpPort'));
};

// ============================================
// 命令生成器 Modal
// ============================================
const initCmdModal = () => {
  // 命令参数变化时重新渲染
  ['cmdRunMode', 'cmdModule', 'cmdConfigPath', 'cmdCleanTag'].forEach(id => {
    document.getElementById(id)?.addEventListener('change', () => {
      renderCmd();
      persistCmdSettings();
    });
    document.getElementById(id)?.addEventListener('input', () => {
      renderCmd();
      persistCmdSettings();
    });
  });
  
  // 随机后缀切换
  document.querySelectorAll('.cmd-toggle').forEach(btn => {
    btn.addEventListener('click', () => {
      document.querySelectorAll('.cmd-toggle').forEach(b => b.classList.remove('active'));
      btn.classList.add('active');
      renderCmd();
      persistCmdSettings();
    });
  });
  
  // 复制命令
  document.getElementById('btnCopyCmd')?.addEventListener('click', async () => {
    const cmd = document.getElementById('cmdOut')?.textContent || '';
    try {
      await navigator.clipboard.writeText(cmd);
      showToast('命令已复制');
    } catch (err) {
      console.warn('[IKB] Failed to copy command', err);
      showToast('复制失败');
    }
  });
  
  // 保存预设
  document.getElementById('btnSavePreset')?.addEventListener('click', savePreset);
  
  restoreCmdSettings();
  renderCmd();
  renderPresets();
};

const renderCmd = () => {
  const getValue = (id: string) => {
    const el = document.getElementById(id) as HTMLInputElement | HTMLSelectElement;
    return el?.value || '';
  };
  
  const runMode = getValue('cmdRunMode');
  const module = getValue('cmdModule');
  const configPath = getValue('cmdConfigPath') || './config.yml';
  const cleanTag = getValue('cmdCleanTag');
  
  const randBtn = document.querySelector('.cmd-toggle.active');
  const rand = randBtn?.getAttribute('data-value') || '1';
  
  const parts = ['./ikuai-bypass', '-r', runMode, '-c', configPath];
  
  if (runMode === 'clean') {
    if (cleanTag) parts.push('-tag', cleanTag);
  } else if (runMode !== 'web') {
    parts.push('-m', module);
  }
  
  if (state.cfg.ikuaiUrl && state.cfg.username) {
    parts.push('-login', `${state.cfg.ikuaiUrl},${state.cfg.username},${state.cfg.password}`);
  }
  
  parts.push('-isIpGroupNameAddRandomSuff', rand);
  
  const cmdOut = document.getElementById('cmdOut');
  if (cmdOut) cmdOut.textContent = parts.join(' ');
};

const persistCmdSettings = () => {
  const getValue = (id: string) => {
    const el = document.getElementById(id) as HTMLInputElement | HTMLSelectElement;
    return el?.value || '';
  };
  
  const randBtn = document.querySelector('.cmd-toggle.active');
  
  saveJson('ikb_cmd', {
    runMode: getValue('cmdRunMode'),
    module: getValue('cmdModule'),
    cleanTag: getValue('cmdCleanTag'),
    randomSuff: randBtn?.getAttribute('data-value') || '1',
  });
};

const restoreCmdSettings = () => {
  const saved = loadJson('ikb_cmd', { runMode: 'cron', module: 'ispdomain', cleanTag: 'cleanAll', randomSuff: '1' });
  
  const setValue = (id: string, value: string) => {
    const el = document.getElementById(id) as HTMLInputElement | HTMLSelectElement;
    if (el) el.value = value;
  };
  
  setValue('cmdRunMode', saved.runMode);
  setValue('cmdModule', saved.module);
  setValue('cmdCleanTag', saved.cleanTag);
  
  document.querySelectorAll('.cmd-toggle').forEach(btn => {
    btn.classList.toggle('active', btn.getAttribute('data-value') === saved.randomSuff);
  });
};

type CmdPreset = {
  name: string;
  data: {
    runMode: string;
    module: string;
    cleanTag: string;
    randomSuff: string;
  };
};

const savePreset = async () => {
  const presets: CmdPreset[] = loadJson('ikb_cmd_presets', []);
  if (presets.length >= 5) {
    showToast('最多保存 5 个预设');
    return;
  }
  
  const name = prompt('预设名称:', `预设 ${presets.length + 1}`);
  if (!name) return;
  
  const getValue = (id: string) => {
    const el = document.getElementById(id) as HTMLInputElement | HTMLSelectElement;
    return el?.value || '';
  };
  
  const randBtn = document.querySelector('.cmd-toggle.active');
  
  presets.push({
    name,
    data: {
      runMode: getValue('cmdRunMode'),
      module: getValue('cmdModule'),
      cleanTag: getValue('cmdCleanTag'),
      randomSuff: randBtn?.getAttribute('data-value') || '1',
    }
  });
  
  saveJson('ikb_cmd_presets', presets);
  renderPresets();
  showToast('预设已保存');
};

const renderPresets = () => {
  const container = document.getElementById('presetList');
  if (!container) return;
  
  const presets: CmdPreset[] = loadJson('ikb_cmd_presets', []);
  
  if (presets.length === 0) {
    container.innerHTML = '<div class="text-sm text-gray-400 italic">暂无预设</div>';
    return;
  }
  
  container.innerHTML = '';
  presets.forEach((preset, index) => {
    const div = document.createElement('div');
    div.className = 'glass-card rounded-xl p-3 flex items-center justify-between';
    div.innerHTML = `
      <span class="text-sm font-medium text-gray-700 dark:text-gray-300">${preset.name}</span>
      <div class="flex gap-2">
        <button class="text-xs text-primary-600 dark:text-primary-400 font-medium px-2 py-1" data-load="${index}">加载</button>
        <button class="text-xs text-red-500 font-medium px-2 py-1" data-del="${index}">删除</button>
      </div>
    `;
    
    div.querySelector('[data-load]')?.addEventListener('click', () => {
      const setValue = (id: string, value: string) => {
        const el = document.getElementById(id) as HTMLInputElement | HTMLSelectElement;
        if (el) el.value = value;
      };
      
      setValue('cmdRunMode', preset.data.runMode);
      setValue('cmdModule', preset.data.module);
      setValue('cmdCleanTag', preset.data.cleanTag);
      
      document.querySelectorAll('.cmd-toggle').forEach(btn => {
        btn.classList.toggle('active', btn.getAttribute('data-value') === preset.data.randomSuff);
      });
      
      renderCmd();
      showToast('预设已加载');
    });
    
    div.querySelector('[data-del]')?.addEventListener('click', () => {
      presets.splice(index, 1);
      saveJson('ikb_cmd_presets', presets);
      renderPresets();
      showToast('预设已删除');
    });
    
    container.appendChild(div);
  });
};

// ============================================
// 后端数据加载
// ============================================
const loadBackend = async () => {
  try {
    const meta = await bridge.getConfigMeta();
    const metaObj = meta as Record<string, unknown>;
    const rawYamlFromBackend = typeof metaObj.raw_yaml === 'string' ? metaObj.raw_yaml : '';
    configMissingDetected = !rawYamlFromBackend.trim();
    const parsed = fromBackendMeta(meta);
    
    state.cfg = parsed.cfg;
    state.comments = parsed.comments;
    state.confPath = parsed.confPath;
    state.rawYaml = rawYamlFromBackend || yamlDumpWithComments(toBackendPayload(parsed.cfg), parsed.comments);
    applyStateFromRawYaml();
    bindConfigFields();
    renderCmd();
    
    // 更新配置路径显示
    const cmdConfigPath = document.getElementById('cmdConfigPath') as HTMLInputElement;
    if (cmdConfigPath) cmdConfigPath.value = state.confPath || './config.yml';
    
    // 更新副标题
    const subtitle = document.getElementById('subtitle');
    if (subtitle) subtitle.textContent = (await bridge.isTauriReady()) ? 'Tauri App' : 'WebUI';

    // 配置文件缺失/为空时，提示用户通过远程载入模板初始化。
    // When config is missing/empty, guide user to load a template remotely.
    if (configMissingDetected && (await bridge.isTauriReady()) && !configMissingPrompted) {
      configMissingPrompted = true;
      setRemoteTemplateTip('检测到配置文件不存在或为空，是否通过远程配置载入一个模板？载入后请修改 iKuai 地址/用户名/密码，并点击保存配置。');
      const remoteUrlInput = document.getElementById('remoteUrl') as HTMLInputElement | null;
      if (remoteUrlInput && !remoteUrlInput.value.trim()) {
        remoteUrlInput.value = DEFAULT_REMOTE_TEMPLATE_URL;
      }

      // 首次提示时默认打开 ghproxy 开关，减少直连失败概率。
      // Enable ghproxy by default on first prompt.
      try {
        if (localStorage.getItem('ikb_remote_use_ghproxy') == null) {
          const use = document.getElementById('remoteUseGhProxy') as HTMLInputElement | null;
          if (use) {
            use.checked = true;
            use.dispatchEvent(new Event('change', { bubbles: true }));
          }
        }
      } catch {
        // ignore
      }
      openModal('remoteConfigModal');
    }
    
  } catch (err) {
    showToast('加载配置失败: ' + getErrorMessage(err));
  }
};

// ============================================
// 初始化
// ============================================
const init = async () => {
  // 初始化主题
  initTheme();
  initMainTabs();
  initConfigSubTabs();
  initRunModeSelection();
  
  // 初始化模块选择
  initModuleSelection();
  
  // 初始化快速操作
  initQuickActions();
  
  // 初始化 Modal
  initConfigModal();
  initStopCronModal();
  initCmdModal();
  initModalEscape();
  
  // 启动状态更新（不依赖后端数据加载，保证 UI 始终可交互）
  // Status polling is independent — ensures UI stays interactive even if IPC fails
  updateStatus();
  setInterval(updateStatus, 1500);
  
  // 后端数据和日志流的加载允许失败，不阻塞 UI
  // Backend data + log stream: non-blocking, UI must remain interactive on failure
  try {
    await loadBackend();
  } catch (err) {
    console.warn('[IKB] loadBackend failed, UI remains usable', err);
    showToast('配置加载失败，请检查连接');
  }
  
  try {
    await loadInitialLogs();
  } catch (err) {
    console.warn('[IKB] loadInitialLogs failed', err);
  }
  
  // 启动日志流
  startLogStream();
  
  // Cron 输入同步
  const cronInput = document.getElementById('cronInput') as HTMLInputElement;
  if (cronInput && state.cfg.cron) {
    cronInput.value = state.cfg.cron;
  }
  
  console.log('iKuai Bypass App initialized');
};

// 启动应用
init().catch((err) => {
  console.error(err);
  showToast('初始化失败: ' + getErrorMessage(err));
});
