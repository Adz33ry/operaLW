import { defineConfig } from 'vite';

export default defineConfig(({ mode }) => ({
  // Use relative base so packaged Tauri app can load assets
  base: mode === 'development' ? '/' : './',
  server: {
    port: 5173,
    strictPort: true,
    host: true,
  },
  envPrefix: ['VITE_', 'TAURI_'],
}));
