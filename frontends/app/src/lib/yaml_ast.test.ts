import { describe, expect, it } from 'vitest';

import { removeYamlSeqItem, updateYamlPaths, upsertYamlSeqItem } from './yaml_ast';

describe('yaml_ast helpers', () => {
  it('updates scalar paths', () => {
    const out = updateYamlPaths('', [
      { path: ['ikuai-url'], value: 'http://192.168.9.1' },
      { path: ['username'], value: 'admin' },
    ]);
    expect(out).toContain('ikuai-url: http://192.168.9.1');
    expect(out).toContain('username: admin');
  });

  it('upserts sequence item', () => {
    const raw = 'custom-isp:\n  - tag: A\n    url: http://a\n';
    const out = upsertYamlSeqItem(raw, ['custom-isp'], 1, { tag: 'B', url: 'http://b' });
    expect(out).toContain('tag: A');
    expect(out).toContain('tag: B');
  });

  it('removes sequence item', () => {
    const raw = 'ip-group:\n  - tag: A\n    url: http://a\n  - tag: B\n    url: http://b\n';
    const out = removeYamlSeqItem(raw, ['ip-group'], 0);
    expect(out).not.toContain('tag: A');
    expect(out).toContain('tag: B');
  });
});
