import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// ✅ Output to `../frontend/dist`
export default defineConfig({
  plugins: [react()],
  build: {
    outDir: 'dist',
  },
})
