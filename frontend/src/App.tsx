/*
 * @Author: 1orz cloudorzi@gmail.com
 * @Date: 2025-12-10 09:19:05
 * @LastEditors: 1orz cloudorzi@gmail.com
 * @LastEditTime: 2025-12-13 12:45:12
 * @FilePath: /udx710-backend/frontend/src/App.tsx
 * @Description: 
 * 
 * Copyright (c) 2025 by 1orz, All Rights Reserved. 
 */
import { lazy, Suspense } from 'react'
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom'
import { QueryClientProvider } from '@tanstack/react-query'
import { Box, CircularProgress } from '@mui/material'
import { ThemeProvider } from './contexts/ThemeContext'
import { queryClient } from './lib/queryClient'
import MainLayout from './components/Layout/MainLayout'

// 路由级别代码分割 - 按需加载页面组件
const Dashboard = lazy(() => import('./pages/Dashboard'))
const DeviceInfo = lazy(() => import('./pages/DeviceInfo'))
const Network = lazy(() => import('./pages/Network'))
const Phone = lazy(() => import('./pages/Phone'))
const SMS = lazy(() => import('./pages/SMS'))
const Configuration = lazy(() => import('./pages/Configuration'))
const ATConsole = lazy(() => import('./pages/ATConsole'))
const Terminal = lazy(() => import('./pages/Terminal'))
const OtaUpdate = lazy(() => import('./pages/OtaUpdate'))

// 页面加载中的 fallback
function PageLoading() {
  return (
    <Box display="flex" justifyContent="center" alignItems="center" minHeight="50vh">
      <CircularProgress size={32} />
    </Box>
  )
}

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <ThemeProvider>
        <BrowserRouter>
          <Routes>
            <Route path="/" element={<MainLayout />}>
              <Route index element={<Suspense fallback={<PageLoading />}><Dashboard /></Suspense>} />
              <Route path="device" element={<Suspense fallback={<PageLoading />}><DeviceInfo /></Suspense>} />
              <Route path="network" element={<Suspense fallback={<PageLoading />}><Network /></Suspense>} />
              {/* 旧路由重定向到网络状态页面 */}
              <Route path="network-interfaces" element={<Navigate to="/network" replace />} />
              <Route path="band-lock" element={<Navigate to="/network" replace />} />
              <Route path="phone" element={<Suspense fallback={<PageLoading />}><Phone /></Suspense>} />
              <Route path="sms" element={<Suspense fallback={<PageLoading />}><SMS /></Suspense>} />
              <Route path="config" element={<Suspense fallback={<PageLoading />}><Configuration /></Suspense>} />
              <Route path="ota" element={<Suspense fallback={<PageLoading />}><OtaUpdate /></Suspense>} />
              <Route path="at-console" element={<Suspense fallback={<PageLoading />}><ATConsole /></Suspense>} />
              <Route path="terminal" element={<Suspense fallback={<PageLoading />}><Terminal /></Suspense>} />
            </Route>
          </Routes>
        </BrowserRouter>
      </ThemeProvider>
    </QueryClientProvider>
  )
}

export default App
