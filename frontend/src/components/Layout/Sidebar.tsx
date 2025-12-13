/*
 * @Author: 1orz cloudorzi@gmail.com
 * @Date: 2025-12-10 09:19:05
 * @LastEditors: 1orz cloudorzi@gmail.com
 * @LastEditTime: 2025-12-13 12:43:08
 * @FilePath: /udx710-backend/frontend/src/components/Layout/Sidebar.tsx
 * @Description: 
 * 
 * Copyright (c) 2025 by 1orz, All Rights Reserved. 
 */
import { useNavigate, useLocation } from 'react-router-dom'
import {
  Drawer,
  List,
  ListItem,
  ListItemButton,
  ListItemIcon,
  ListItemText,
  Toolbar,
  Divider,
  Box,
  Typography,
  Link,
} from '@mui/material'
import {
  Dashboard as DashboardIcon,
  Devices as DevicesIcon,
  SignalCellularAlt as SignalIcon,
  Settings as SettingsIcon,
  Terminal as TerminalIcon,
  Phone as PhoneIcon,
  Sms as SmsIcon,
  GitHub as GitHubIcon,
  WebAsset as WebTerminalIcon,
  SystemUpdateAlt as OtaIcon,
} from '@mui/icons-material'

interface SidebarProps {
  drawerWidth: number
  mobileOpen: boolean
  desktopOpen: boolean
  onClose: () => void
  isMobile: boolean
}

// 导航菜单项（已整合网络接口和频段锁定到网络状态）
const menuItems = [
  { path: '/', label: '仪表盘', icon: DashboardIcon },
  { path: '/device', label: '设备信息', icon: DevicesIcon },
  { path: '/network', label: '网络状态', icon: SignalIcon },
  { path: '/phone', label: '电话管理', icon: PhoneIcon },
  { path: '/sms', label: '短信管理', icon: SmsIcon },
  { path: '/config', label: '系统配置', icon: SettingsIcon },
  { path: '/ota', label: 'OTA 更新', icon: OtaIcon },
  { path: '/at-console', label: 'AT控制台', icon: TerminalIcon },
  { path: '/terminal', label: 'Web终端', icon: WebTerminalIcon },
]

export default function Sidebar({ drawerWidth, mobileOpen, desktopOpen, onClose, isMobile }: SidebarProps) {
  const navigate = useNavigate()
  const location = useLocation()

  const handleNavigation = (path: string) => {
    void navigate(path)
    if (isMobile) {
      onClose()
    }
  }

  const drawer = (
    <Box sx={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
      <Toolbar>
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
          <Typography variant="h6" noWrap component="div" fontWeight={600}>
            UDX710
          </Typography>
        </Box>
      </Toolbar>
      <Divider />
      <List sx={{ flexGrow: 1 }}>
        {menuItems.map((item) => {
          const IconComponent = item.icon
          return (
            <ListItem key={item.path} disablePadding>
              <ListItemButton
                selected={location.pathname === item.path}
                onClick={() => handleNavigation(item.path)}
              >
                <ListItemIcon>
                  <IconComponent />
                </ListItemIcon>
                <ListItemText primary={item.label} />
              </ListItemButton>
            </ListItem>
          )
        })}
      </List>
      {/* Footer with copyright */}
      <Box sx={{ p: 2, borderTop: 1, borderColor: 'divider' }}>
        <Link
          href="https://github.com/1orz/project-cpe"
          target="_blank"
          rel="noopener noreferrer"
          sx={{
            display: 'flex',
            alignItems: 'center',
            gap: 0.5,
            color: 'text.secondary',
            textDecoration: 'none',
            fontSize: '0.75rem',
            '&:hover': {
              color: 'primary.main',
            },
          }}
        >
          <GitHubIcon sx={{ fontSize: 16 }} />
          <Typography variant="caption" color="inherit">
            1orz/project-cpe
          </Typography>
        </Link>
        <Typography variant="caption" color="text.disabled" sx={{ display: 'block', mt: 0.5 }}>
          v{__APP_VERSION__} ({__GIT_BRANCH__}/{__GIT_COMMIT__})
        </Typography>
        <Typography variant="caption" color="text.disabled" sx={{ display: 'block', mt: 0.5 }}>
          Copyright 2025 1orz
        </Typography>
      </Box>
    </Box>
  )

  return (
    <Box
      component="nav"
      sx={{ 
        width: { xs: 0, sm: desktopOpen ? drawerWidth : 0 },
        flexShrink: { sm: 0 },
        transition: 'width 0.3s',
      }}
    >
      {/* 移动端抽屉 */}
      <Drawer
        variant="temporary"
        open={mobileOpen}
        onClose={onClose}
        ModalProps={{
          keepMounted: true, // 提升移动端性能
        }}
        sx={{
          display: { xs: 'block', sm: 'none' },
          '& .MuiDrawer-paper': {
            boxSizing: 'border-box',
            width: drawerWidth,
          },
        }}
      >
        {drawer}
      </Drawer>

      {/* 桌面端可折叠抽屉 */}
      <Drawer
        variant="persistent"
        open={desktopOpen}
        sx={{
          display: { xs: 'none', sm: 'block' },
          '& .MuiDrawer-paper': {
            boxSizing: 'border-box',
            width: drawerWidth,
            transition: 'transform 0.3s',
          },
        }}
      >
        {drawer}
      </Drawer>
    </Box>
  )
}

