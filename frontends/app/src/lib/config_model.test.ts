import { describe, expect, it } from 'vitest';

import {
  defaultUiConfig,
  fromBackendMeta,
  toBackendPayload,
} from './config_model';

describe('config_model', () => {
  it('loads backend meta with expected defaults', () => {
    const { cfg, confPath } = fromBackendMeta({
      conf_path: '/tmp/config.yml',
      'ikuai-url': 'http://192.168.9.1',
      username: 'admin',
      password: 'admin888',
      proxy: {
        mode: 'smart',
      },
      webui: {
        enable: true,
        port: '19001',
      },
      'custom-isp': [{ tag: 'A', url: 'http://x/isp.txt' }],
    });

    expect(confPath).toBe('/tmp/config.yml');
    expect(cfg.ikuaiUrl).toBe('http://192.168.9.1');
    expect(cfg.proxy.mode).toBe('smart');
    expect(cfg.webui.enable).toBe(true);
    expect(cfg.customIsp).toEqual([{ tag: 'A', url: 'http://x/isp.txt' }]);
  });

  it('converts ui config to backend payload schema', () => {
    const cfg = defaultUiConfig();
    cfg.ikuaiUrl = 'http://192.168.9.1';
    cfg.username = 'admin';
    cfg.password = 'admin888';
    cfg.streamIpPort.push({
      optTagName: 'RouteA',
      type: '1',
      interface: '',
      nexthop: '192.168.1.2',
      srcAddr: '192.168.1.10-192.168.1.20',
      srcAddrOptIpGroup: '',
      ipGroup: 'TagA',
      mode: '0',
      ifaceband: '0',
    });

    const payload = toBackendPayload(cfg);
    expect(payload['ikuai-url']).toBe('http://192.168.9.1');
    expect(payload.username).toBe('admin');
    expect(payload['stream-ipport']).toEqual([
      {
        'opt-tagname': 'RouteA',
        type: '1',
        interface: '',
        nexthop: '192.168.1.2',
        'src-addr': '192.168.1.10-192.168.1.20',
        'src-addr-opt-ipgroup': '',
        'ip-group': 'TagA',
        mode: 0,
        ifaceband: 0,
      },
    ]);
  });
});
