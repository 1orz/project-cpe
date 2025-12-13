/*
 * @Author: 1orz cloudorzi@gmail.com
 * @Date: 2025-11-22 10:30:41
 * @LastEditors: 1orz cloudorzi@gmail.com
 * @LastEditTime: 2025-12-13 12:43:05
 * @FilePath: /udx710-backend/frontend/src/components/Layout/MainLayout.tsx
 * @Description: 
 * 
 * Copyright (c) 2025 by 1orz, All Rights Reserved. 
 */
import { useState } from 'react'
import { Outlet } from 'react-router-dom'
import { Box, useMediaQuery, useTheme } from '@mui/material'
import Sidebar from './Sidebar'
import TopBar from './TopBar'
import { RefreshContext } from '../../contexts/RefreshContext'

const DRAWER_WIDTH = 240

export default function MainLayout() {
  const theme = useTheme()
  const isMobile = useMediaQuery(theme.breakpoints.down('sm'))
  const [mobileOpen, setMobileOpen] = useState(false)
  const [desktopOpen, setDesktopOpen] = useState(true) // 桌面端侧边栏状态，默认展开
  const [refreshInterval, setRefreshInterval] = useState(3000) // 默认 3 秒（移动端友好）
  const [refreshKey, setRefreshKey] = useState(0)

  const handleDrawerToggle = () => {
    if (isMobile) {
      setMobileOpen(!mobileOpen)
    } else {
      setDesktopOpen(!desktopOpen)
    }
  }

  const triggerRefresh = () => {
    setRefreshKey((prev) => prev + 1)
  }

  return (
    <RefreshContext.Provider
      value={{ refreshInterval, setRefreshInterval, refreshKey, triggerRefresh }}
    >
      <Box sx={{ display: 'flex', minHeight: '100vh' }}>
        {/* 顶部导航栏 */}
        <TopBar
          drawerWidth={desktopOpen ? DRAWER_WIDTH : 0}
          onMenuClick={handleDrawerToggle}
          refreshInterval={refreshInterval}
          onRefreshIntervalChange={setRefreshInterval}
        />

        {/* 侧边栏 */}
        <Sidebar
          drawerWidth={DRAWER_WIDTH}
          mobileOpen={mobileOpen}
          desktopOpen={desktopOpen}
          onClose={handleDrawerToggle}
          isMobile={isMobile}
        />

        {/* 主内容区 */}
        <Box
          component="main"
          sx={{
            flexGrow: 1,
            p: { xs: 2, sm: 3 },
            width: { 
              xs: '100%',
              sm: desktopOpen ? `calc(100% - ${DRAWER_WIDTH}px)` : '100%'
            },
            ml: {
              xs: 0,
              sm: desktopOpen ? 0 : 0
            },
            mt: { xs: 7, sm: 8 },
            minHeight: '100vh',
            backgroundColor: 'background.default',
            transition: theme.transitions.create(['width', 'margin'], {
              easing: theme.transitions.easing.sharp,
              duration: theme.transitions.duration.leavingScreen,
            }),
          }}
        >
          <Outlet />
        </Box>
      </Box>
    </RefreshContext.Provider>
  )
}
