import { defineConfig } from 'vite';
import solidPlugin from 'vite-plugin-solid';

export default defineConfig({
  plugins: [solidPlugin()],
  build: {
    target: 'es2020',
    outDir: '../core/target/ui',
    emptyOutDir: true,
  },
  server: {
    port: 3000,
  },
});