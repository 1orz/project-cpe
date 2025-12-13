import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import { execSync } from 'child_process'
import { readFileSync } from 'fs'
import { fileURLToPath, URL } from 'node:url'

// Read version and git info at build time
const getVersionInfo = () => {
  try {
    const version = readFileSync('../VERSION', 'utf-8').trim()
    const gitBranch = execSync('git rev-parse --abbrev-ref HEAD').toString().trim()
    const gitCommit = execSync('git rev-parse --short HEAD').toString().trim()
    return { version, gitBranch, gitCommit }
  } catch {
    return { version: '3.0.0', gitBranch: 'unknown', gitCommit: 'unknown' }
  }
}

const { version, gitBranch, gitCommit } = getVersionInfo()

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    react(),
  ],

  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url)),
    },
  },

  define: {
    __APP_VERSION__: JSON.stringify(version),
    __GIT_BRANCH__: JSON.stringify(gitBranch),
    __GIT_COMMIT__: JSON.stringify(gitCommit),
  },
  
  server: {
    port: 5173,
    // 配置开发服务器代理，将 /api 请求转发到 Rust 后端
    proxy: {
      '/api': {
        target: 'http://192.168.66.1:3000',
        changeOrigin: true,
      }
    }
  },

  // 生产构建配置
  build: {
    // outDir: '../www',
    // emptyOutDir: true,
    rollupOptions: {
      output: {
        // 手动分包：将大型依赖分离
        manualChunks: {
          // React 核心
          'vendor-react': ['react', 'react-dom', 'react-router-dom'],
          // MUI 组件库
          'vendor-mui': ['@mui/material', '@mui/icons-material'],
          // MUI 图表
          'vendor-charts': ['@mui/x-charts'],
          // React Query
          'vendor-query': ['@tanstack/react-query'],
        },
      },
    },
  },
})
