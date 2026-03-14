declare module 'monaco-editor/esm/vs/editor/editor.api' {
  export * from 'monaco-editor';
  import * as monaco from 'monaco-editor';
  export default monaco;
}

declare module 'monaco-editor/esm/vs/basic-languages/yaml/yaml.js' {
  import type * as monaco from 'monaco-editor';
  export const conf: monaco.languages.LanguageConfiguration;
  export const language: monaco.languages.IMonarchLanguage;
}

declare module 'monaco-editor/esm/vs/editor/editor.worker?worker' {
  const EditorWorker: new () => Worker;
  export default EditorWorker;
}
