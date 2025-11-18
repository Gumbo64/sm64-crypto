import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import wasm from "vite-plugin-wasm";
// https://vite.dev/config/
export default defineConfig({
  assetsInclude: ['**/*.xor'],
  plugins: [
    react(),
    wasm(),
  ],
  base: '/sm64-crypto',
})
