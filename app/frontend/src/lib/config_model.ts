import yaml from 'js-yaml';

export type CmdPreset = {
  name: string;
  data: any;
};

export type UiConfig = {
  ikuaiUrl: string;
  username: string;
  password: string;
  cron: string;
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
    ipGroup: string;
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

function asStr(v: any, fallback = ''): string {
  if (typeof v === 'string') return v;
  if (typeof v === 'number') return String(v);
  if (v == null) return fallback;
  return String(v);
}

export function fromBackendMeta(meta: any): { cfg: UiConfig; comments: CommentMaps; confPath: string } {
  const cfg = defaultUiConfig();
  const comments = defaultCommentMaps();

  const confPath = asStr(meta?.conf_path, '');
  if (meta?.top_level_comments) comments.top = meta.top_level_comments;
  if (meta?.item_comments) comments.item = meta.item_comments;
  if (meta?.webui_comments) comments.webui = meta.webui_comments;
  if (meta?.max_number_of_one_records_comments) comments.maxNumberOfOneRecords = meta.max_number_of_one_records_comments;

  cfg.ikuaiUrl = asStr(meta?.['ikuai-url'], '');
  cfg.username = asStr(meta?.username, '');
  cfg.password = asStr(meta?.password, '');
  cfg.cron = asStr(meta?.cron, '');
  cfg.githubProxy = asStr(meta?.['github-proxy'], '');

  cfg.addErrRetryWait = asStr(meta?.AddErrRetryWait, cfg.addErrRetryWait);
  cfg.addWait = asStr(meta?.AddWait, cfg.addWait);

  if (meta?.MaxNumberOfOneRecords) {
    cfg.maxNumberOfOneRecords = {
      Isp: Number(meta.MaxNumberOfOneRecords.Isp || 5000),
      Ipv4: Number(meta.MaxNumberOfOneRecords.Ipv4 || 1000),
      Ipv6: Number(meta.MaxNumberOfOneRecords.Ipv6 || 1000),
      Domain: Number(meta.MaxNumberOfOneRecords.Domain || 5000),
    };
  }

  if (meta?.webui) {
    cfg.webui.enable = !!meta.webui.enable;
    cfg.webui.port = asStr(meta.webui.port, cfg.webui.port);
    cfg.webui.user = asStr(meta.webui.user, '');
    cfg.webui.pass = asStr(meta.webui.pass, '');
    cfg.webui.cdnPrefix = asStr(meta.webui['cdn-prefix'], cfg.webui.cdnPrefix);
  }

  cfg.customIsp = (meta?.['custom-isp'] || []).map((i: any) => ({ tag: asStr(i.tag), url: asStr(i.url) }));
  cfg.ipGroup = (meta?.['ip-group'] || []).map((i: any) => ({ tag: asStr(i.tag), url: asStr(i.url) }));
  cfg.ipv6Group = (meta?.['ipv6-group'] || []).map((i: any) => ({ tag: asStr(i.tag), url: asStr(i.url) }));
  cfg.streamDomain = (meta?.['stream-domain'] || []).map((i: any) => ({
    interface: asStr(i.interface),
    srcAddr: asStr(i['src-addr']),
    srcAddrOptIpGroup: asStr(i['src-addr-opt-ipgroup']),
    url: asStr(i.url),
    tag: asStr(i.tag),
  }));
  cfg.streamIpPort = (meta?.['stream-ipport'] || []).map((i: any) => ({
    optTagName: asStr(i['opt-tagname']),
    type: asStr(i.type),
    interface: asStr(i.interface),
    nexthop: asStr(i.nexthop),
    srcAddr: asStr(i['src-addr']),
    srcAddrOptIpGroup: asStr(i['src-addr-opt-ipgroup']),
    ipGroup: asStr(i['ip-group']),
    mode: asStr(i.mode ?? '0'),
    ifaceband: asStr(i.ifaceband ?? '0'),
  }));

  return { cfg, comments, confPath };
}

export function toBackendPayload(ui: UiConfig): Record<string, unknown> {
  return {
    'ikuai-url': ui.ikuaiUrl,
    username: ui.username,
    password: ui.password,
    cron: ui.cron,
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
      'ip-group': i.ipGroup,
      mode: Number(i.mode || 0),
      ifaceband: Number(i.ifaceband || 0),
    })),
  };
}

export function yamlDump(payload: any): string {
  return yaml.dump(payload, { lineWidth: 120, noCompatMode: true });
}

function q(s: any): string {
  const v = typeof s === 'string' ? s : (s == null ? '' : String(s));
  if (v === '') return '""';
  const need = /[:#\n\r\t]/.test(v) || /^\s/.test(v) || /\s$/.test(v);
  if (!need) return v;
  return '"' + v.replaceAll('\\', '\\\\').replaceAll('"', '\\"') + '"';
}

export function yamlDumpWithComments(payload: any, comments: CommentMaps): string {
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

  function kv(key: string, value: any, c?: string) {
    cmt(c);
    out.push(key + ': ' + q(value));
  }

  kv('ikuai-url', payload?.['ikuai-url'], top['ikuai-url']);
  kv('username', payload?.username, top.username);
  kv('password', payload?.password, top.password);
  kv('cron', payload?.cron, top.cron);
  kv('AddErrRetryWait', payload?.AddErrRetryWait, top.AddErrRetryWait);
  kv('AddWait', payload?.AddWait, top.AddWait);
  kv('github-proxy', payload?.['github-proxy'], top['github-proxy']);

  cmt(top['custom-isp']);
  out.push('custom-isp:');
  for (const it of payload?.['custom-isp'] || []) {
    out.push('  - tag: ' + q(it.tag));
    if (item.tag) out.push('    # ' + item.tag);
    out.push('    url: ' + q(it.url));
  }

  cmt(top['stream-domain']);
  out.push('stream-domain:');
  for (const it of payload?.['stream-domain'] || []) {
    out.push('  - interface: ' + q(it.interface));
    if (item.interface) out.push('    # ' + item.interface);
    out.push('    src-addr: ' + q(it['src-addr'] || ''));
    if (item['src-addr']) out.push('    # ' + item['src-addr']);
    out.push('    src-addr-opt-ipgroup: ' + q(it['src-addr-opt-ipgroup'] || ''));
    if (item['src-addr-opt-ipgroup']) out.push('    # ' + item['src-addr-opt-ipgroup']);
    out.push('    url: ' + q(it.url));
    out.push('    tag: ' + q(it.tag));
    if (item.tag) out.push('    # ' + item.tag);
  }

  cmt(top['ip-group']);
  out.push('ip-group:');
  for (const it of payload?.['ip-group'] || []) {
    out.push('  - tag: ' + q(it.tag));
    out.push('    url: ' + q(it.url));
  }

  cmt(top['ipv6-group']);
  out.push('ipv6-group:');
  for (const it of payload?.['ipv6-group'] || []) {
    out.push('  - tag: ' + q(it.tag));
    out.push('    url: ' + q(it.url));
  }

  cmt(top['stream-ipport']);
  out.push('stream-ipport:');
  for (const it of payload?.['stream-ipport'] || []) {
    out.push('  - type: ' + q(it.type));
    if (item.type) out.push('    # ' + item.type);
    out.push('    opt-tagname: ' + q(it['opt-tagname'] || ''));
    if (item['opt-tagname']) out.push('    # ' + item['opt-tagname']);
    out.push('    interface: ' + q(it.interface || ''));
    out.push('    nexthop: ' + q(it.nexthop || ''));
    out.push('    src-addr: ' + q(it['src-addr'] || ''));
    out.push('    src-addr-opt-ipgroup: ' + q(it['src-addr-opt-ipgroup'] || ''));
    out.push('    ip-group: ' + q(it['ip-group'] || ''));
    out.push('    mode: ' + String(it.mode ?? 0));
    if (item.mode) out.push('    # ' + item.mode);
    out.push('    ifaceband: ' + String(it.ifaceband ?? 0));
    if (item.ifaceband) out.push('    # ' + item.ifaceband);
  }

  cmt(top.webui);
  out.push('webui:');
  const w = payload?.webui || {};
  out.push('  port: ' + q(w.port || ''));
  if (webui.port) out.push('  # ' + webui.port);
  out.push('  user: ' + q(w.user || ''));
  out.push('  pass: ' + q(w.pass || ''));
  out.push('  enable: ' + (w.enable ? 'true' : 'false'));
  out.push('  cdn-prefix: ' + q(w['cdn-prefix'] || ''));

  cmt(top.MaxNumberOfOneRecords);
  out.push('MaxNumberOfOneRecords:');
  const m = payload?.MaxNumberOfOneRecords || {};
  out.push('  Isp: ' + String(m.Isp ?? 0));
  if (maxn.Isp) out.push('  # ' + maxn.Isp);
  out.push('  Ipv4: ' + String(m.Ipv4 ?? 0));
  out.push('  Ipv6: ' + String(m.Ipv6 ?? 0));
  out.push('  Domain: ' + String(m.Domain ?? 0));

  return out.join('\n') + '\n';
}

export function yamlParse(text: string): any {
  return yaml.load(text);
}
