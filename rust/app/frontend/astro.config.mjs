import { defineConfig } from 'astro/config';

export default defineConfig({
  output: 'static',
  server: {
    host: '0.0.0.0',
    port: 4321,
    strictPort: false,
  },
});
