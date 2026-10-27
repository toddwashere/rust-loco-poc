import { defineConfig, loadEnv } from 'vite'
import react from '@vitejs/plugin-react'

// The backend reads its port from `PORT` (see `config/development.yaml`:
// `get_env(name="PORT", default="5150")`), so this proxy has to read the same
// variable. Hardcoding the default silently sends every `/api` call to 5150 the
// moment anyone overrides it.
//
// `loadEnv` is Vite's own reader; the empty prefix takes shell variables too.
// It is used here for the dev server only — nothing is exposed to the client.
export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, '..', '')
  const backendPort = env.PORT || '5150'

  return {
    plugins: [react()],
    // Served by Loco: assets under /static/app/*, the HTML shell at /app/* (src/controllers/spa.rs).
    base: '/static/app/',
    build: { outDir: '../assets/static/app', emptyOutDir: true },
    server: {
      port: 5173,
      // The session cookie is host-scoped, so proxied calls are authenticated too.
      proxy: {
        '/api': `http://localhost:${backendPort}`,
        '/login': `http://localhost:${backendPort}`,
      },
    },
  }
})
