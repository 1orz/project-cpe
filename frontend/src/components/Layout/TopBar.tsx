/*
 * @Author: 1orz cloudorzi@gmail.com
 * @Date: 2025-11-22 10:30:41
 * @LastEditors: 1orz cloudorzi@gmail.com
 * @LastEditTime: 2025-12-13 12:43:28
 * @FilePath: /udx710-backend/frontend/src/components/Layout/TopBar.tsx
 * @Description: 
 * 
 * Copyright (c) 2025 by 1orz, All Rights Reserved. 
 */
/*
 * @Author: 1orz cloudorzi@gmail.com
 * @Date: 2025-11-22 10:30:41
 * @LastEditors: 1orz cloudorzi@gmail.com
 * @LastEditTime: 2025-12-13 12:43:22
 * @FilePath: /udx710-backend/frontend/src/components/Layout/TopBar.tsx
 * @Description: 
 * 
 * Copyright (c) 2025 by 1orz, All Rights Reserved. 
 */
import { useState } from 'react'
import {
  AppBar,
  Toolbar,
  Typography,
  IconButton,
  Box,
  Menu,
  MenuItem,
  ListItemIcon,
  ListItemText,
  Divider,
} from '@mui/material'
import {
  Menu as MenuIcon,
  Refresh as RefreshIcon,
  MoreVert as MoreVertIcon,
  Brightness4 as DarkModeIcon,
  Brightness7 as LightModeIcon,
  Speed as SpeedIcon,
} from '@mui/icons-material'
import { useTheme } from '../../contexts/ThemeContext'
import { useRefreshInterval } from '../../contexts/RefreshContext'

interface TopBarProps {
  drawerWidth: number
  onMenuClick: () => void
  refreshInterval: number
  onRefreshIntervalChange: (interval: number) => void
}

export default function TopBar({
  drawerWidth,
  onMenuClick,
  refreshInterval,
  onRefreshIntervalChange,
}: TopBarProps) {
  const { mode, toggleTheme } = useTheme()
  const { triggerRefresh } = useRefreshInterval()
  const [anchorEl, setAnchorEl] = useState<null | HTMLElement>(null)
  const [refreshMenuAnchor, setRefreshMenuAnchor] = useState<null | HTMLElement>(null)

  const handleMenuOpen = (event: React.MouseEvent<HTMLElement>) => {
    setAnchorEl(event.currentTarget)
  }

  const handleMenuClose = () => {
    setAnchorEl(null)
  }

  const handleRefreshMenuOpen = (event: React.MouseEvent<HTMLElement>) => {
    setRefreshMenuAnchor(event.currentTarget)
  }

  const handleRefreshMenuClose = () => {
    setRefreshMenuAnchor(null)
  }

  const handleRefreshIntervalChange = (interval: number) => {
    onRefreshIntervalChange(interval)
    handleRefreshMenuClose()
  }

  const handleRefresh = () => {
    triggerRefresh()
  }

  const handleThemeToggle = () => {
    toggleTheme()
    handleMenuClose()
  }

  const getRefreshLabel = () => {
    if (refreshInterval === 0) return '手动'
    if (refreshInterval === 1000) return '1秒'
    if (refreshInterval === 3000) return '3秒'
    if (refreshInterval === 5000) return '5秒'
    if (refreshInterval === 10000) return '10秒'
    return `${refreshInterval / 1000}秒`
  }

  return (
    <AppBar
      position="fixed"
      sx={{
        width: { sm: `calc(100% - ${drawerWidth}px)` },
        ml: { sm: `${drawerWidth}px` },
      }}
    >
      <Toolbar sx={{ minHeight: { xs: 56, sm: 64 } }}>
        {/* 菜单折叠按钮 - 所有屏幕尺寸都可见 */}
        <IconButton
          color="inherit"
          aria-label="切换侧边栏"
          edge="start"
          onClick={onMenuClick}
          sx={{ mr: 2 }}
        >
          <MenuIcon />
        </IconButton>

        {/* 标题 */}
        <Typography
          variant="h6"
          noWrap
          component="div"
          sx={{
            flexGrow: 1,
            fontSize: { xs: '1rem', sm: '1.25rem' },
          }}
        >
          控制面板
        </Typography>

        {/* 右侧按钮组 */}
        <Box sx={{ display: 'flex', alignItems: 'center', gap: { xs: 0.5, sm: 1 } }}>
          {/* 刷新按钮 - 始终显示 */}
          <IconButton
            color="inherit"
            onClick={handleRefresh}
            title="刷新页面"
            sx={{ display: { xs: 'inline-flex', sm: 'inline-flex' } }}
          >
            <RefreshIcon />
          </IconButton>

          {/* 更多选项按钮 - 折叠其他功能 */}
          <IconButton
            color="inherit"
            onClick={handleMenuOpen}
            title="更多选项"
            sx={{ display: { xs: 'inline-flex', sm: 'inline-flex' } }}
          >
            <MoreVertIcon />
          </IconButton>
        </Box>

        {/* 更多选项菜单 */}
        <Menu
          anchorEl={anchorEl}
          open={Boolean(anchorEl)}
          onClose={handleMenuClose}
          anchorOrigin={{
            vertical: 'bottom',
            horizontal: 'right',
          }}
          transformOrigin={{
            vertical: 'top',
            horizontal: 'right',
          }}
          PaperProps={{
            sx: {
              minWidth: 200,
              mt: 1,
            },
          }}
        >
          {/* 主题切换 */}
          <MenuItem onClick={handleThemeToggle}>
            <ListItemIcon>
              {mode === 'dark' ? <LightModeIcon fontSize="small" /> : <DarkModeIcon fontSize="small" />}
            </ListItemIcon>
            <ListItemText>{mode === 'dark' ? '浅色模式' : '深色模式'}</ListItemText>
          </MenuItem>

          <Divider />

          {/* 刷新频率 */}
          <MenuItem onClick={handleRefreshMenuOpen}>
            <ListItemIcon>
              <SpeedIcon fontSize="small" />
            </ListItemIcon>
            <ListItemText
              primary="刷新频率"
              secondary={getRefreshLabel()}
              secondaryTypographyProps={{ variant: 'caption' }}
            />
          </MenuItem>
        </Menu>

        {/* 刷新频率子菜单 */}
        <Menu
          anchorEl={refreshMenuAnchor}
          open={Boolean(refreshMenuAnchor)}
          onClose={handleRefreshMenuClose}
          anchorOrigin={{
            vertical: 'top',
            horizontal: 'left',
          }}
          transformOrigin={{
            vertical: 'top',
            horizontal: 'right',
          }}
          PaperProps={{
            sx: {
              minWidth: 150,
            },
          }}
        >
          <MenuItem
            selected={refreshInterval === 1000}
            onClick={() => handleRefreshIntervalChange(1000)}
          >
            1秒/次
          </MenuItem>
          <MenuItem
            selected={refreshInterval === 3000}
            onClick={() => handleRefreshIntervalChange(3000)}
          >
            3秒/次
          </MenuItem>
          <MenuItem
            selected={refreshInterval === 5000}
            onClick={() => handleRefreshIntervalChange(5000)}
          >
            5秒/次
          </MenuItem>
          <MenuItem
            selected={refreshInterval === 10000}
            onClick={() => handleRefreshIntervalChange(10000)}
          >
            10秒/次
          </MenuItem>
          <Divider />
          <MenuItem
            selected={refreshInterval === 0}
            onClick={() => handleRefreshIntervalChange(0)}
          >
            手动刷新
          </MenuItem>
        </Menu>
      </Toolbar>
    </AppBar>
  )
}
