import { bridge } from '../lib/bridge.ts';
import { defaultUiConfig, fromBackendMeta, toBackendPayload, yamlDump, yamlDumpWithComments, yamlParse } from '../lib/config_model.ts';
import { loadJson, saveJson } from '../lib/storage.ts';

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
const getTextArea = (id: string) => getEl<HTMLTextAreaElement>(id);

const main = async () => {
  const tabsEl = getEl<HTMLDivElement>('tabs');
  const envBadge = getEl<HTMLSpanElement>('envBadge');
  const confBadge = getEl<HTMLSpanElement>('confBadge');
  const subtitle = getEl<HTMLSpanElement>('subtitle');
  const themeBadge = getEl<HTMLSpanElement>('themeBadge');

  const pageHelp = getEl<HTMLElement>('pageHelp');
  const pageConfig = getEl<HTMLElement>('pageConfig');
  const pageRuntime = getEl<HTMLElement>('pageRuntime');
  const tauriSidebar = getEl<HTMLElement>('tauriSidebar');

  const logBox = getEl<HTMLPreElement>('logBox');
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
    pageRuntime.style.display = tab === 'runtime' ? '' : 'none';
  };

  const setHint = (el: HTMLElement, msg: string) => {
    el.textContent = msg || '';
  };

  const appendLogLine = (line: string) => {
    const prev = logBox.textContent || '';
    const next = (prev + '\n' + line).trim();
    const lines = next.split('\n');
    logBox.textContent = lines.slice(-2000).join('\n');
    logBox.scrollTop = logBox.scrollHeight;
  };

  const renderCmd = () => {
    const runMode = getSelect('cmdRunMode').value;
    const module = getSelect('cmdModule').value;
    const configPath = getInput('cmdConfigPath').value || './config.yml';
    const cleanTag = getInput('cmdCleanTag').value || '';
    const login = getInput('cmdLogin').value || '';
    const exportPath = getInput('cmdExportPath').value || '';
    const rand = getSelect('cmdRandomSuff').value;
    const exeRel = './ikuai-bypass';
    const pathMode = getSelect('cmdPathMode').value;
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
    try {
      await navigator.clipboard.writeText(text);
    } catch (_) {}
  };

  const bindBaseFields = () => {
    getInput('cfgIkuaiUrl').value = cfg.ikuaiUrl;
    getInput('cfgCron').value = cfg.cron;
    getInput('cfgUser').value = cfg.username;
    getInput('cfgPass').value = cfg.password;
    getInput('cfgGhProxy').value = cfg.githubProxy;
    getInput('cfgRetryWait').value = cfg.addErrRetryWait;
    getInput('cfgAddWait').value = cfg.addWait;

    getSelect('cfgWebEnable').value = cfg.webui.enable ? '1' : '0';
    getSelect('cfgWebEnableUpdate').value = cfg.webui.enableUpdate ? '1' : '0';
    getInput('cfgWebPort').value = cfg.webui.port;
    getInput('cfgWebCdn').value = cfg.webui.cdnPrefix;
    getInput('cfgWebUser').value = cfg.webui.user;
    getInput('cfgWebPass').value = cfg.webui.pass;

    getInput('cfgMaxIsp').value = String(cfg.maxNumberOfOneRecords.Isp);
    getInput('cfgMaxIpv4').value = String(cfg.maxNumberOfOneRecords.Ipv4);
    getInput('cfgMaxIpv6').value = String(cfg.maxNumberOfOneRecords.Ipv6);
    getInput('cfgMaxDomain').value = String(cfg.maxNumberOfOneRecords.Domain);

    getInput('rtCron').value = cfg.cron;
    getInput('cronExpr').value = cfg.cron;
  };

  const syncFromInputs = () => {
    cfg.ikuaiUrl = getInput('cfgIkuaiUrl').value;
    cfg.cron = getInput('cfgCron').value;
    cfg.username = getInput('cfgUser').value;
    cfg.password = getInput('cfgPass').value;
    cfg.githubProxy = getInput('cfgGhProxy').value;
    cfg.addErrRetryWait = getInput('cfgRetryWait').value;
    cfg.addWait = getInput('cfgAddWait').value;

    cfg.webui.enable = getSelect('cfgWebEnable').value === '1';
    cfg.webui.enableUpdate = getSelect('cfgWebEnableUpdate').value === '1';
    cfg.webui.port = getInput('cfgWebPort').value;
    cfg.webui.cdnPrefix = getInput('cfgWebCdn').value;
    cfg.webui.user = getInput('cfgWebUser').value;
    cfg.webui.pass = getInput('cfgWebPass').value;

    cfg.maxNumberOfOneRecords.Isp = Number(getInput('cfgMaxIsp').value || 0) || cfg.maxNumberOfOneRecords.Isp;
    cfg.maxNumberOfOneRecords.Ipv4 = Number(getInput('cfgMaxIpv4').value || 0) || cfg.maxNumberOfOneRecords.Ipv4;
    cfg.maxNumberOfOneRecords.Ipv6 = Number(getInput('cfgMaxIpv6').value || 0) || cfg.maxNumberOfOneRecords.Ipv6;
    cfg.maxNumberOfOneRecords.Domain = Number(getInput('cfgMaxDomain').value || 0) || cfg.maxNumberOfOneRecords.Domain;

    getInput('rtCron').value = cfg.cron;
    getInput('cronExpr').value = cfg.cron;
  };

  const renderListSimple = (
    container: HTMLElement,
    arr: Array<{ tag: string; url: string }>,
    onDel: (idx: number) => void,
  ) => {
    container.innerHTML = '';
    for (let i = 0; i < arr.length; i++) {
      const it = arr[i];
      const div = document.createElement('div');
      div.className = 'item';
      div.innerHTML = `
        <div class="kv">
          <div><label>tag</label><input data-k="tag" value="${it.tag || ''}" /></div>
          <div><label>url</label><input data-k="url" value="${it.url || ''}" /></div>
        </div>
        <div class="row" style="justify-content:flex-end;margin-top:8px;">
          <button class="danger" data-del="1">删除</button>
        </div>
      `;
      const del = div.querySelector<HTMLButtonElement>('[data-del]');
      if (del) del.onclick = () => onDel(i);
      const tagEl = div.querySelector<HTMLInputElement>('[data-k="tag"]');
      if (tagEl) tagEl.oninput = (e) => { it.tag = (e.target as HTMLInputElement).value; };
      const urlEl = div.querySelector<HTMLInputElement>('[data-k="url"]');
      if (urlEl) urlEl.oninput = (e) => { it.url = (e.target as HTMLInputElement).value; };
      container.appendChild(div);
    }
  };

  const renderStreamDomain = () => {
    const container = getEl<HTMLElement>('listStreamDomain');
    container.innerHTML = '';
    for (let i = 0; i < cfg.streamDomain.length; i++) {
      const it = cfg.streamDomain[i];
      const div = document.createElement('div');
      div.className = 'item';
      div.innerHTML = `
        <div class="kv">
          <div><label>interface</label><input data-k="interface" value="${it.interface || ''}" /></div>
          <div><label>tag</label><input data-k="tag" value="${it.tag || ''}" /></div>
          <div><label>src-addr</label><input data-k="srcAddr" value="${it.srcAddr || ''}" /></div>
          <div><label>src-addr-opt-ipgroup</label><input data-k="srcAddrOptIpGroup" value="${it.srcAddrOptIpGroup || ''}" /></div>
          <div style="grid-column:1 / -1"><label>url</label><input data-k="url" value="${it.url || ''}" style="width:100%" /></div>
        </div>
        <div class="row" style="justify-content:flex-end;margin-top:8px;">
          <button class="danger" data-del="1">删除</button>
        </div>
      `;
      const del = div.querySelector<HTMLButtonElement>('[data-del]');
      if (del) del.onclick = () => { cfg.streamDomain.splice(i, 1); renderStreamDomain(); };
      const iEl = div.querySelector<HTMLInputElement>('[data-k="interface"]');
      if (iEl) iEl.oninput = (e) => { it.interface = (e.target as HTMLInputElement).value; };
      const tEl = div.querySelector<HTMLInputElement>('[data-k="tag"]');
      if (tEl) tEl.oninput = (e) => { it.tag = (e.target as HTMLInputElement).value; };
      const sEl = div.querySelector<HTMLInputElement>('[data-k="srcAddr"]');
      if (sEl) sEl.oninput = (e) => { it.srcAddr = (e.target as HTMLInputElement).value; };
      const soEl = div.querySelector<HTMLInputElement>('[data-k="srcAddrOptIpGroup"]');
      if (soEl) soEl.oninput = (e) => { it.srcAddrOptIpGroup = (e.target as HTMLInputElement).value; };
      const uEl = div.querySelector<HTMLInputElement>('[data-k="url"]');
      if (uEl) uEl.oninput = (e) => { it.url = (e.target as HTMLInputElement).value; };
      container.appendChild(div);
    }
  };

  const renderStreamIpPort = () => {
    const container = getEl<HTMLElement>('listStreamIpPort');
    container.innerHTML = '';
    for (let i = 0; i < cfg.streamIpPort.length; i++) {
      const it = cfg.streamIpPort[i];
      const div = document.createElement('div');
      div.className = 'item';
      div.innerHTML = `
        <div class="kv">
          <div><label>type</label><input data-k="type" value="${it.type || ''}" /></div>
          <div><label>opt-tagname</label><input data-k="optTagName" value="${it.optTagName || ''}" /></div>
          <div><label>interface</label><input data-k="interface" value="${it.interface || ''}" /></div>
          <div><label>nexthop</label><input data-k="nexthop" value="${it.nexthop || ''}" /></div>
          <div><label>src-addr</label><input data-k="srcAddr" value="${it.srcAddr || ''}" /></div>
          <div><label>src-addr-opt-ipgroup</label><input data-k="srcAddrOptIpGroup" value="${it.srcAddrOptIpGroup || ''}" /></div>
          <div><label>ip-group</label><input data-k="ipGroup" value="${it.ipGroup || ''}" /></div>
          <div><label>mode</label><input data-k="mode" value="${it.mode || '0'}" /></div>
          <div><label>ifaceband</label><input data-k="ifaceband" value="${it.ifaceband || '0'}" /></div>
        </div>
        <div class="row" style="justify-content:flex-end;margin-top:8px;">
          <button class="danger" data-del="1">删除</button>
        </div>
      `;
      const del = div.querySelector<HTMLButtonElement>('[data-del]');
      if (del) del.onclick = () => { cfg.streamIpPort.splice(i, 1); renderStreamIpPort(); };
      const tyEl = div.querySelector<HTMLInputElement>('[data-k="type"]');
      if (tyEl) tyEl.oninput = (e) => { it.type = (e.target as HTMLInputElement).value; };
      const otEl = div.querySelector<HTMLInputElement>('[data-k="optTagName"]');
      if (otEl) otEl.oninput = (e) => { it.optTagName = (e.target as HTMLInputElement).value; };
      const inEl = div.querySelector<HTMLInputElement>('[data-k="interface"]');
      if (inEl) inEl.oninput = (e) => { it.interface = (e.target as HTMLInputElement).value; };
      const nhEl = div.querySelector<HTMLInputElement>('[data-k="nexthop"]');
      if (nhEl) nhEl.oninput = (e) => { it.nexthop = (e.target as HTMLInputElement).value; };
      const saEl = div.querySelector<HTMLInputElement>('[data-k="srcAddr"]');
      if (saEl) saEl.oninput = (e) => { it.srcAddr = (e.target as HTMLInputElement).value; };
      const soEl = div.querySelector<HTMLInputElement>('[data-k="srcAddrOptIpGroup"]');
      if (soEl) soEl.oninput = (e) => { it.srcAddrOptIpGroup = (e.target as HTMLInputElement).value; };
      const ipEl = div.querySelector<HTMLInputElement>('[data-k="ipGroup"]');
      if (ipEl) ipEl.oninput = (e) => { it.ipGroup = (e.target as HTMLInputElement).value; };
      const moEl = div.querySelector<HTMLInputElement>('[data-k="mode"]');
      if (moEl) moEl.oninput = (e) => { it.mode = (e.target as HTMLInputElement).value; };
      const ibEl = div.querySelector<HTMLInputElement>('[data-k="ifaceband"]');
      if (ibEl) ibEl.oninput = (e) => { it.ifaceband = (e.target as HTMLInputElement).value; };
      container.appendChild(div);
    }
  };

  const renderAllLists = () => {
    renderListSimple(getEl('listCustomIsp'), cfg.customIsp, (i) => { cfg.customIsp.splice(i, 1); renderAllLists(); });
    renderListSimple(getEl('listIpGroup'), cfg.ipGroup, (i) => { cfg.ipGroup.splice(i, 1); renderAllLists(); });
    renderListSimple(getEl('listIpv6Group'), cfg.ipv6Group, (i) => { cfg.ipv6Group.splice(i, 1); renderAllLists(); });
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

  const previewYaml = async () => {
    syncFromInputs();
    const payload = toBackendPayload(cfg);
    const mode = getSelect('previewMode').value;
    getTextArea('yamlBox').value = mode === 'comments' ? yamlDumpWithComments(payload, comments) : yamlDump(payload);
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

  const startStream = async () => {
    if (streamReconnectTimer) {
      clearTimeout(streamReconnectTimer);
      streamReconnectTimer = null;
    }
    stopStream();
    try {
      unlistenLogs = await bridge.listenLogs(
        (r) => {
          appendLogLine((r.ts || '') + ' [' + (r.module || '') + '] [' + (r.tag || '') + '] ' + (r.detail || ''));
        },
        () => {
          appendLogLine('[LogStream] 连接断开，3秒后自动重连...');
          if (streamReconnectTimer) {
            clearTimeout(streamReconnectTimer);
          }
          streamReconnectTimer = setTimeout(() => {
            startStream().catch(() => {});
          }, RECONNECT_DELAY);
        },
      );
    } catch (e) {
      const errMsg = e instanceof Error ? e.message : String(e);
      appendLogLine('[LogStream] 连接断开(' + errMsg + ')，3秒后自动重连...');
      streamReconnectTimer = setTimeout(() => {
        startStream().catch(() => {});
      }, RECONNECT_DELAY);
    }
  };

  const stopStream = () => {
    if (streamReconnectTimer) {
      clearTimeout(streamReconnectTimer);
      streamReconnectTimer = null;
    }
    if (unlistenLogs) {
      try { unlistenLogs(); } catch (_) {}
    }
    unlistenLogs = null;
  };

  const initEnvUi = () => {
    const t = bridge.isTauri();
    envBadge.textContent = 'env=' + (t ? 'tauri' : 'web');
    subtitle.textContent = t ? 'App（Tauri v2）' : 'WebUI（Bun + Astro）';
    document.body.classList.toggle('tauri', t);
    if (t) {
      tauriSidebar.style.display = '';
    }
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
  };

  const loadRemoteConfig = async () => {
    const rawUrl = getInput('remoteUrl').value || '';
    if (!rawUrl) {
      setHint(getEl('remoteHint'), 'Remote URL is empty');
      return;
    }
    if (!confirm('确定从远程覆盖当前表单吗？')) return;
    setHint(getEl('remoteHint'), '正在加载...');
    try {
      let finalUrl = rawUrl;
      const proxy = (cfg.githubProxy || '').trim();
      if (proxy && rawUrl.startsWith('https://raw.githubusercontent.com/')) {
        finalUrl = proxy.endsWith('/') ? (proxy + rawUrl) : (proxy + '/' + rawUrl);
      }
      const r = await fetch(finalUrl);
      if (!r.ok) throw new Error(await r.text());
      const doc = yamlParse(await r.text()) || {};
      const meta = {
        ...toBackendPayload(cfg),
        ...doc,
      };
      const parsed = fromBackendMeta(meta);
      Object.assign(cfg, parsed.cfg);
      bindBaseFields();
      renderAllLists();
      await previewYaml();
      saveJson('ikb_remote_url', rawUrl);
      setHint(getEl('remoteHint'), '加载成功');
    } catch (e: any) {
      setHint(getEl('remoteHint'), String(e && e.message ? e.message : e));
    }
  };

  const restoreCmdSettings = () => {
    const saved = loadJson('ikb_cmd', {
      runMode: 'cron',
      module: 'ispdomain',
      cleanTag: 'cleanAll',
      randomSuff: '1',
      pathMode: 'relative',
    });
    getSelect('cmdRunMode').value = saved.runMode;
    getSelect('cmdModule').value = saved.module;
    getInput('cmdCleanTag').value = saved.cleanTag;
    getSelect('cmdRandomSuff').value = saved.randomSuff;
    getSelect('cmdPathMode').value = saved.pathMode;
  };

  const loadPresets = (): CmdPreset[] => loadJson('ikb_cmd_presets', [] as CmdPreset[]);
  const savePresets = (p: CmdPreset[]) => saveJson('ikb_cmd_presets', p);

  const renderPresets = () => {
    const el = getEl<HTMLElement>('presetList');
    const presets = loadPresets();
    el.innerHTML = '';
    if (!presets.length) {
      el.innerHTML = '<div class="hint">暂无预设（最多 5 个）</div>';
      return;
    }
    for (let i = 0; i < presets.length; i++) {
      const p = presets[i];
      const div = document.createElement('div');
      div.className = 'item';
      div.innerHTML = `
        <div class="row" style="justify-content:space-between">
          <b>${p.name || ('预设 ' + (i + 1))}</b>
          <div class="row">
            <button data-load="1">加载</button>
            <button data-rename="1">改名</button>
            <button class="danger" data-del="1">删除</button>
          </div>
        </div>
      `;
      const loadBtn = div.querySelector<HTMLButtonElement>('[data-load]');
      if (loadBtn) loadBtn.onclick = () => {
        getSelect('cmdRunMode').value = p.data.runMode;
        getSelect('cmdModule').value = p.data.module;
        getInput('cmdCleanTag').value = p.data.cleanTag;
        getSelect('cmdRandomSuff').value = p.data.randomSuff;
        getInput('cmdLogin').value = p.data.login;
        renderCmd();
        persistCmdSettings();
      };
      const renameBtn = div.querySelector<HTMLButtonElement>('[data-rename]');
      if (renameBtn) renameBtn.onclick = () => {
        const name = prompt('请输入新名称:', p.name || ('预设 ' + (i + 1)));
        if (!name) return;
        presets[i].name = name;
        savePresets(presets);
        renderPresets();
      };
      const delBtn = div.querySelector<HTMLButtonElement>('[data-del]');
      if (delBtn) delBtn.onclick = () => {
        if (!confirm('确定要删除该预设吗？')) return;
        presets.splice(i, 1);
        savePresets(presets);
        renderPresets();
      };
      el.appendChild(div);
    }
  };

  const persistCmdSettings = () => {
    saveJson('ikb_cmd', {
      runMode: getSelect('cmdRunMode').value,
      module: getSelect('cmdModule').value,
      cleanTag: getInput('cmdCleanTag').value,
      randomSuff: getSelect('cmdRandomSuff').value,
      pathMode: getSelect('cmdPathMode').value,
    });
    saveJson('ikb_export_path', getInput('cmdExportPath').value || '/tmp');
  };

  const wire = () => {
    tabsEl.addEventListener('click', (e) => {
      const b = (e.target as HTMLElement).closest('button[data-tab]') as HTMLElement | null;
      if (!b) return;
      setTab(b.dataset.tab || 'config');
    });

    for (const id of ['cmdPathMode', 'cmdRunMode', 'cmdModule', 'cmdConfigPath', 'cmdCleanTag', 'cmdLogin', 'cmdExportPath', 'cmdRandomSuff']) {
      getEl<HTMLInputElement | HTMLSelectElement>(id).addEventListener('input', () => { renderCmd(); persistCmdSettings(); });
      getEl<HTMLInputElement | HTMLSelectElement>(id).addEventListener('change', () => { renderCmd(); persistCmdSettings(); });
    }

    getEl<HTMLButtonElement>('btnCopyCmd').onclick = async () => {
      await copyText(getEl<HTMLPreElement>('cmdOut').textContent || '');
      setHint(getEl('cmdHint'), '已复制');
      setTimeout(() => setHint(getEl('cmdHint'), ''), 1500);
    };

    getEl<HTMLButtonElement>('btnSavePreset').onclick = () => {
      const presets = loadPresets();
      if (presets.length >= 5) {
        setHint(getEl('presetHint'), '最多只能保存 5 个预设');
        return;
      }
      const name = prompt('请输入预设名称:', '预设 ' + (presets.length + 1));
      if (!name) return;
      presets.push({
        name,
        data: {
          runMode: getSelect('cmdRunMode').value,
          module: getSelect('cmdModule').value,
          cleanTag: getInput('cmdCleanTag').value,
          randomSuff: getSelect('cmdRandomSuff').value,
          login: getInput('cmdLogin').value,
        },
      });
      savePresets(presets);
      renderPresets();
      setHint(getEl('presetHint'), '已保存');
      setTimeout(() => setHint(getEl('presetHint'), ''), 1500);
    };

    getEl<HTMLButtonElement>('btnRestoreDefaults').onclick = () => {
      if (!confirm('重置运行命令参数为默认值？')) return;
      getSelect('cmdRunMode').value = 'cron';
      getSelect('cmdModule').value = 'ispdomain';
      getInput('cmdCleanTag').value = 'cleanAll';
      getSelect('cmdRandomSuff').value = '1';
      getInput('cmdLogin').value = (cfg.ikuaiUrl && cfg.username) ? (cfg.ikuaiUrl + ',' + cfg.username + ',' + cfg.password) : '';
      renderCmd();
      persistCmdSettings();
    };

    getEl<HTMLButtonElement>('themeLight').onclick = () => applyTheme('light');
    getEl<HTMLButtonElement>('themeDark').onclick = () => applyTheme('dark');
    getEl<HTMLButtonElement>('themeAuto').onclick = () => applyTheme('auto');

    getEl<HTMLButtonElement>('btnPreview').onclick = () => previewYaml().catch((e) => setHint(getEl('saveHint'), String(e)));
    getSelect('previewMode').onchange = () => previewYaml().catch(() => {});
    getEl<HTMLButtonElement>('btnCopyYaml').onclick = async () => { await copyText(getTextArea('yamlBox').value || ''); setHint(getEl('saveHint'), '已复制'); };
    getEl<HTMLButtonElement>('btnSaveNoComments').onclick = () => saveConfig(false).catch((e) => setHint(getEl('saveHint'), String(e)));
    getEl<HTMLButtonElement>('btnSaveWithComments').onclick = () => saveConfig(true).catch((e) => setHint(getEl('saveHint'), String(e)));

    getEl<HTMLButtonElement>('btnLoadRemote').onclick = () => loadRemoteConfig();
    getEl<HTMLButtonElement>('btnResetRemote').onclick = () => {
      const def = 'https://raw.githubusercontent.com/joyanhui/ikuai-bypass/refs/heads/main/config_example.yml';
      getInput('remoteUrl').value = def;
      saveJson('ikb_remote_url', def);
    };

    getEl<HTMLButtonElement>('addCustomIsp').onclick = () => { cfg.customIsp.push({ tag: '', url: '' }); renderAllLists(); };
    getEl<HTMLButtonElement>('addIpGroup').onclick = () => { cfg.ipGroup.push({ tag: '', url: '' }); renderAllLists(); };
    getEl<HTMLButtonElement>('addIpv6Group').onclick = () => { cfg.ipv6Group.push({ tag: '', url: '' }); renderAllLists(); };
    getEl<HTMLButtonElement>('addStreamDomain').onclick = () => { cfg.streamDomain.push({ interface: '', srcAddr: '', srcAddrOptIpGroup: '', url: '', tag: '' }); renderAllLists(); };
    getEl<HTMLButtonElement>('addStreamIpPort').onclick = () => { cfg.streamIpPort.push({ optTagName: '', type: '0', interface: '', nexthop: '', srcAddr: '', srcAddrOptIpGroup: '', ipGroup: '', mode: '0', ifaceband: '0' }); renderAllLists(); };

    getEl<HTMLButtonElement>('btnRunOnce').onclick = () => runOnce(getSelect('rtModule').value).catch((e) => setHint(runtimeHint, String(e)));
    getEl<HTMLButtonElement>('btnRunOnce2').onclick = () => runOnce(getSelect('moduleSelect').value).catch((e) => setHint(runtimeHint, String(e)));
    getEl<HTMLButtonElement>('btnCronStart').onclick = () => cronStart(getInput('rtCron').value, getSelect('rtModule').value).catch((e) => setHint(runtimeHint, String(e)));
    getEl<HTMLButtonElement>('btnCronStart2').onclick = () => cronStart(getInput('cronExpr').value, getSelect('moduleSelect').value).catch((e) => setHint(runtimeHint, String(e)));
    getEl<HTMLButtonElement>('btnCronStop').onclick = () => cronStop().catch((e) => setHint(runtimeHint, String(e)));
    getEl<HTMLButtonElement>('btnCronStop2').onclick = () => cronStop().catch((e) => setHint(runtimeHint, String(e)));

    getEl<HTMLButtonElement>('btnTail').onclick = () => tailLogs().catch((e) => appendLogLine(String(e)));
    getEl<HTMLButtonElement>('btnTail2').onclick = () => tailLogs().catch((e) => appendLogLine(String(e)));
    getEl<HTMLButtonElement>('btnStream').style.display = 'none';
    getEl<HTMLButtonElement>('btnStream2').style.display = 'none';
    getEl<HTMLButtonElement>('btnStopStream').style.display = 'none';
    getEl<HTMLButtonElement>('btnStopStream2').style.display = 'none';
  };

  initEnvUi();
  applyTheme(loadJson('ikb_theme', 'auto'));
  getInput('remoteUrl').value = loadJson('ikb_remote_url', 'https://raw.githubusercontent.com/joyanhui/ikuai-bypass/refs/heads/main/config_example.yml');
  restoreCmdSettings();
  renderPresets();
  wire();
  setTab(bridge.isTauri() ? 'runtime' : 'config');
  await loadBackend().catch((e) => setHint(getEl('configHint'), String(e)));
  await previewYaml().catch(() => {});
  await refreshStatus().catch(() => {});
  setInterval(() => refreshStatus().catch(() => {}), 1500);
  startStream().catch(() => {});
};

main().catch(() => {});
