import { parseDocument, YAMLMap, YAMLSeq, isSeq } from 'yaml';

type YamlPath = Array<string | number>;

function ensureDoc(rawYaml: string) {
  const source = rawYaml.trim() ? rawYaml : '{}\n';
  const doc = parseDocument(source);
  if (!(doc.contents instanceof YAMLMap)) {
    doc.contents = new YAMLMap() as unknown as typeof doc.contents;
  }
  return doc;
}

export function updateYamlPaths(
  rawYaml: string,
  updates: Array<{ path: YamlPath; value: unknown }>,
) {
  const doc = ensureDoc(rawYaml);
  for (const update of updates) {
    doc.setIn(update.path, update.value);
  }
  return String(doc);
}

export function upsertYamlSeqItem(
  rawYaml: string,
  path: YamlPath,
  index: number,
  value: unknown,
) {
  const doc = ensureDoc(rawYaml);
  const node = doc.getIn(path, true);
  let seq: YAMLSeq;

  if (isSeq(node)) {
    seq = node as YAMLSeq;
  } else {
    seq = new YAMLSeq(doc.schema);
    doc.setIn(path, seq);
  }

  const itemNode = doc.createNode(value);
  if (index >= 0 && index < seq.items.length) {
    seq.items[index] = itemNode;
  } else {
    seq.items.push(itemNode);
  }

  return String(doc);
}

export function removeYamlSeqItem(rawYaml: string, path: YamlPath, index: number) {
  const doc = ensureDoc(rawYaml);
  const node = doc.getIn(path, true);
  if (isSeq(node)) {
    const seq = node as YAMLSeq;
    if (index >= 0 && index < seq.items.length) {
      seq.items.splice(index, 1);
    }
  }
  return String(doc);
}
