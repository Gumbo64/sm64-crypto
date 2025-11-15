import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// https://vite.dev/config/
export default defineConfig({
  assetsInclude: ['**/*.xor'],
  plugins: [
    react()
  ],
  base: '/sm64-crypto',
})
