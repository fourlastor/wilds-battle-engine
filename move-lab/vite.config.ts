import { defineConfig } from 'vite';

// Relative base so the built site works from any path (for example GitHub Pages).
export default defineConfig({
  base: './',
  server: {
    // The built-in move files live one level up, in the engine repo's moves/ folder.
    fs: { allow: ['..'] },
  },
  build: {
    target: 'es2022',
    chunkSizeWarningLimit: 1600,
  },
});
