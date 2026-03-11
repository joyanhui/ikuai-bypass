import { bridge } from '../lib/bridge.ts';
import { defaultUiConfig, fromBackendMeta, toBackendPayload, yamlDump, yamlParse } from '../lib/config_model.ts';
import { loadJson, saveJson } from '../lib/storage.ts';

// ============================================
// 全局状态
// ============================================
const state = {
  cfg: defaultUiConfig(),
  comments: { top: {}, item: {}, webui: {}, maxNumberOfOneRecords: {} },
  confPath: '',
  selectedModule: 'ispdomain',
  selectedRunMode: 'once' as 'cron' | 'cronAft' | 'once' | 'clean',
  isRunning: false,
  isCronRunning: false,
  unlistenLogs: null as (() => void) | null,
  streamReconnectTimer: null as ReturnType<typeof setTimeout> | null,
};

const RECONNECT_DELAY = 3000;

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
    document.body.style.overflow = '';
  }, 300);
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
    container.removeChild(container.firstChild!);
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
    try { state.unlistenLogs(); } catch (_) {}
  }
  
  try {
    state.unlistenLogs = await bridge.listenLogs(
      (rec) => {
        const line = `${rec.ts || ''} [${rec.module || ''}] [${rec.tag || ''}] ${rec.detail || ''}`;
        appendLog(line);
      },
      () => scheduleReconnect()
    );
  } catch (_) {
    scheduleReconnect();
  }
};

const scheduleReconnect = () => {
  if (state.streamReconnectTimer) clearTimeout(state.streamReconnectTimer);
  state.streamReconnectTimer = setTimeout(() => {
    startLogStream().catch(() => {});
  }, RECONNECT_DELAY);
};

const loadInitialLogs = async () => {
  try {
    const logs = await bridge.runtimeTailLogs(100);
    for (const rec of logs) {
      const line = `${rec.ts || ''} [${rec.module || ''}] [${rec.tag || ''}] ${rec.detail || ''}`;
      appendLog(line);
    }
  } catch (_) {}
};

// ============================================
// 状态管理
// ============================================
const updateStatus = async () => {
  try {
    const st = await bridge.runtimeStatus();
    state.isRunning = st.running;
    state.isCronRunning = st.cron_running;
    
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
      subStatus.textContent = `下次执行: ${st.next_run_at}`;
    } else if (st.last_run_at) {
      subStatus.textContent = `上次执行: ${st.last_run_at}`;
    } else {
      subStatus.textContent = '等待启动...';
    }
    
    // 更新 Cron 按钮状态
    updateCronButton();
    
  } catch (e) {
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
    runBtn.textContent = state.isCronRunning ? '停止任务' : '停止执行';
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
    clean: '切到命令生成器',
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
    } catch (e: any) {
      showToast('启动失败: ' + (e.message || '未知错误'));
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
const initConfigModal = () => {
  // 打开配置
  document.getElementById('btnOpenConfig')?.addEventListener('click', () => {
    openModal('configModal');
    bindConfigFields();
  });
  
  // 关闭配置
  document.getElementById('btnCloseConfig')?.addEventListener('click', () => {
    closeModal('configModal');
  });
  
  document.getElementById('configModalBackdrop')?.addEventListener('click', () => {
    closeModal('configModal');
  });
  
  // 远程配置
  document.getElementById('btnLoadRemote')?.addEventListener('click', loadRemoteConfig);
  document.getElementById('btnResetRemote')?.addEventListener('click', () => {
    const def = 'https://raw.githubusercontent.com/joyanhui/ikuai-bypass/refs/heads/main/config.yml';
    const input = document.getElementById('remoteUrl') as HTMLInputElement;
    if (input) input.value = def;
    saveJson('ikb_remote_url', def);
    showToast('已恢复默认地址');
  });
  
  // 保存配置
  document.getElementById('btnSaveNoComments')?.addEventListener('click', () => saveConfig(false));
  document.getElementById('btnSaveWithComments')?.addEventListener('click', () => saveConfig(true));
  
  // 添加规则项
  document.getElementById('addCustomIsp')?.addEventListener('click', () => {
    state.cfg.customIsp.push({ tag: '', url: '' });
    renderCustomIspList();
  });
  document.getElementById('addIpGroup')?.addEventListener('click', () => {
    state.cfg.ipGroup.push({ tag: '', url: '' });
    renderIpGroupList();
  });
  document.getElementById('addIpv6Group')?.addEventListener('click', () => {
    state.cfg.ipv6Group.push({ tag: '', url: '' });
    renderIpv6GroupList();
  });
  document.getElementById('addStreamDomain')?.addEventListener('click', () => {
    state.cfg.streamDomain.push({ interface: '', srcAddr: '', srcAddrOptIpGroup: '', url: '', tag: '' });
    renderStreamDomainList();
  });
  document.getElementById('addStreamIpPort')?.addEventListener('click', () => {
    state.cfg.streamIpPort.push({ optTagName: '', type: '0', interface: '', nexthop: '', srcAddr: '', srcAddrOptIpGroup: '', ipGroup: '', mode: '0', ifaceband: '0' });
    renderStreamIpPortList();
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
  setValue('cfgRetryWait', state.cfg.addErrRetryWait);
  setValue('cfgAddWait', state.cfg.addWait);
  setValue('cfgCronInline', state.cfg.cron);
  
  // WebUI
  const webEnable = document.getElementById('cfgWebEnable') as HTMLInputElement;
  const webEnableUpdate = document.getElementById('cfgWebEnableUpdate') as HTMLInputElement;
  if (webEnable) webEnable.checked = state.cfg.webui.enable;
  if (webEnableUpdate) webEnableUpdate.checked = state.cfg.webui.enableUpdate;
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
  const savedUrl = loadJson('ikb_remote_url', 'https://raw.githubusercontent.com/joyanhui/ikuai-bypass/refs/heads/main/config.yml');
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
  const webEnableUpdate = document.getElementById('cfgWebEnableUpdate') as HTMLInputElement;
  state.cfg.webui.enable = webEnable?.checked || false;
  state.cfg.webui.enableUpdate = webEnableUpdate?.checked || false;
  state.cfg.webui.port = getValue('cfgWebPort');
  state.cfg.webui.cdnPrefix = getValue('cfgWebCdn');
  state.cfg.webui.user = getValue('cfgWebUser');
  state.cfg.webui.pass = getValue('cfgWebPass');
  
  state.cfg.maxNumberOfOneRecords.Isp = Number(getValue('cfgMaxIsp')) || 5000;
  state.cfg.maxNumberOfOneRecords.Ipv4 = Number(getValue('cfgMaxIpv4')) || 1000;
  state.cfg.maxNumberOfOneRecords.Ipv6 = Number(getValue('cfgMaxIpv6')) || 1000;
  state.cfg.maxNumberOfOneRecords.Domain = Number(getValue('cfgMaxDomain')) || 5000;
};

const saveConfig = async (withComments: boolean) => {
  try {
    syncConfigFromInputs();
    const payload = toBackendPayload(state.cfg);
    await bridge.saveConfig(payload, withComments);
    showToast(withComments ? '配置已保存(带注释)' : '配置已保存');
    await loadBackend();
  } catch (e: any) {
    showToast('保存失败: ' + (e.message || '未知错误'));
  }
};

const loadRemoteConfig = async () => {
  const input = document.getElementById('remoteUrl') as HTMLInputElement;
  const hint = document.getElementById('remoteHint');
  const url = input?.value.trim();
  
  if (!url) {
    if (hint) hint.textContent = '请输入 URL';
    return;
  }
  
  if (hint) hint.textContent = '正在加载...';
  
  try {
    let finalUrl = url;
    const proxy = state.cfg.githubProxy.trim();
    if (proxy && url.startsWith('https://raw.githubusercontent.com/')) {
      finalUrl = proxy.endsWith('/') ? proxy + url : proxy + '/' + url;
    }
    
    const text = bridge.isTauri() 
      ? await bridge.fetchRemoteConfig(finalUrl, proxy)
      : await (await fetch(finalUrl)).text();
    
    const doc = yamlParse(text) || {};
    const meta = { ...toBackendPayload(state.cfg), ...doc };
    const parsed = fromBackendMeta(meta);
    
    Object.assign(state.cfg, parsed.cfg);
    state.comments = parsed.comments;
    
    bindConfigFields();
    saveJson('ikb_remote_url', url);
    if (hint) hint.textContent = '加载成功';
    showToast('远程配置已加载');
  } catch (e: any) {
    if (hint) hint.textContent = '加载失败: ' + (e.message || '未知错误');
    showToast('加载失败');
  }
};

// ============================================
// 列表渲染
// ============================================
const createRuleItem = (fields: Array<{ key: string; placeholder: string; width?: string }>, 
                        data: any, 
                        onChange: (key: string, value: string) => void,
                        onDelete: () => void) => {
  const div = document.createElement('div');
  div.className = 'rule-item space-y-2';
  
  const grid = document.createElement('div');
  grid.className = 'grid gap-2';
  grid.style.gridTemplateColumns = fields.map(f => f.width || '1fr').join(' ');
  
  fields.forEach(field => {
    const input = document.createElement('input');
    input.type = 'text';
    input.placeholder = field.placeholder;
    input.value = data[field.key] || '';
    input.className = 'bg-transparent border-b border-gray-300 dark:border-gray-600 focus:border-primary-500 dark:focus:border-primary-400 outline-none py-1 text-sm text-gray-900 dark:text-white transition-colors';
    input.addEventListener('input', (e) => {
      onChange(field.key, (e.target as HTMLInputElement).value);
    });
    grid.appendChild(input);
  });
  
  const deleteBtn = document.createElement('button');
  deleteBtn.className = 'text-xs text-red-500 hover:text-red-600 font-medium mt-1';
  deleteBtn.textContent = '删除';
  deleteBtn.addEventListener('click', onDelete);
  
  div.appendChild(grid);
  div.appendChild(deleteBtn);
  
  return div;
};

const renderCustomIspList = () => {
  const container = document.getElementById('listCustomIsp');
  if (!container) return;
  
  container.innerHTML = '';
  if (state.cfg.customIsp.length === 0) {
    container.innerHTML = '<div class="text-sm text-gray-400 italic py-2">暂无规则</div>';
    return;
  }
  
  state.cfg.customIsp.forEach((item, index) => {
    const el = createRuleItem(
      [{ key: 'tag', placeholder: '标签', width: '1fr' }, { key: 'url', placeholder: 'URL', width: '2fr' }],
      item,
      (key, value) => { (item as any)[key] = value; },
      () => { state.cfg.customIsp.splice(index, 1); renderCustomIspList(); }
    );
    container.appendChild(el);
  });
};

const renderIpGroupList = () => {
  const container = document.getElementById('listIpGroup');
  if (!container) return;
  
  container.innerHTML = '';
  if (state.cfg.ipGroup.length === 0) {
    container.innerHTML = '<div class="text-sm text-gray-400 italic py-2">暂无规则</div>';
    return;
  }
  
  state.cfg.ipGroup.forEach((item, index) => {
    const el = createRuleItem(
      [{ key: 'tag', placeholder: '标签', width: '1fr' }, { key: 'url', placeholder: 'URL', width: '2fr' }],
      item,
      (key, value) => { (item as any)[key] = value; },
      () => { state.cfg.ipGroup.splice(index, 1); renderIpGroupList(); }
    );
    container.appendChild(el);
  });
};

const renderIpv6GroupList = () => {
  const container = document.getElementById('listIpv6Group');
  if (!container) return;
  
  container.innerHTML = '';
  if (state.cfg.ipv6Group.length === 0) {
    container.innerHTML = '<div class="text-sm text-gray-400 italic py-2">暂无规则</div>';
    return;
  }
  
  state.cfg.ipv6Group.forEach((item, index) => {
    const el = createRuleItem(
      [{ key: 'tag', placeholder: '标签', width: '1fr' }, { key: 'url', placeholder: 'URL', width: '2fr' }],
      item,
      (key, value) => { (item as any)[key] = value; },
      () => { state.cfg.ipv6Group.splice(index, 1); renderIpv6GroupList(); }
    );
    container.appendChild(el);
  });
};

const renderStreamDomainList = () => {
  const container = document.getElementById('listStreamDomain');
  if (!container) return;
  
  container.innerHTML = '';
  if (state.cfg.streamDomain.length === 0) {
    container.innerHTML = '<div class="text-sm text-gray-400 italic py-2">暂无规则</div>';
    return;
  }
  
  state.cfg.streamDomain.forEach((item, index) => {
    const el = createRuleItem(
      [
        { key: 'tag', placeholder: '标签', width: '1fr' },
        { key: 'interface', placeholder: '接口', width: '1fr' },
        { key: 'url', placeholder: 'URL', width: '2fr' }
      ],
      item,
      (key, value) => { (item as any)[key] = value; },
      () => { state.cfg.streamDomain.splice(index, 1); renderStreamDomainList(); }
    );
    container.appendChild(el);
  });
};

const renderStreamIpPortList = () => {
  const container = document.getElementById('listStreamIpPort');
  if (!container) return;
  
  container.innerHTML = '';
  if (state.cfg.streamIpPort.length === 0) {
    container.innerHTML = '<div class="text-sm text-gray-400 italic py-2">暂无规则</div>';
    return;
  }
  
  state.cfg.streamIpPort.forEach((item, index) => {
    const el = createRuleItem(
      [
        { key: 'optTagName', placeholder: '标签名', width: '1fr' },
        { key: 'type', placeholder: '类型', width: '80px' },
        { key: 'interface', placeholder: '接口', width: '1fr' },
        { key: 'nexthop', placeholder: '下一跳', width: '1fr' }
      ],
      item,
      (key, value) => { (item as any)[key] = value; },
      () => { state.cfg.streamIpPort.splice(index, 1); renderStreamIpPortList(); }
    );
    container.appendChild(el);
  });
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
    } catch (_) {
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
    const parsed = fromBackendMeta(meta);
    
    state.cfg = parsed.cfg;
    state.comments = parsed.comments;
    state.confPath = parsed.confPath;
    bindConfigFields();
    renderCmd();
    
    // 更新配置路径显示
    const cmdConfigPath = document.getElementById('cmdConfigPath') as HTMLInputElement;
    if (cmdConfigPath) cmdConfigPath.value = state.confPath || './config.yml';
    
    // 更新副标题
    const subtitle = document.getElementById('subtitle');
    if (subtitle) subtitle.textContent = bridge.isTauri() ? 'Tauri App' : 'WebUI';
    
  } catch (e: any) {
    showToast('加载配置失败: ' + (e.message || '未知错误'));
  }
};

// ============================================
// 初始化
// ============================================
const init = async () => {
  // 初始化主题
  initTheme();
  initMainTabs();
  initRunModeSelection();
  
  // 初始化模块选择
  initModuleSelection();
  
  // 初始化快速操作
  initQuickActions();
  
  // 初始化 Modal
  initConfigModal();
  initCmdModal();
  
  // 加载后端数据
  await loadBackend();
  
  // 加载初始日志
  await loadInitialLogs();
  
  // 启动日志流
  startLogStream();
  
  // 启动状态更新
  updateStatus();
  setInterval(updateStatus, 1500);
  
  // Cron 输入同步
  const cronInput = document.getElementById('cronInput') as HTMLInputElement;
  if (cronInput && state.cfg.cron) {
    cronInput.value = state.cfg.cron;
  }
  
  console.log('iKuai Bypass App initialized');
};

// 启动应用
init().catch(console.error);
