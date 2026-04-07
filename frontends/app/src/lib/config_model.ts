import yaml from 'js-yaml';

type JsonPrimitive = null | boolean | number | string;
type JsonValue = JsonPrimitive | JsonValue[] | { [k: string]: JsonValue };
type JsonRecord = Record<string, JsonValue>;
type UnknownRecord = Record<string, unknown>;

export type CmdPreset = {
  name: string;
  data: JsonRecord;
};

export type UiConfig = {
  ikuaiUrl: string;
  username: string;
  password: string;
  cron: string;
  proxy: {
    mode: 'custom' | 'system' | 'smart';
    url: string;
    user: string;
    pass: string;
  };
  githubProxy: string;
  addErrRetryWait: string;
  addWait: string;
  webui: {
    enable: boolean;
    port: string;
    user: string;
    pass: string;
    cdnPrefix: string;
  };
  maxNumberOfOneRecords: {
    Isp: number;
    Ipv4: number;
    Ipv6: number;
    Domain: number;
  };
  customIsp: Array<{ tag: string; url: string }>;
  ipGroup: Array<{ tag: string; url: string }>;
  ipv6Group: Array<{ tag: string; url: string }>;
  streamDomain: Array<{
    interface: string;
    srcAddr: string;
    srcAddrOptIpGroup: string;
    url: string;
    tag: string;
  }>;
  streamIpPort: Array<{
    optTagName: string;
    type: string;
    interface: string;
    nexthop: string;
    srcAddr: string;
    srcAddrOptIpGroup: string;
    srcAddrInv: string;
    ipGroup: string;
    dstAddrInv: string;
    mode: string;
    ifaceband: string;
  }>;
};

export type CommentMaps = {
  top: Record<string, string>;
  item: Record<string, string>;
  webui: Record<string, string>;
  maxNumberOfOneRecords: Record<string, string>;
};

export function defaultUiConfig(): UiConfig {
  return {
    ikuaiUrl: '',
    username: '',
    password: '',
    cron: '',
    proxy: {
      mode: 'smart',
      url: '',
      user: '',
      pass: '',
    },
    githubProxy: '',
    addErrRetryWait: '10s',
    addWait: '1s',
    webui: {
      enable: false,
      port: '19001',
      user: '',
      pass: '',
      cdnPrefix: 'https://cdn.jsdelivr.net/npm',
    },
    maxNumberOfOneRecords: {
      Isp: 5000,
      Ipv4: 1000,
      Ipv6: 1000,
      Domain: 5000,
    },
    customIsp: [],
    ipGroup: [],
    ipv6Group: [],
    streamDomain: [],
    streamIpPort: [],
  };
}

export function defaultCommentMaps(): CommentMaps {
  return {
    top: {},
    item: {},
    webui: {},
    maxNumberOfOneRecords: {},
  };
}

function isRecord(v: unknown): v is UnknownRecord {
  return typeof v === 'object' && v !== null && !Array.isArray(v);
}

function asRecord(v: unknown): UnknownRecord {
  return isRecord(v) ? v : {};
}

function asArray(v: unknown): unknown[] {
  return Array.isArray(v) ? v : [];
}

function asNum(v: unknown, fallback: number): number {
  if (typeof v === 'number' && !Number.isNaN(v)) return v;
  if (typeof v === 'string') {
    const trimmed = v.trim();
    if (trimmed) {
      const parsed = Number(trimmed);
      if (!Number.isNaN(parsed)) return parsed;
    }
  }
  return fallback;
}

function asStr(v: unknown, fallback = ''): string {
  if (typeof v === 'string') return v;
  if (typeof v === 'number') return String(v);
  if (v == null) return fallback;
  return String(v);
}

function asToggleStr(v: unknown): string {
  return asNum(v, 0) === 1 ? '1' : '0';
}

function asStringMap(v: unknown): Record<string, string> {
  const obj = asRecord(v);
  const out: Record<string, string> = {};
  for (const [k, val] of Object.entries(obj)) {
    out[k] = asStr(val);
  }
  return out;
}

export function fromBackendMeta(meta: unknown): { cfg: UiConfig; comments: CommentMaps; confPath: string } {
  const cfg = defaultUiConfig();
  const comments = defaultCommentMaps();

  const metaObj = asRecord(meta);
  const confPath = asStr(metaObj.conf_path, '');
  comments.top = asStringMap(metaObj.top_level_comments);
  comments.item = asStringMap(metaObj.item_comments);
  comments.webui = asStringMap(metaObj.webui_comments);
  comments.maxNumberOfOneRecords = asStringMap(metaObj.max_number_of_one_records_comments);

  cfg.ikuaiUrl = asStr(metaObj['ikuai-url'], '');
  cfg.username = asStr(metaObj.username, '');
  cfg.password = asStr(metaObj.password, '');
  cfg.cron = asStr(metaObj.cron, '');

  if (metaObj.proxy) {
    const p = asRecord(metaObj.proxy);
    const modeRaw = asStr(p.mode, cfg.proxy.mode);
    cfg.proxy.mode =
      modeRaw === 'custom'
        ? 'custom'
        : modeRaw === 'smart' || modeRaw === 'onlyGithubApi' || modeRaw === 'only-github-api' || modeRaw === 'only_github_api'
          ? 'smart'
          : 'system';
    cfg.proxy.url = asStr(p.url, cfg.proxy.url);
    cfg.proxy.user = asStr(p.user, cfg.proxy.user);
    cfg.proxy.pass = asStr(p.pass, cfg.proxy.pass);
  }
  cfg.githubProxy = asStr(metaObj['github-proxy'], '');

  cfg.addErrRetryWait = asStr(metaObj.AddErrRetryWait, cfg.addErrRetryWait);
  cfg.addWait = asStr(metaObj.AddWait, cfg.addWait);

  if (metaObj.MaxNumberOfOneRecords) {
    const max = asRecord(metaObj.MaxNumberOfOneRecords);
    cfg.maxNumberOfOneRecords = {
      Isp: asNum(max.Isp, 5000),
      Ipv4: asNum(max.Ipv4, 1000),
      Ipv6: asNum(max.Ipv6, 1000),
      Domain: asNum(max.Domain, 5000),
    };
  }

  if (metaObj.webui) {
    const webui = asRecord(metaObj.webui);
    cfg.webui.enable = !!webui.enable;
    cfg.webui.port = asStr(webui.port, cfg.webui.port);
    cfg.webui.user = asStr(webui.user, '');
    cfg.webui.pass = asStr(webui.pass, '');
    cfg.webui.cdnPrefix = asStr(webui['cdn-prefix'], cfg.webui.cdnPrefix);
  }

  cfg.customIsp = asArray(metaObj['custom-isp']).map((i) => {
    const item = asRecord(i);
    return { tag: asStr(item.tag), url: asStr(item.url) };
  });
  cfg.ipGroup = asArray(metaObj['ip-group']).map((i) => {
    const item = asRecord(i);
    return { tag: asStr(item.tag), url: asStr(item.url) };
  });
  cfg.ipv6Group = asArray(metaObj['ipv6-group']).map((i) => {
    const item = asRecord(i);
    return { tag: asStr(item.tag), url: asStr(item.url) };
  });
  cfg.streamDomain = asArray(metaObj['stream-domain']).map((i) => {
    const item = asRecord(i);
    return {
      interface: asStr(item.interface),
      srcAddr: asStr(item['src-addr']),
      srcAddrOptIpGroup: asStr(item['src-addr-opt-ipgroup']),
      url: asStr(item.url),
      tag: asStr(item.tag),
    };
  });
  cfg.streamIpPort = asArray(metaObj['stream-ipport']).map((i) => {
    const item = asRecord(i);
    return {
      optTagName: asStr(item['opt-tagname']),
      type: asStr(item.type),
      interface: asStr(item.interface),
      nexthop: asStr(item.nexthop),
      srcAddr: asStr(item['src-addr']),
      srcAddrOptIpGroup: asStr(item['src-addr-opt-ipgroup']),
      srcAddrInv: asToggleStr(item['src-addr-inv']),
      ipGroup: asStr(item['ip-group']),
      dstAddrInv: asToggleStr(item['dst-addr-inv']),
      mode: asStr(item.mode ?? '0'),
      ifaceband: asStr(item.ifaceband ?? '0'),
    };
  });

  return { cfg, comments, confPath };
}

export function toBackendPayload(ui: UiConfig): JsonRecord {
  return {
    'ikuai-url': ui.ikuaiUrl,
    username: ui.username,
    password: ui.password,
    cron: ui.cron,
    proxy: {
      mode: ui.proxy.mode,
      url: ui.proxy.url,
      user: ui.proxy.user,
      pass: ui.proxy.pass,
    },
    'github-proxy': ui.githubProxy,
    AddErrRetryWait: ui.addErrRetryWait,
    AddWait: ui.addWait,
    MaxNumberOfOneRecords: {
      Isp: Number(ui.maxNumberOfOneRecords.Isp),
      Ipv4: Number(ui.maxNumberOfOneRecords.Ipv4),
      Ipv6: Number(ui.maxNumberOfOneRecords.Ipv6),
      Domain: Number(ui.maxNumberOfOneRecords.Domain),
    },
    webui: {
      enable: !!ui.webui.enable,
      port: String(ui.webui.port),
      user: ui.webui.user,
      pass: ui.webui.pass,
      'cdn-prefix': ui.webui.cdnPrefix,
    },
    'custom-isp': ui.customIsp.map((i) => ({ tag: i.tag, url: i.url })),
    'stream-domain': ui.streamDomain.map((i) => ({
      interface: i.interface,
      'src-addr': i.srcAddr,
      'src-addr-opt-ipgroup': i.srcAddrOptIpGroup,
      url: i.url,
      tag: i.tag,
    })),
    'ip-group': ui.ipGroup.map((i) => ({ tag: i.tag, url: i.url })),
    'ipv6-group': ui.ipv6Group.map((i) => ({ tag: i.tag, url: i.url })),
    'stream-ipport': ui.streamIpPort.map((i) => ({
      'opt-tagname': i.optTagName,
      type: String(i.type),
      interface: i.interface,
      nexthop: i.nexthop,
      'src-addr': i.srcAddr,
      'src-addr-opt-ipgroup': i.srcAddrOptIpGroup,
      'src-addr-inv': i.srcAddrInv === '1' ? 1 : 0,
      'ip-group': i.ipGroup,
      'dst-addr-inv': i.dstAddrInv === '1' ? 1 : 0,
      mode: Number(i.mode || 0),
      ifaceband: Number(i.ifaceband || 0),
    })),
  };
}

export function yamlDump(payload: JsonRecord): string {
  return yaml.dump(payload, { lineWidth: 120, noCompatMode: true });
}

function q(s: unknown): string {
  const v = typeof s === 'string' ? s : (s == null ? '' : String(s));
  if (v === '') return '""';
  const need = /[:#\n\r\t]/.test(v) || /^\s/.test(v) || /\s$/.test(v);
  if (!need) return v;
  return '"' + v.replaceAll('\\', '\\\\').replaceAll('"', '\\"') + '"';
}

export function yamlDumpWithComments(payload: JsonRecord, comments: CommentMaps): string {
  const top = comments?.top || {};
  const item = comments?.item || {};
  const webui = comments?.webui || {};
  const maxn = comments?.maxNumberOfOneRecords || {};

  const out: string[] = [];
  out.push('#  iKuai Bypass 配置文件 大部分时候请使用默认设置即可');
  out.push('#  详情参考: https://github.com/joyanhui/ikuai-bypass');
  out.push('#');
  out.push('#  【重要】tag 字段长度限制说明 / Important: tag field length limitation');
  out.push('#  爱快固件 4.0.101 对规则名称(tagname)有 15 字符的长度限制');
  out.push('#  系统会自动添加 "IKB" 前缀，因此 tag 字段建议不超过 11 个字符');
  out.push('#  超过限制的 tag 会被自动截断并打印警告日志');

  function cmt(s?: string) {
    if (!s) return;
    out.push('');
    out.push('# ' + s);
  }

  function kv(key: string, value: unknown, c?: string) {
    cmt(c);
    out.push(key + ': ' + q(value));
  }

  kv('ikuai-url', payload?.['ikuai-url'], top['ikuai-url']);
  kv('username', payload?.username, top.username);
  kv('password', payload?.password, top.password);
  kv('cron', payload?.cron, top.cron);
  kv('AddErrRetryWait', payload?.AddErrRetryWait, top.AddErrRetryWait);
  kv('AddWait', payload?.AddWait, top.AddWait);

  // proxy
  cmt(top.proxy);
  const p = asRecord(payload?.proxy);
  const modeRaw = asStr(p.mode, 'system');
  const mode =
    modeRaw === 'custom'
      ? 'custom'
      : modeRaw === 'smart' || modeRaw === 'onlyGithubApi' || modeRaw === 'only-github-api' || modeRaw === 'only_github_api'
        ? 'smart'
        : 'system';
  const url = asStr(p.url, 'http://127.0.0.1:7890');
  const user = asStr(p.user, '');
  const pass = asStr(p.pass, '');
  out.push('proxy:');
  out.push('  mode: ' + mode);
  out.push('  url: ' + q(url));
  out.push('  user: ' + q(user));
  out.push('  pass: ' + q(pass));

  kv('github-proxy', payload?.['github-proxy'], top['github-proxy']);

  cmt(top['custom-isp']);
  out.push('custom-isp:');
  for (const itRaw of asArray(payload?.['custom-isp'])) {
    const it = asRecord(itRaw);
    out.push('  - tag: ' + q(asStr(it.tag)));
    if (item.tag) out.push('    # ' + item.tag);
    out.push('    url: ' + q(asStr(it.url)));
  }

  cmt(top['stream-domain']);
  out.push('stream-domain:');
  for (const itRaw of asArray(payload?.['stream-domain'])) {
    const it = asRecord(itRaw);
    out.push('  - interface: ' + q(asStr(it.interface)));
    if (item.interface) out.push('    # ' + item.interface);
    out.push('    src-addr: ' + q(asStr(it['src-addr'], '')));
    if (item['src-addr']) out.push('    # ' + item['src-addr']);
    out.push('    src-addr-opt-ipgroup: ' + q(asStr(it['src-addr-opt-ipgroup'], '')));
    if (item['src-addr-opt-ipgroup']) out.push('    # ' + item['src-addr-opt-ipgroup']);
    out.push('    url: ' + q(asStr(it.url)));
    out.push('    tag: ' + q(asStr(it.tag)));
    if (item.tag) out.push('    # ' + item.tag);
  }

  cmt(top['ip-group']);
  out.push('ip-group:');
  for (const itRaw of asArray(payload?.['ip-group'])) {
    const it = asRecord(itRaw);
    out.push('  - tag: ' + q(asStr(it.tag)));
    out.push('    url: ' + q(asStr(it.url)));
  }

  cmt(top['ipv6-group']);
  out.push('ipv6-group:');
  for (const itRaw of asArray(payload?.['ipv6-group'])) {
    const it = asRecord(itRaw);
    out.push('  - tag: ' + q(asStr(it.tag)));
    out.push('    url: ' + q(asStr(it.url)));
  }

  cmt(top['stream-ipport']);
  out.push('stream-ipport:');
  for (const itRaw of asArray(payload?.['stream-ipport'])) {
    const it = asRecord(itRaw);
    out.push('  - type: ' + q(asStr(it.type)));
    if (item.type) out.push('    # ' + item.type);
    out.push('    opt-tagname: ' + q(asStr(it['opt-tagname'], '')));
    if (item['opt-tagname']) out.push('    # ' + item['opt-tagname']);
    out.push('    interface: ' + q(asStr(it.interface, '')));
    out.push('    nexthop: ' + q(asStr(it.nexthop, '')));
    out.push('    src-addr: ' + q(asStr(it['src-addr'], '')));
    out.push('    src-addr-opt-ipgroup: ' + q(asStr(it['src-addr-opt-ipgroup'], '')));
    out.push('    src-addr-inv: ' + String(asNum(it['src-addr-inv'], 0)));
    if (item['src-addr-inv']) out.push('    # ' + item['src-addr-inv']);
    out.push('    ip-group: ' + q(asStr(it['ip-group'], '')));
    out.push('    dst-addr-inv: ' + String(asNum(it['dst-addr-inv'], 0)));
    if (item['dst-addr-inv']) out.push('    # ' + item['dst-addr-inv']);
    out.push('    mode: ' + String(asNum(it.mode, 0)));
    if (item.mode) out.push('    # ' + item.mode);
    out.push('    ifaceband: ' + String(asNum(it.ifaceband, 0)));
    if (item.ifaceband) out.push('    # ' + item.ifaceband);
  }

  cmt(top.webui);
  out.push('webui:');
  const w = asRecord(payload?.webui);
  out.push('  port: ' + q(asStr(w.port, '')));
  if (webui.port) out.push('  # ' + webui.port);
  out.push('  user: ' + q(asStr(w.user, '')));
  out.push('  pass: ' + q(asStr(w.pass, '')));
  out.push('  enable: ' + (!!w.enable ? 'true' : 'false'));
  out.push('  cdn-prefix: ' + q(asStr(w['cdn-prefix'], '')));

  cmt(top.MaxNumberOfOneRecords);
  out.push('MaxNumberOfOneRecords:');
  const m = asRecord(payload?.MaxNumberOfOneRecords);
  out.push('  Isp: ' + String(asNum(m.Isp, 0)));
  if (maxn.Isp) out.push('  # ' + maxn.Isp);
  out.push('  Ipv4: ' + String(asNum(m.Ipv4, 0)));
  out.push('  Ipv6: ' + String(asNum(m.Ipv6, 0)));
  out.push('  Domain: ' + String(asNum(m.Domain, 0)));

  return out.join('\n') + '\n';
}

export function yamlParse(text: string): unknown {
  return yaml.load(text);
}
