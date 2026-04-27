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
      srcAddrInv: '1',
      ipGroup: 'TagA',
      dstAddrInv: '1',
      prio: '5',
      mode: 6,
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
        'src-addr-inv': 1,
        'ip-group': 'TagA',
        'dst-addr-inv': 1,
        prio: 5,
        mode: 6,
        ifaceband: 0,
      },
    ]);
  });

  it('defaults stream-ipport addr inversion flags to zero', () => {
    const { cfg } = fromBackendMeta({
      'stream-ipport': [
        {
          'opt-tagname': 'RouteA',
          type: '1',
          interface: '',
          nexthop: '192.168.1.2',
          'src-addr': '192.168.1.10-192.168.1.20',
          'src-addr-opt-ipgroup': '',
          'ip-group': 'TagA',
          prio: 0,
          mode: 0,
          ifaceband: 0,
        },
      ],
    });

    expect(cfg.streamIpPort).toEqual([
      {
        optTagName: 'RouteA',
        type: '1',
        interface: '',
        nexthop: '192.168.1.2',
        srcAddr: '192.168.1.10-192.168.1.20',
        srcAddrOptIpGroup: '',
        srcAddrInv: '0',
        ipGroup: 'TagA',
        dstAddrInv: '0',
        prio: '0',
        mode: 0,
        ifaceband: '0',
      },
    ]);
  });

  it('normalizes stream-ipport addr inversion flags to binary toggles', () => {
    const { cfg } = fromBackendMeta({
      'stream-ipport': [
        {
          'opt-tagname': 'RouteA',
          type: '1',
          interface: '',
          nexthop: '192.168.1.2',
          'src-addr': '192.168.1.10-192.168.1.20',
          'src-addr-opt-ipgroup': '',
          'src-addr-inv': 2,
          'ip-group': 'TagA',
          'dst-addr-inv': -1,
          mode: 0,
          ifaceband: 0,
        },
      ],
    });

    expect(cfg.streamIpPort[0]?.srcAddrInv).toBe('0');
    expect(cfg.streamIpPort[0]?.dstAddrInv).toBe('0');
    expect(cfg.streamIpPort[0]?.prio).toBe('0');
  });

  it('loads stream-ipport prio from backend meta', () => {
    const { cfg } = fromBackendMeta({
      'stream-ipport': [
        {
          'opt-tagname': 'RouteA',
          type: '1',
          interface: '',
          nexthop: '192.168.1.2',
          'src-addr': '192.168.1.10-192.168.1.20',
          'src-addr-opt-ipgroup': '',
          'ip-group': 'TagA',
          prio: 63,
          mode: 6,
          ifaceband: 0,
        },
      ],
    });

    expect(cfg.streamIpPort[0]?.prio).toBe('63');
    expect(cfg.streamIpPort[0]?.mode).toBe(6);
  });

  it('roundtrips stream-ipport mode as numeric enum value', () => {
    const { cfg } = fromBackendMeta({
      'stream-ipport': [
        {
          'opt-tagname': 'RouteA',
          type: '1',
          interface: '',
          nexthop: '192.168.1.2',
          'src-addr': '192.168.1.10-192.168.1.20',
          'src-addr-opt-ipgroup': '',
          'src-addr-inv': 1,
          'ip-group': 'TagA',
          'dst-addr-inv': 0,
          prio: 63,
          mode: 6,
          ifaceband: 0,
        },
      ],
    });

    expect(cfg.streamIpPort[0]?.mode).toBe(6);
    expect(toBackendPayload(cfg)['stream-ipport']).toEqual([
      {
        'opt-tagname': 'RouteA',
        type: '1',
        interface: '',
        nexthop: '192.168.1.2',
        'src-addr': '192.168.1.10-192.168.1.20',
        'src-addr-opt-ipgroup': '',
        'src-addr-inv': 1,
        'ip-group': 'TagA',
        'dst-addr-inv': 0,
        prio: 63,
        mode: 6,
        ifaceband: 0,
      },
    ]);
  });

});
