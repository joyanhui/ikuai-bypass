import { defineConfig } from 'astro/config';
import tailwindcss from '@tailwindcss/vite';

export default defineConfig({
  output: 'static',
  vite: {
    plugins: [tailwindcss()],
    build: {
      rollupOptions: {
        input: {
          app: new URL('./src/scripts/app.ts', import.meta.url),
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
