import { bridge } from '../lib/bridge.ts';
import { defaultUiConfig, fromBackendMeta, toBackendPayload, yamlDump, yamlDumpWithComments, yamlParse } from '../lib/config_model.ts';
import { loadJson, saveJson } from '../lib/storage.ts';

import * as monaco from 'monaco-editor';
import editorWorker from 'monaco-editor/esm/vs/editor/editor.worker?worker';
import yamlWorker from 'monaco-editor/esm/vs/basic-languages/yaml/yaml?worker';

self.MonacoEnvironment = {
  getWorker: function (_workerId: string, label: string) {
    if (label === 'yaml' || label === 'toml') {
      return new yamlWorker();
    }
    return new editorWorker();
  }
};

// ============================================
// Unified Modal System (Replace native confirm/prompt)
// ============================================

type ModalMode = 'confirm' | 'prompt';

interface ModalOptions {
  mode: ModalMode;
  title: string;
  message: string;
  defaultValue?: string;
  confirmText?: string;
  cancelText?: string;
}

let modalResolve: ((value: boolean | string | null) => void) | null = null;

const createModalContainer = (): HTMLElement => {
  const container = document.createElement('div');
  container.id = 'ikb-modal-container';
  container.innerHTML = `
    <div id="ikb-modal-overlay" class="modal-overlay" style="display: none;">
      <div class="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-2xl shadow-2xl max-w-[420px] w-[90%] transform transition-all duration-200 scale-100">
        <div class="px-6 pt-5">
          <span id="ikb-modal-title" class="text-lg font-semibold text-gray-900 dark:text-gray-100"></span>
        </div>
        <div class="px-6 py-4">
          <p id="ikb-modal-message" class="text-sm text-gray-600 dark:text-gray-300 mb-4"></p>
          <input id="ikb-modal-input" type="text" 
            class="w-full px-3.5 py-3 text-sm rounded-lg border border-gray-200 dark:border-gray-600 bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none transition-all" 
            style="display: none;" />
        </div>
        <div class="px-6 pb-6 flex gap-3 justify-end">
          <button id="ikb-modal-cancel" class="px-5 py-2.5 text-sm font-medium rounded-lg bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 text-gray-700 dark:text-gray-300 transition-all border border-gray-200 dark:border-gray-600">取消</button>
          <button id="ikb-modal-confirm" class="px-5 py-2.5 text-sm font-medium rounded-lg bg-blue-600 hover:bg-blue-700 text-white transition-all border border-transparent shadow-sm">确定</button>
        </div>
      </div>
    </div>
  `;
  document.body.appendChild(container);
  
  // Add modal overlay styles dynamically
  const style = document.createElement('style');
  style.textContent = `
    .ikb-modal-open { overflow: hidden; }
    #ikb-modal-overlay {
      position: fixed;
      top: 0;
      left: 0;
      right: 0;
      bottom: 0;
      background: rgba(0, 0, 0, 0.5);
      backdrop-filter: blur(4px);
      -webkit-backdrop-filter: blur(4px);
      display: flex;
      align-items: center;
      justify-content: center;
      z-index: 100;
      animation: ikb-fadeIn 0.15s ease;
    }
    @keyframes ikb-fadeIn { from { opacity: 0; } to { opacity: 1; } }
    #ikb-modal-overlay[style*="display: none"] { display: none !important; }
  `;
  document.head.appendChild(style);
  
  return container;
};

const getModalElements = () => {
  let container = document.getElementById('ikb-modal-container');
  if (!container) {
    container = createModalContainer();
  }
  return {
    overlay: document.getElementById('ikb-modal-overlay') as HTMLElement,
    title: document.getElementById('ikb-modal-title') as HTMLElement,
    message: document.getElementById('ikb-modal-message') as HTMLElement,
    input: document.getElementById('ikb-modal-input') as HTMLInputElement,
    cancelBtn: document.getElementById('ikb-modal-cancel') as HTMLButtonElement,
    confirmBtn: document.getElementById('ikb-modal-confirm') as HTMLButtonElement,
  };
};

const showModal = (options: ModalOptions): Promise<boolean | string | null> => {
  return new Promise((resolve) => {
    modalResolve = resolve;
    const { overlay, title, message, input, cancelBtn, confirmBtn } = getModalElements();
    
    // Set content
    title.textContent = options.title;
    message.textContent = options.message;
    
    // Configure mode
    if (options.mode === 'prompt') {
      input.style.display = '';
      input.value = options.defaultValue || '';
      input.placeholder = '请输入...';
      // Focus input after a small delay for animation
      setTimeout(() => input.focus(), 100);
    } else {
      input.style.display = 'none';
    }
    
    // Set button texts
    cancelBtn.textContent = options.cancelText || '取消';
    confirmBtn.textContent = options.confirmText || '确定';
    
    // Show modal
    overlay.style.display = 'flex';
    document.body.classList.add('ikb-modal-open');
    
    // Handler functions
    const cleanup = () => {
      overlay.style.display = 'none';
      document.body.classList.remove('ikb-modal-open');
      cancelBtn.onclick = null;
      confirmBtn.onclick = null;
      input.onkeydown = null;
      modalResolve = null;
    };
    
    const handleCancel = () => {
      cleanup();
      resolve(false);
    };
    
    const handleConfirm = () => {
      cleanup();
      if (options.mode === 'prompt') {
        const value = input.value.trim();
        resolve(value || null);
      } else {
        resolve(true);
      }
    };
    
    // Attach handlers
    cancelBtn.onclick = handleCancel;
    confirmBtn.onclick = handleConfirm;
    
    // Handle Enter key in prompt mode
    input.onkeydown = (e) => {
      if (e.key === 'Enter') {
        e.preventDefault();
        handleConfirm();
      } else if (e.key === 'Escape') {
        handleCancel();
      }
    };
    
    // Handle Esc key globally
    const handleEsc = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && overlay.style.display !== 'none') {
        handleCancel();
        document.removeEventListener('keydown', handleEsc);
      }
    };
    document.addEventListener('keydown', handleEsc);
    
    // Handle backdrop click
    overlay.onclick = (e) => {
      if (e.target === overlay) {
        handleCancel();
        document.removeEventListener('keydown', handleEsc);
      }
    };
  });
};

// Global exposed functions
declare global {
  interface Window {
    showConfirm: (message: string, title?: string) => Promise<boolean>;
    showPrompt: (message: string, defaultValue?: string, title?: string) => Promise<string | null>;
  }
}

window.showConfirm = (message: string, title: string = '确认'): Promise<boolean> => {
  return showModal({
    mode: 'confirm',
    title,
    message,
    confirmText: '确定',
    cancelText: '取消',
  }) as Promise<boolean>;
};

window.showPrompt = (message: string, defaultValue?: string, title: string = '输入'): Promise<string | null> => {
  return showModal({
    mode: 'prompt',
    title,
    message,
    defaultValue,
    confirmText: '确定',
    cancelText: '取消',
  }) as Promise<string | null>;
};

// ============================================
// End Modal System
// ============================================

type CmdPreset = {
  name: string;
  data: {
    runMode: string;
    module: string;
    cleanTag: string;
    randomSuff: string;
    login: string;
  };
};

const getEl = <T extends HTMLElement>(id: string): T => {
  const el = document.getElementById(id);
  if (!el) {
    throw new Error(`missing element: ${id}`);
  }
  return el as T;
};

const getInput = (id: string) => getEl<HTMLInputElement>(id);
const getSelect = (id: string) => getEl<HTMLSelectElement>(id);

let monacoEditor: monaco.editor.IStandaloneCodeEditor | null = null;
let currentEditMode: 'visual' | 'manual' = 'visual';

const main = async () => {
  const tabsEl = getEl<HTMLDivElement>('tabs');
  const envBadge = getEl<HTMLSpanElement>('envBadge');
  const confBadge = getEl<HTMLSpanElement>('confBadge');
  const subtitle = getEl<HTMLSpanElement>('subtitle');
  const themeBadge = getEl<HTMLSpanElement>('themeBadge');

  const pageHelp = getEl<HTMLElement>('pageHelp');
  const pageConfig = getEl<HTMLElement>('pageConfig');
  const pageCmd = getEl<HTMLElement>('pageCmd');
  const pageRuntime = getEl<HTMLElement>('pageRuntime');

  const logBox = getEl<HTMLPreElement>('logBox');
  const logScroll = getEl<HTMLDivElement>('logScroll');
  const runtimeHint = getEl<HTMLSpanElement>('runtimeHint');
  const statusLine2 = getEl<HTMLSpanElement>('statusLine2');
  const statusLine = getEl<HTMLSpanElement>('statusLine');

  const cfg = defaultUiConfig();
  let comments = { top: {}, item: {}, webui: {}, maxNumberOfOneRecords: {} };
  let confPath = '';

  let unlistenLogs: (() => void) | null = null;

  const setTab = (tab: string) => {
    for (const b of tabsEl.querySelectorAll<HTMLButtonElement>('button[data-tab]')) {
      b.classList.toggle('active', b.dataset.tab === tab);
    }
    pageHelp.style.display = tab === 'help' ? '' : 'none';
    pageConfig.style.display = tab === 'config' ? '' : 'none';
    pageCmd.style.display = tab === 'cmd' ? '' : 'none';
    pageRuntime.style.display = tab === 'runtime' ? '' : 'none';
  };

  const setConfigSubTab = (subtab: string) => {
    const subTabsEl = getEl<HTMLDivElement>('configSubTabs');
    for (const b of subTabsEl.querySelectorAll<HTMLButtonElement>('.sub-tab')) {
      b.classList.toggle('active', b.dataset.subtab === subtab);
    }
    const sections = document.querySelectorAll('.config-section');
    // Map subtab names to section IDs
    const sectionMap: Record<string, string> = {
      'basic': 'sectionBasic',
      'limits': 'sectionDataPagination',
      'rules': 'sectionRules'
    };
    const targetSection = sectionMap[subtab] || ('section' + subtab.charAt(0).toUpperCase() + subtab.slice(1));
    sections.forEach((sec) => {
      sec.classList.toggle('active', sec.id === targetSection);
    });
  };

  const setEditMode = (mode: 'visual' | 'manual') => {
    currentEditMode = mode;
    const visualContainer = getEl<HTMLElement>('visualEditContainer');
    const manualContainer = getEl<HTMLElement>('manualEditContainer');
    const editTabsEl = getEl<HTMLDivElement>('editTabs');
    
    for (const b of editTabsEl.querySelectorAll<HTMLButtonElement>('.edit-tab')) {
      b.classList.toggle('active', b.dataset.edit === mode);
    }

    if (mode === 'visual') {
      visualContainer.style.display = '';
      manualContainer.style.display = 'none';
    } else {
      visualContainer.style.display = 'none';
      manualContainer.style.display = '';
      initMonaco();
    }
  };

  const initMonaco = () => {
    const container = getEl<HTMLElement>('monacoContainer');
    if (monacoEditor) {
      monacoEditor.layout();
      return;
    }

    const isDark = document.documentElement.classList.contains('dark') || 
                  (!document.documentElement.dataset.theme && window.matchMedia('(prefers-color-scheme: dark)').matches);

    syncFromInputs();
    const payload = toBackendPayload(cfg);
    const yamlText = yamlDump(payload);

    monacoEditor = monaco.editor.create(container, {
      value: yamlText,
      language: 'yaml',
      theme: isDark ? 'vs-dark' : 'vs',
      automaticLayout: true,
      minimap: { enabled: false },
      fontSize: 13,
      lineNumbers: 'on',
      scrollBeyondLastLine: false,
      wordWrap: 'on',
      tabSize: 2,
      insertSpaces: true,
    });
  };

  const updateMonacoTheme = () => {
    if (!monacoEditor) return;
    const isDark = document.documentElement.classList.contains('dark') || 
                  (!document.documentElement.dataset.theme && window.matchMedia('(prefers-color-scheme: dark)').matches);
    monaco.editor.setTheme(isDark ? 'vs-dark' : 'vs');
  };

  const syncMonacoToVisual = () => {
    if (!monacoEditor) return;
    const yamlText = monacoEditor.getValue();
    try {
      const doc = yamlParse(yamlText);
      if (!doc) {
        setHint(getEl('monacoHint'), '无效的 YAML');
        return;
      }
      const meta = { ...toBackendPayload(cfg), ...doc };
      const parsed = fromBackendMeta(meta);
      Object.assign(cfg, parsed.cfg);
      comments = parsed.comments;
      bindBaseFields();
      renderAllLists();
      setHint(getEl('monacoHint'), '已同步到可视化表单');
    } catch (e: any) {
      setHint(getEl('monacoHint'), '错误: ' + String(e));
    }
  };

  const syncVisualToMonaco = () => {
    if (!monacoEditor) return;
    syncFromInputs();
    const payload = toBackendPayload(cfg);
    const yamlText = yamlDump(payload);
    monacoEditor.setValue(yamlText);
    setHint(getEl('monacoHint'), '已从可视化表单加载');
  };

  const setHint = (el: HTMLElement, msg: string) => {
    el.textContent = msg || '';
  };

  const appendLogLine = (line: string) => {
    const prev = logBox.textContent || '';
    const next = (prev + '\n' + line).trim();
    const lines = next.split('\n');
    logBox.textContent = lines.slice(-2000).join('\n');
    logScroll.scrollTop = logScroll.scrollHeight;
  };

  const renderCmd = () => {
    const runMode = getSelect('cmdRunMode').value;
    const module = getSelect('cmdModule').value;
    const configPath = getInput('cmdConfigPath').value || './config.yml';
    const cleanTag = getInput('cmdCleanTag').value || '';
    const login = getInput('cmdLogin').value || '';
    const exportPath = getInput('cmdExportPath').value || '';
    
    const pathModeGroup = document.getElementById('cmdPathModeGroup');
    const activePathBtn = pathModeGroup?.querySelector('.toggle-btn.active');
    const pathMode = activePathBtn?.getAttribute('data-value') || 'relative';
    
    const randomSuffGroup = document.getElementById('cmdRandomSuffGroup');
    const activeRandBtn = randomSuffGroup?.querySelector('.toggle-btn.active');
    const rand = activeRandBtn?.getAttribute('data-value') || '1';
    
    const exeRel = './ikuai-bypass';
    const exeAbs = getInput('cmdExePath').value || exeRel;
    const exe = pathMode === 'absolute' ? exeAbs : exeRel;
    const parts = [exe, '-r', runMode, '-c', configPath];
    if (runMode === 'clean') {
      if (cleanTag) parts.push('-tag', cleanTag);
    } else if (runMode !== 'web') {
      parts.push('-m', module);
    }
    if (login) parts.push('-login', login);
    if (exportPath) parts.push('-exportPath', exportPath);
    if (rand) parts.push('-isIpGroupNameAddRandomSuff', rand);
    getEl<HTMLPreElement>('cmdOut').textContent = parts.join(' ');
  };

  const copyText = async (text: string) => {
    try { await navigator.clipboard.writeText(text); } catch (_) {}
  };

  const bindBaseFields = () => {
    getInput('cfgIkuaiUrl').value = cfg.ikuaiUrl;
    getInput('cfgCron').value = cfg.cron;
    getInput('cfgUser').value = cfg.username;
    getInput('cfgPass').value = cfg.password;
    getInput('cfgGhProxy').value = cfg.githubProxy;
    getInput('cfgRetryWait').value = cfg.addErrRetryWait;
    getInput('cfgAddWait').value = cfg.addWait;

    // WebUI toggle switches
    const cfgWebEnable = document.getElementById('cfgWebEnable') as HTMLInputElement | null;
    const cfgWebEnableUpdate = document.getElementById('cfgWebEnableUpdate') as HTMLInputElement | null;
    if (cfgWebEnable) cfgWebEnable.checked = cfg.webui.enable;
    if (cfgWebEnableUpdate) cfgWebEnableUpdate.checked = cfg.webui.enableUpdate;
    getInput('cfgWebPort').value = cfg.webui.port;
    getInput('cfgWebCdn').value = cfg.webui.cdnPrefix;
    getInput('cfgWebUser').value = cfg.webui.user;
    getInput('cfgWebPass').value = cfg.webui.pass;

    getInput('cfgMaxIsp').value = String(cfg.maxNumberOfOneRecords.Isp);
    getInput('cfgMaxIpv4').value = String(cfg.maxNumberOfOneRecords.Ipv4);
    getInput('cfgMaxIpv6').value = String(cfg.maxNumberOfOneRecords.Ipv6);
    getInput('cfgMaxDomain').value = String(cfg.maxNumberOfOneRecords.Domain);

    const cronInput = document.getElementById('cronInput') as HTMLInputElement | null;
    if (cronInput) cronInput.value = cfg.cron;
  };

  const syncFromInputs = () => {
    cfg.ikuaiUrl = getInput('cfgIkuaiUrl').value;
    cfg.cron = getInput('cfgCron').value;
    cfg.username = getInput('cfgUser').value;
    cfg.password = getInput('cfgPass').value;
    cfg.githubProxy = getInput('cfgGhProxy').value;
    cfg.addErrRetryWait = getInput('cfgRetryWait').value;
    cfg.addWait = getInput('cfgAddWait').value;

    // WebUI toggle switches
    const cfgWebEnable = document.getElementById('cfgWebEnable') as HTMLInputElement | null;
    const cfgWebEnableUpdate = document.getElementById('cfgWebEnableUpdate') as HTMLInputElement | null;
    cfg.webui.enable = cfgWebEnable ? cfgWebEnable.checked : cfg.webui.enable;
    cfg.webui.enableUpdate = cfgWebEnableUpdate ? cfgWebEnableUpdate.checked : cfg.webui.enableUpdate;
    cfg.webui.port = getInput('cfgWebPort').value;
    cfg.webui.cdnPrefix = getInput('cfgWebCdn').value;
    cfg.webui.user = getInput('cfgWebUser').value;
    cfg.webui.pass = getInput('cfgWebPass').value;

    cfg.maxNumberOfOneRecords.Isp = Number(getInput('cfgMaxIsp').value || 0) || cfg.maxNumberOfOneRecords.Isp;
    cfg.maxNumberOfOneRecords.Ipv4 = Number(getInput('cfgMaxIpv4').value || 0) || cfg.maxNumberOfOneRecords.Ipv4;
    cfg.maxNumberOfOneRecords.Ipv6 = Number(getInput('cfgMaxIpv6').value || 0) || cfg.maxNumberOfOneRecords.Ipv6;
    cfg.maxNumberOfOneRecords.Domain = Number(getInput('cfgMaxDomain').value || 0) || cfg.maxNumberOfOneRecords.Domain;

    const cronInput = document.getElementById('cronInput') as HTMLInputElement | null;
    if (cronInput) cronInput.value = cfg.cron;
  };

  const renderListSimple = (container: HTMLElement, arr: Array<{ tag: string; url: string }>, onDel: (idx: number) => void, title: string) => {
    container.innerHTML = '';
    if (arr.length === 0) {
      container.innerHTML = '<div class="table-empty">暂无数据</div>';
      return;
    }
    
    const wrapper = document.createElement('div');
    wrapper.className = 'rules-table-wrapper';
    
    const table = document.createElement('table');
    table.className = 'w-full text-sm text-left text-gray-500 dark:text-gray-400';
    table.innerHTML = `
      <thead class="text-xs text-gray-700 uppercase bg-gray-50 dark:bg-gray-700 dark:text-gray-400">
        <tr>
          <th class="px-3 py-3" style="width: 40px;">#</th>
          <th class="px-3 py-3">tag</th>
          <th class="px-3 py-3">url</th>
          <th class="px-3 py-3 delete-cell">操作</th>
        </tr>
      </thead>
      <tbody></tbody>
    `;
    
    const tbody = table.querySelector('tbody')!;
    for (let i = 0; i < arr.length; i++) {
      const it = arr[i];
      const tr = document.createElement('tr');
      tr.className = 'bg-white border-b dark:bg-gray-800 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-600';
      tr.innerHTML = `
        <td class="px-3 py-3">${i + 1}</td>
        <td class="px-3 py-3"><input data-k="tag" value="${it.tag || ''}" placeholder="tag名称" class="bg-transparent border-0 w-full outline-none focus:ring-0 dark:text-white" /></td>
        <td class="px-3 py-3"><input data-k="url" value="${it.url || ''}" placeholder="订阅URL" class="bg-transparent border-0 w-full outline-none focus:ring-0 dark:text-white" /></td>
        <td class="px-3 py-3 delete-cell"><button class="delete-btn" data-del="1">删除</button></td>
      `;
      
      const delBtn = tr.querySelector<HTMLButtonElement>('[data-del]');
      if (delBtn) delBtn.onclick = () => onDel(i);
      
      const tagEl = tr.querySelector<HTMLInputElement>('[data-k="tag"]');
      if (tagEl) tagEl.oninput = (e) => { it.tag = (e.target as HTMLInputElement).value; };
      
      const urlEl = tr.querySelector<HTMLInputElement>('[data-k="url"]');
      if (urlEl) urlEl.oninput = (e) => { it.url = (e.target as HTMLInputElement).value; };
      
      tbody.appendChild(tr);
    }
    
    wrapper.appendChild(table);
    container.appendChild(wrapper);
  };

  const renderStreamDomain = () => {
    const container = getEl<HTMLElement>('listStreamDomain');
    container.innerHTML = '';
    if (cfg.streamDomain.length === 0) {
      container.innerHTML = '<div class="table-empty">暂无数据</div>';
      return;
    }
    
    const wrapper = document.createElement('div');
    wrapper.className = 'rules-table-wrapper';
    
    const table = document.createElement('table');
    table.className = 'w-full text-sm text-left text-gray-500 dark:text-gray-400';
    table.innerHTML = `
      <thead class="text-xs text-gray-700 uppercase bg-gray-50 dark:bg-gray-700 dark:text-gray-400">
        <tr>
          <th class="px-3 py-3" style="width: 40px;">#</th>
          <th class="px-3 py-3">interface</th>
          <th class="px-3 py-3">tag</th>
          <th class="px-3 py-3">src-addr</th>
          <th class="px-3 py-3">src-addr-opt-ipgroup</th>
          <th class="px-3 py-3">url</th>
          <th class="px-3 py-3 delete-cell">操作</th>
        </tr>
      </thead>
      <tbody></tbody>
    `;
    
    const tbody = table.querySelector('tbody')!;
    for (let i = 0; i < cfg.streamDomain.length; i++) {
      const it = cfg.streamDomain[i];
      const tr = document.createElement('tr');
      tr.className = 'bg-white border-b dark:bg-gray-800 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-600';
      tr.innerHTML = `
        <td class="px-3 py-3">${i + 1}</td>
        <td class="px-3 py-3"><input data-k="interface" value="${it.interface || ''}" placeholder="接口" class="bg-transparent border-0 w-full outline-none focus:ring-0 dark:text-white" /></td>
        <td class="px-3 py-3"><input data-k="tag" value="${it.tag || ''}" placeholder="标签" class="bg-transparent border-0 w-full outline-none focus:ring-0 dark:text-white" /></td>
        <td class="px-3 py-3"><input data-k="srcAddr" value="${it.srcAddr || ''}" placeholder="源地址" class="bg-transparent border-0 w-full outline-none focus:ring-0 dark:text-white" /></td>
        <td class="px-3 py-3"><input data-k="srcAddrOptIpGroup" value="${it.srcAddrOptIpGroup || ''}" placeholder="IP组" class="bg-transparent border-0 w-full outline-none focus:ring-0 dark:text-white" /></td>
        <td class="px-3 py-3"><input data-k="url" value="${it.url || ''}" placeholder="订阅URL" class="bg-transparent border-0 w-full outline-none focus:ring-0 dark:text-white" /></td>
        <td class="px-3 py-3 delete-cell"><button class="delete-btn" data-del="1">删除</button></td>
      `;
      
      const delBtn = tr.querySelector<HTMLButtonElement>('[data-del]');
      if (delBtn) delBtn.onclick = () => { cfg.streamDomain.splice(i, 1); renderStreamDomain(); };
      
      const iEl = tr.querySelector<HTMLInputElement>('[data-k="interface"]');
      if (iEl) iEl.oninput = (e) => { it.interface = (e.target as HTMLInputElement).value; };
      
      const tEl = tr.querySelector<HTMLInputElement>('[data-k="tag"]');
      if (tEl) tEl.oninput = (e) => { it.tag = (e.target as HTMLInputElement).value; };
      
      const sEl = tr.querySelector<HTMLInputElement>('[data-k="srcAddr"]');
      if (sEl) sEl.oninput = (e) => { it.srcAddr = (e.target as HTMLInputElement).value; };
      
      const soEl = tr.querySelector<HTMLInputElement>('[data-k="srcAddrOptIpGroup"]');
      if (soEl) soEl.oninput = (e) => { it.srcAddrOptIpGroup = (e.target as HTMLInputElement).value; };
      
      const uEl = tr.querySelector<HTMLInputElement>('[data-k="url"]');
      if (uEl) uEl.oninput = (e) => { it.url = (e.target as HTMLInputElement).value; };
      
      tbody.appendChild(tr);
    }
    
    wrapper.appendChild(table);
    container.appendChild(wrapper);
  };

  const renderStreamIpPort = () => {
    const container = getEl<HTMLElement>('listStreamIpPort');
    container.innerHTML = '';
    if (cfg.streamIpPort.length === 0) {
      container.innerHTML = '<div class="table-empty">暂无数据</div>';
      return;
    }
    
    const wrapper = document.createElement('div');
    wrapper.className = 'rules-table-wrapper';
    
    const table = document.createElement('table');
    table.className = 'w-full text-sm text-left text-gray-500 dark:text-gray-400';
    table.innerHTML = `
      <thead class="text-xs text-gray-700 uppercase bg-gray-50 dark:bg-gray-700 dark:text-gray-400">
        <tr>
          <th class="px-3 py-3" style="width: 40px;">#</th>
          <th class="px-3 py-3">type</th>
          <th class="px-3 py-3">opt-tagname</th>
          <th class="px-3 py-3">interface</th>
          <th class="px-3 py-3">nexthop</th>
          <th class="px-3 py-3">src-addr</th>
          <th class="px-3 py-3">src-addr-opt-ipgroup</th>
          <th class="px-3 py-3">ip-group</th>
          <th class="px-3 py-3">mode</th>
          <th class="px-3 py-3">ifaceband</th>
          <th class="px-3 py-3 delete-cell">操作</th>
        </tr>
      </thead>
      <tbody></tbody>
    `;
    
    const tbody = table.querySelector('tbody')!;
    for (let i = 0; i < cfg.streamIpPort.length; i++) {
      const it = cfg.streamIpPort[i];
      const tr = document.createElement('tr');
      tr.className = 'bg-white border-b dark:bg-gray-800 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-600';
      tr.innerHTML = `
        <td class="px-3 py-3">${i + 1}</td>
        <td class="px-3 py-3"><input data-k="type" value="${it.type || ''}" placeholder="类型" style="width: 60px;" class="bg-transparent border-0 w-full outline-none focus:ring-0 dark:text-white" /></td>
        <td class="px-3 py-3"><input data-k="optTagName" value="${it.optTagName || ''}" placeholder="标签名" class="bg-transparent border-0 w-full outline-none focus:ring-0 dark:text-white" /></td>
        <td class="px-3 py-3"><input data-k="interface" value="${it.interface || ''}" placeholder="接口" class="bg-transparent border-0 w-full outline-none focus:ring-0 dark:text-white" /></td>
        <td class="px-3 py-3"><input data-k="nexthop" value="${it.nexthop || ''}" placeholder="下一跳" class="bg-transparent border-0 w-full outline-none focus:ring-0 dark:text-white" /></td>
        <td class="px-3 py-3"><input data-k="srcAddr" value="${it.srcAddr || ''}" placeholder="源地址" class="bg-transparent border-0 w-full outline-none focus:ring-0 dark:text-white" /></td>
        <td class="px-3 py-3"><input data-k="srcAddrOptIpGroup" value="${it.srcAddrOptIpGroup || ''}" placeholder="IP组" class="bg-transparent border-0 w-full outline-none focus:ring-0 dark:text-white" /></td>
        <td class="px-3 py-3"><input data-k="ipGroup" value="${it.ipGroup || ''}" placeholder="IP分组" class="bg-transparent border-0 w-full outline-none focus:ring-0 dark:text-white" /></td>
        <td class="px-3 py-3"><input data-k="mode" value="${it.mode || '0'}" placeholder="模式" style="width: 50px;" class="bg-transparent border-0 w-full outline-none focus:ring-0 dark:text-white" /></td>
        <td class="px-3 py-3"><input data-k="ifaceband" value="${it.ifaceband || '0'}" placeholder="带宽" style="width: 50px;" class="bg-transparent border-0 w-full outline-none focus:ring-0 dark:text-white" /></td>
        <td class="px-3 py-3 delete-cell"><button class="delete-btn" data-del="1">删除</button></td>
      `;
      
      const delBtn = tr.querySelector<HTMLButtonElement>('[data-del]');
      if (delBtn) delBtn.onclick = () => { cfg.streamIpPort.splice(i, 1); renderStreamIpPort(); };
      
      const tyEl = tr.querySelector<HTMLInputElement>('[data-k="type"]');
      if (tyEl) tyEl.oninput = (e) => { it.type = (e.target as HTMLInputElement).value; };
      
      const otEl = tr.querySelector<HTMLInputElement>('[data-k="optTagName"]');
      if (otEl) otEl.oninput = (e) => { it.optTagName = (e.target as HTMLInputElement).value; };
      
      const inEl = tr.querySelector<HTMLInputElement>('[data-k="interface"]');
      if (inEl) inEl.oninput = (e) => { it.interface = (e.target as HTMLInputElement).value; };
      
      const nhEl = tr.querySelector<HTMLInputElement>('[data-k="nexthop"]');
      if (nhEl) nhEl.oninput = (e) => { it.nexthop = (e.target as HTMLInputElement).value; };
      
      const saEl = tr.querySelector<HTMLInputElement>('[data-k="srcAddr"]');
      if (saEl) saEl.oninput = (e) => { it.srcAddr = (e.target as HTMLInputElement).value; };
      
      const soEl = tr.querySelector<HTMLInputElement>('[data-k="srcAddrOptIpGroup"]');
      if (soEl) soEl.oninput = (e) => { it.srcAddrOptIpGroup = (e.target as HTMLInputElement).value; };
      
      const ipEl = tr.querySelector<HTMLInputElement>('[data-k="ipGroup"]');
      if (ipEl) ipEl.oninput = (e) => { it.ipGroup = (e.target as HTMLInputElement).value; };
      
      const moEl = tr.querySelector<HTMLInputElement>('[data-k="mode"]');
      if (moEl) moEl.oninput = (e) => { it.mode = (e.target as HTMLInputElement).value; };
      
      const ibEl = tr.querySelector<HTMLInputElement>('[data-k="ifaceband"]');
      if (ibEl) ibEl.oninput = (e) => { it.ifaceband = (e.target as HTMLInputElement).value; };
      
      tbody.appendChild(tr);
    }
    
    wrapper.appendChild(table);
    container.appendChild(wrapper);
  };

  const renderAllLists = () => {
    renderListSimple(getEl('listCustomIsp'), cfg.customIsp, (i) => { cfg.customIsp.splice(i, 1); renderAllLists(); }, 'custom-isp');
    renderListSimple(getEl('listIpGroup'), cfg.ipGroup, (i) => { cfg.ipGroup.splice(i, 1); renderAllLists(); }, 'ip-group');
    renderListSimple(getEl('listIpv6Group'), cfg.ipv6Group, (i) => { cfg.ipv6Group.splice(i, 1); renderAllLists(); }, 'ipv6-group');
    renderStreamDomain();
    renderStreamIpPort();
  };

  const loadBackend = async () => {
    const meta = await bridge.getConfigMeta();
    const parsed = fromBackendMeta(meta);
    Object.assign(cfg, parsed.cfg);
    comments = parsed.comments;
    confPath = parsed.confPath;
    confBadge.textContent = 'conf=' + confPath;
    getInput('cmdExePath').value = String(meta.exe_path || '');
    bindBaseFields();
    renderAllLists();
    getInput('cmdConfigPath').value = confPath || './config.yml';
    getInput('cmdLogin').value = cfg.ikuaiUrl && cfg.username ? (cfg.ikuaiUrl + ',' + cfg.username + ',' + cfg.password) : '';
    getInput('cmdExportPath').value = loadJson('ikb_export_path', '/tmp');
    renderCmd();
  };

  const saveConfig = async (withComments: boolean) => {
    syncFromInputs();
    const payload = toBackendPayload(cfg);
    await bridge.saveConfig(payload, withComments);
    setHint(getEl('saveHint'), '保存成功');
    await loadBackend();
  };

  const refreshStatus = async () => {
    const st = await bridge.runtimeStatus();
    const txt = 'Running=' + st.running + ' Cron=' + st.cron_running + ' Next=' + (st.next_run_at || '');
    statusLine2.textContent = txt;
    statusLine.textContent = txt;
  };

  const runOnce = async (module: string) => {
    const started = await bridge.runtimeRunOnce(module);
    setHint(runtimeHint, started ? '已启动' : '任务已在运行中');
  };

  const cronStart = async (expr: string, module: string) => {
    await bridge.runtimeCronStart(expr, module);
    setHint(runtimeHint, '定时任务已启动');
  };

  const cronStop = async () => {
    await bridge.runtimeCronStop();
    setHint(runtimeHint, '定时任务已停止');
  };

  const tailLogs = async () => {
    const arr = await bridge.runtimeTailLogs(400);
    logBox.textContent = '';
    for (const r of arr) {
      appendLogLine((r.ts || '') + ' [' + (r.module || '') + '] [' + (r.tag || '') + '] ' + (r.detail || ''));
    }
  };

  let streamReconnectTimer: ReturnType<typeof setTimeout> | null = null;
  const RECONNECT_DELAY = 3000;

  const scheduleReconnect = () => {
    if (streamReconnectTimer) clearTimeout(streamReconnectTimer);
    streamReconnectTimer = setTimeout(() => { startStream().catch(() => {}); }, RECONNECT_DELAY);
  };

  const startStream = async () => {
    if (streamReconnectTimer) { clearTimeout(streamReconnectTimer); streamReconnectTimer = null; }
    stopStream();
    try {
      unlistenLogs = await bridge.listenLogs(
        (r) => { appendLogLine((r.ts || '') + ' [' + (r.module || '') + '] [' + (r.tag || '') + '] ' + (r.detail || '')); },
        () => scheduleReconnect(),
      );
    } catch (_) { scheduleReconnect(); }
  };

  const stopStream = () => {
    if (streamReconnectTimer) { clearTimeout(streamReconnectTimer); streamReconnectTimer = null; }
    if (unlistenLogs) { try { unlistenLogs(); } catch (_) {} }
    unlistenLogs = null;
  };

  const initEnvUi = () => {
    const t = bridge.isTauri();
    envBadge.textContent = 'env=' + (t ? 'tauri' : 'web');
    subtitle.textContent = t ? 'App (Tauri v2)' : 'WebUI';
    document.body.classList.toggle('tauri', t);
  };

  const applyTheme = (mode: string) => {
    const m = (mode || 'auto') as 'auto' | 'dark' | 'light';
    const root = document.documentElement;
    root.dataset.theme = m;
    saveJson('ikb_theme', m);
    themeBadge.textContent = 'theme=' + m;
    const prefersDark = window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches;
    const useDark = m === 'dark' || (m === 'auto' && prefersDark);
    root.classList.toggle('light', m === 'light');
    root.classList.toggle('dark', useDark);
    updateMonacoTheme();
  };

  const loadRemoteConfig = async () => {
    const rawUrl = (getInput('remoteUrl').value || '').trim();
    if (!rawUrl) { setHint(getEl('remoteHint'), 'Remote URL is empty'); return; }
    
    const modal = document.getElementById('remoteConfigModal');
    const modalUrl = document.getElementById('modalRemoteUrl');
    const modalCancel = document.getElementById('modalCancel');
    const modalConfirm = document.getElementById('modalConfirm');
    
    if (!modal || !modalUrl || !modalCancel || !modalConfirm) return;
    
    modalUrl.textContent = rawUrl;
    modal.style.display = 'flex';
    
    const cleanup = () => {
      modal.style.display = 'none';
      modalCancel?.removeEventListener('click', onCancel);
      modalConfirm?.removeEventListener('click', onConfirm);
    };
    
    const onCancel = () => cleanup();
    
    const onConfirm = async () => {
      cleanup();
      await doLoadRemoteConfig(rawUrl, 'remoteHint');
    };
    
    modalCancel.addEventListener('click', onCancel);
    modalConfirm.addEventListener('click', onConfirm);
  };
  
  const loadRemoteConfigManual = async () => {
    const rawUrl = (getInput('remoteUrlManual').value || '').trim();
    if (!rawUrl) { setHint(getEl('remoteHintManual'), 'Remote URL is empty'); return; }
    
    const modal = document.getElementById('remoteConfigModal');
    const modalUrl = document.getElementById('modalRemoteUrl');
    const modalCancel = document.getElementById('modalCancel');
    const modalConfirm = document.getElementById('modalConfirm');
    
    if (!modal || !modalUrl || !modalCancel || !modalConfirm) return;
    
    modalUrl.textContent = rawUrl;
    modal.style.display = 'flex';
    
    const cleanup = () => {
      modal.style.display = 'none';
      modalCancel?.removeEventListener('click', onCancel);
      modalConfirm?.removeEventListener('click', onConfirm);
    };
    
    const onCancel = () => cleanup();
    
    const onConfirm = async () => {
      cleanup();
      await doLoadRemoteConfig(rawUrl, 'remoteHintManual');
    };
    
    modalCancel.addEventListener('click', onCancel);
    modalConfirm.addEventListener('click', onConfirm);
  };
  
  const doLoadRemoteConfig = async (rawUrl: string, hintId: string) => {
    setHint(getEl(hintId), '正在加载...');
    try {
      let finalUrl = rawUrl;
      const proxy = (cfg.githubProxy || '').trim();
      if (proxy && rawUrl.startsWith('https://raw.githubusercontent.com/')) {
        finalUrl = proxy.endsWith('/') ? (proxy + rawUrl) : (proxy + '/' + rawUrl);
      }
      let text = '';
      if (bridge.isTauri()) {
        text = await bridge.fetchRemoteConfig(finalUrl, proxy);
      } else {
        const r = await fetch(finalUrl);
        if (!r.ok) throw new Error('HTTP ' + r.status + ' ' + r.statusText);
        text = await r.text();
      }
      const doc = yamlParse(text) || {};
      const meta = { ...toBackendPayload(cfg), ...doc };
      const parsed = fromBackendMeta(meta);
      Object.assign(cfg, parsed.cfg);
      bindBaseFields();
      renderAllLists();
      saveJson('ikb_remote_url', rawUrl);
      
      if (monacoEditor) {
        syncFromInputs();
        const payload = toBackendPayload(cfg);
        const yamlText = yamlDump(payload);
        monacoEditor.setValue(yamlText);
      }
      
      const manualUrlInput = document.getElementById('remoteUrlManual') as HTMLInputElement | null;
      if (manualUrlInput) manualUrlInput.value = rawUrl;
      
      setHint(getEl(hintId), '加载成功');
    } catch (e: any) {
      let msg = String(e && e.message ? e.message : e);
      setHint(getEl(hintId), '加载失败: ' + msg);
    }
  };

  const restoreCmdSettings = () => {
    const saved = loadJson('ikb_cmd', { runMode: 'cron', module: 'ispdomain', cleanTag: 'cleanAll', randomSuff: '1', pathMode: 'relative' });
    getSelect('cmdRunMode').value = saved.runMode;
    getSelect('cmdModule').value = saved.module;
    getInput('cmdCleanTag').value = saved.cleanTag;
    
    const pathModeGroup = document.getElementById('cmdPathModeGroup');
    if (pathModeGroup) {
      const buttons = pathModeGroup.querySelectorAll('.toggle-btn');
      buttons.forEach((btn) => {
        btn.classList.toggle('active', btn.getAttribute('data-value') === saved.pathMode);
      });
    }
    
    const randomSuffGroup = document.getElementById('cmdRandomSuffGroup');
    if (randomSuffGroup) {
      const buttons = randomSuffGroup.querySelectorAll('.toggle-btn');
      buttons.forEach((btn) => {
        btn.classList.toggle('active', btn.getAttribute('data-value') === saved.randomSuff);
      });
    }
  };

  const loadPresets = (): CmdPreset[] => loadJson('ikb_cmd_presets', [] as CmdPreset[]);
  const savePresets = (p: CmdPreset[]) => saveJson('ikb_cmd_presets', p);

  const renderPresets = () => {
    const el = getEl<HTMLElement>('presetList');
    const presets = loadPresets();
    el.innerHTML = '';
    if (!presets.length) { el.innerHTML = '<div class="text-sm text-gray-500 dark:text-gray-400">暂无预设（最多 5 个）</div>'; return; }
    for (let i = 0; i < presets.length; i++) {
      const p = presets[i];
      const div = document.createElement('div');
      div.className = 'bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-xl p-5 transition-all hover:border-blue-500 dark:hover:border-blue-500 hover:shadow-md';
      div.innerHTML = `
        <div class="flex flex-wrap justify-between items-center gap-3">
          <b class="text-sm font-semibold text-gray-900 dark:text-gray-100">${p.name || ('预设 ' + (i + 1))}</b>
          <div class="flex gap-2">
            <button data-load="1" class="px-3 py-1.5 text-xs font-medium rounded-lg bg-gray-100 dark:bg-gray-700 hover:bg-blue-50 dark:hover:bg-blue-900/30 text-gray-700 dark:text-gray-300 transition-all border border-gray-200 dark:border-gray-600">加载</button>
            <button data-rename="1" class="px-3 py-1.5 text-xs font-medium rounded-lg bg-gray-100 dark:bg-gray-700 hover:bg-blue-50 dark:hover:bg-blue-900/30 text-gray-700 dark:text-gray-300 transition-all border border-gray-200 dark:border-gray-600">改名</button>
            <button data-del="1" class="px-3 py-1.5 text-xs font-medium rounded-lg bg-red-50 dark:bg-red-900/20 hover:bg-red-100 dark:hover:bg-red-900/40 text-red-600 dark:text-red-400 transition-all border border-red-200 dark:border-red-800">删除</button>
          </div>
        </div>
      `;
      const loadBtn = div.querySelector<HTMLButtonElement>('[data-load]');
      if (loadBtn) loadBtn.onclick = () => {
        getSelect('cmdRunMode').value = p.data.runMode;
        getSelect('cmdModule').value = p.data.module;
        getInput('cmdCleanTag').value = p.data.cleanTag;
        
        const randomSuffGroup = document.getElementById('cmdRandomSuffGroup');
        if (randomSuffGroup) {
          const buttons = randomSuffGroup.querySelectorAll('.toggle-btn');
          buttons.forEach((btn) => {
            btn.classList.toggle('active', btn.getAttribute('data-value') === p.data.randomSuff);
          });
        }
        
        getInput('cmdLogin').value = p.data.login;
        renderCmd();
        persistCmdSettings();
      };
      const renameBtn = div.querySelector<HTMLButtonElement>('[data-rename]');
      if (renameBtn) renameBtn.onclick = async () => {
        const name = await window.showPrompt('请输入新名称:', p.name || ('预设 ' + (i + 1)));
        if (!name) return;
        presets[i].name = name;
        savePresets(presets);
        renderPresets();
      };
      const delBtn = div.querySelector<HTMLButtonElement>('[data-del]');
      if (delBtn) delBtn.onclick = async () => {
        if (!await window.showConfirm('确定要删除该预设吗？')) return;
        presets.splice(i, 1);
        savePresets(presets);
        renderPresets();
      };
      el.appendChild(div);
    }
  };

  const persistCmdSettings = () => {
    const pathModeGroup = document.getElementById('cmdPathModeGroup');
    const activePathBtn = pathModeGroup?.querySelector('.toggle-btn.active');
    const pathMode = activePathBtn?.getAttribute('data-value') || 'relative';
    
    const randomSuffGroup = document.getElementById('cmdRandomSuffGroup');
    const activeRandBtn = randomSuffGroup?.querySelector('.toggle-btn.active');
    const randomSuff = activeRandBtn?.getAttribute('data-value') || '1';
    
    saveJson('ikb_cmd', {
      runMode: getSelect('cmdRunMode').value,
      module: getSelect('cmdModule').value,
      cleanTag: getInput('cmdCleanTag').value,
      randomSuff: randomSuff,
      pathMode: pathMode,
    });
    saveJson('ikb_export_path', getInput('cmdExportPath').value || '/tmp');
  };

  const wire = () => {
    tabsEl.addEventListener('click', (e) => {
      const b = (e.target as HTMLElement).closest('button[data-tab]') as HTMLElement | null;
      if (!b) return;
      setTab(b.dataset.tab || 'config');
    });

    const configSubTabsEl = getEl<HTMLDivElement>('configSubTabs');
    configSubTabsEl.addEventListener('click', (e) => {
      const b = (e.target as HTMLElement).closest('.sub-tab') as HTMLElement | null;
      if (!b) return;
      setConfigSubTab(b.dataset.subtab || 'basic');
    });

    const editTabsEl = getEl<HTMLDivElement>('editTabs');
    editTabsEl.addEventListener('click', (e) => {
      const b = (e.target as HTMLElement).closest('.edit-tab') as HTMLElement | null;
      if (!b) return;
      setEditMode(b.dataset.edit as 'visual' | 'manual');
    });

    getEl<HTMLButtonElement>('btnSyncToVisual').onclick = () => syncMonacoToVisual();
    getEl<HTMLButtonElement>('btnSyncFromVisual').onclick = () => syncVisualToMonaco();

    for (const id of ['cmdRunMode', 'cmdModule', 'cmdConfigPath', 'cmdCleanTag', 'cmdLogin', 'cmdExportPath']) {
      getEl<HTMLInputElement | HTMLSelectElement>(id).addEventListener('input', () => { renderCmd(); persistCmdSettings(); });
      getEl<HTMLInputElement | HTMLSelectElement>(id).addEventListener('change', () => { renderCmd(); persistCmdSettings(); });
    }
    
    const setupToggleGroup = (groupId: string, onChange: () => void) => {
      const group = document.getElementById(groupId);
      if (!group) return;
      group.addEventListener('click', (e) => {
        const btn = (e.target as HTMLElement).closest('.toggle-btn') as HTMLElement | null;
        if (!btn) return;
        group.querySelectorAll('.toggle-btn').forEach((b) => b.classList.remove('active'));
        btn.classList.add('active');
        onChange();
      });
    };
    
    setupToggleGroup('cmdPathModeGroup', () => { renderCmd(); persistCmdSettings(); });
    setupToggleGroup('cmdRandomSuffGroup', () => { renderCmd(); persistCmdSettings(); });

    getEl<HTMLButtonElement>('btnCopyCmd').onclick = async () => {
      await copyText(getEl<HTMLPreElement>('cmdOut').textContent || '');
      setHint(getEl('cmdHint'), '已复制');
      setTimeout(() => setHint(getEl('cmdHint'), ''), 1500);
    };

    getEl<HTMLButtonElement>('btnSavePreset').onclick = async () => {
      const presets = loadPresets();
      if (presets.length >= 5) { setHint(getEl('presetHint'), '最多只能保存 5 个预设'); return; }
      const name = await window.showPrompt('请输入预设名称:', '预设 ' + (presets.length + 1));
      if (!name) return;
      
      const randomSuffGroup = document.getElementById('cmdRandomSuffGroup');
      const activeRandBtn = randomSuffGroup?.querySelector('.toggle-btn.active');
      const randomSuff = activeRandBtn?.getAttribute('data-value') || '1';
      
      presets.push({ name, data: { runMode: getSelect('cmdRunMode').value, module: getSelect('cmdModule').value, cleanTag: getInput('cmdCleanTag').value, randomSuff: randomSuff, login: getInput('cmdLogin').value } });
      savePresets(presets);
      renderPresets();
      setHint(getEl('presetHint'), '已保存');
      setTimeout(() => setHint(getEl('presetHint'), ''), 1500);
    };

    getEl<HTMLButtonElement>('btnRestoreDefaults').onclick = async () => {
      if (!await window.showConfirm('重置运行命令参数为默认值？')) return;
      getSelect('cmdRunMode').value = 'cron';
      getSelect('cmdModule').value = 'ispdomain';
      getInput('cmdCleanTag').value = 'cleanAll';
      
      const randomSuffGroup = document.getElementById('cmdRandomSuffGroup');
      if (randomSuffGroup) {
        const buttons = randomSuffGroup.querySelectorAll('.toggle-btn');
        buttons.forEach((btn) => {
          btn.classList.toggle('active', btn.getAttribute('data-value') === '1');
        });
      }
      
      getInput('cmdLogin').value = (cfg.ikuaiUrl && cfg.username) ? (cfg.ikuaiUrl + ',' + cfg.username + ',' + cfg.password) : '';
      renderCmd();
      persistCmdSettings();
    };

    getEl<HTMLButtonElement>('themeLight').onclick = () => applyTheme('light');
    getEl<HTMLButtonElement>('themeDark').onclick = () => applyTheme('dark');
    getEl<HTMLButtonElement>('themeAuto').onclick = () => applyTheme('auto');

    getEl<HTMLButtonElement>('btnSaveNoComments').onclick = () => saveConfig(false).catch((e) => setHint(getEl('saveHint'), String(e)));
    getEl<HTMLButtonElement>('btnSaveWithComments').onclick = () => saveConfig(true).catch((e) => setHint(getEl('saveHint'), String(e)));

    getEl<HTMLButtonElement>('btnLoadRemote').onclick = () => loadRemoteConfig();
    getEl<HTMLButtonElement>('btnResetRemote').onclick = () => {
      const def = 'https://raw.githubusercontent.com/joyanhui/ikuai-bypass/refs/heads/main/config.yml';
      getInput('remoteUrl').value = def;
      saveJson('ikb_remote_url', def);
    };
    
    getEl<HTMLButtonElement>('btnLoadRemoteManual').onclick = () => loadRemoteConfigManual();
    getEl<HTMLButtonElement>('btnResetRemoteManual').onclick = () => {
      const def = 'https://raw.githubusercontent.com/joyanhui/ikuai-bypass/refs/heads/main/config.yml';
      getInput('remoteUrlManual').value = def;
      saveJson('ikb_remote_url', def);
    };

    getEl<HTMLButtonElement>('addCustomIsp').onclick = () => { cfg.customIsp.push({ tag: '', url: '' }); renderAllLists(); };
    getEl<HTMLButtonElement>('addIpGroup').onclick = () => { cfg.ipGroup.push({ tag: '', url: '' }); renderAllLists(); };
    getEl<HTMLButtonElement>('addIpv6Group').onclick = () => { cfg.ipv6Group.push({ tag: '', url: '' }); renderAllLists(); };
    getEl<HTMLButtonElement>('addStreamDomain').onclick = () => { cfg.streamDomain.push({ interface: '', srcAddr: '', srcAddrOptIpGroup: '', url: '', tag: '' }); renderAllLists(); };
    getEl<HTMLButtonElement>('addStreamIpPort').onclick = () => { cfg.streamIpPort.push({ optTagName: '', type: '0', interface: '', nexthop: '', srcAddr: '', srcAddrOptIpGroup: '', ipGroup: '', mode: '0', ifaceband: '0' }); renderAllLists(); };

    const moduleChips = getEl<HTMLDivElement>('moduleChips');
    const runModeChips = getEl<HTMLDivElement>('runModeChips');
    const cronBox = getEl<HTMLDivElement>('cronBox');
    const cronInput = getInput('cronInput');

    const modules = [
      { label: '运营商/域名分流', value: 'ispdomain' },
      { label: 'IPv4分组/端口分流', value: 'ipgroup' },
      { label: 'IPv6分组', value: 'ipv6group' },
      { label: '混合模式', value: 'ii' },
      { label: 'IP混合', value: 'ip' },
      { label: '全能模式', value: 'iip' },
    ];

    const runModes = [
      { label: '只执行一次', value: 'once', needCron: false },
      { label: '计划任务', value: 'cron', needCron: true },
      { label: '延迟计划', value: 'cronAft', needCron: true },
      { label: '清理模式', value: 'clean', needCron: false },
    ];

    let selectedModule = modules[0].value;
    let selectedRunMode = runModes[0].value;

    const updateCronBox = () => {
      const rm = runModes.find((m) => m.value === selectedRunMode);
      if (rm && rm.needCron) { cronInput.value = cfg.cron || ''; cronBox.style.display = ''; }
      else { cronBox.style.display = 'none'; }
    };

    const renderChips = (host: HTMLElement, items: Array<{ label: string; value: string }>, current: string, onChange: (v: string) => void) => {
      host.innerHTML = '';
      for (const it of items) {
        const btn = document.createElement('button');
        btn.className = 'chip' + (it.value === current ? ' active' : '');
        btn.textContent = it.label;
        btn.onclick = () => { onChange(it.value); renderChips(host, items, it.value, onChange); };
        host.appendChild(btn);
      }
    };

    renderChips(moduleChips, modules, selectedModule, (v) => { selectedModule = v; });
    renderChips(runModeChips, runModes, selectedRunMode, (v) => { selectedRunMode = v; updateCronBox(); });
    updateCronBox();

    getEl<HTMLButtonElement>('btnStart').onclick = async () => {
      try {
        if (selectedRunMode === 'once') { await runOnce(selectedModule); return; }
        if (selectedRunMode === 'clean') { if (!await window.showConfirm('确定要清理所有 IKB 规则吗？')) return; await runOnce('clean'); return; }
        const expr = cronInput.value.trim();
        if (!expr) { setHint(runtimeHint, '当前未配置 Cron 表达式'); return; }
        if (selectedRunMode === 'cron') { await runOnce(selectedModule); await cronStart(expr, selectedModule); return; }
        if (selectedRunMode === 'cronAft') { await cronStart(expr, selectedModule); }
      } catch (e) { setHint(runtimeHint, String(e)); }
    };

    getEl<HTMLButtonElement>('btnStop').onclick = async () => {
      try { await cronStop(); } catch (e) { setHint(runtimeHint, String(e)); }
    };
  };

  initEnvUi();
  applyTheme(loadJson('ikb_theme', 'auto'));
  const savedRemoteUrl = loadJson('ikb_remote_url', 'https://raw.githubusercontent.com/joyanhui/ikuai-bypass/refs/heads/main/config.yml');
  getInput('remoteUrl').value = savedRemoteUrl;
  const manualUrlInput = document.getElementById('remoteUrlManual') as HTMLInputElement | null;
  if (manualUrlInput) manualUrlInput.value = savedRemoteUrl;
  restoreCmdSettings();
  renderPresets();
  wire();
  setTab(bridge.isTauri() ? 'runtime' : 'config');
  setConfigSubTab('basic');
  setEditMode('visual');
  
  await loadBackend().catch((e) => setHint(getEl('configHint'), String(e)));
  await refreshStatus().catch(() => {});
  await tailLogs().catch(() => {});
  
  setInterval(() => refreshStatus().catch(() => {}), 1500);
  startStream().catch(() => {});
};

main().catch(() => {});
