import { defineConfig } from 'astro/config';
import tailwindcss from '@tailwindcss/vite';

export default defineConfig({
  output: 'static',
  vite: {
    plugins: [tailwindcss()],
    build: {
      // Keep build output warning-free; Monaco is intentionally large and lazy-loaded.
      chunkSizeWarningLimit: 4096,
      rollupOptions: {
        output: {
          manualChunks(id) {
            if (id.includes('monaco-editor')) return 'monaco';
            if (id.includes('/yaml') || id.includes('js-yaml') || id.includes('yaml/')) return 'yaml';
          },
        },
      },
    },
  },
  server: {
    host: '0.0.0.0',
    port: 4321,
    strictPort: false,
  },
});
