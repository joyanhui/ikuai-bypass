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
    prio: string;
    mode: number;
    ifaceband: string;
  }>;
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

export function fromBackendMeta(meta: unknown): { cfg: UiConfig; confPath: string } {
  const cfg = defaultUiConfig();

  const metaObj = asRecord(meta);
  const confPath = asStr(metaObj.conf_path, '');

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
  cfg.streamIpPort = asArray(metaObj['stream-ipport']).map<UiConfig['streamIpPort'][number]>((i) => {
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
      prio: asStr(item.prio ?? '0'),
      mode: asNum(item.mode, 0),
      ifaceband: asStr(item.ifaceband ?? '0'),
    };
  });

  return { cfg, confPath };
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
      prio: Number(i.prio || 0),
      mode: i.mode,
      ifaceband: Number(i.ifaceband || 0),
    })),
  };
}

export function yamlDump(payload: JsonRecord): string {
  return yaml.dump(payload, { lineWidth: 120, noCompatMode: true });
}

export function yamlParse(text: string): unknown {
  return yaml.load(text);
}
