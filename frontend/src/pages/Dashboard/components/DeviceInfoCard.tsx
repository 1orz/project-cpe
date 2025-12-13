/*
 * @Author: 1orz cloudorzi@gmail.com
 * @Date: 2025-12-10 10:21:46
 * @LastEditors: 1orz cloudorzi@gmail.com
 * @LastEditTime: 2025-12-13 12:44:19
 * @FilePath: /udx710-backend/frontend/src/pages/Dashboard/components/DeviceInfoCard.tsx
 * @Description: 
 * 
 * Copyright (c) 2025 by 1orz, All Rights Reserved. 
 */
import { useState } from 'react'
import { Box, Card, CardContent, Typography, IconButton, Tooltip } from '@mui/material'
import { Router, Visibility, VisibilityOff } from '@mui/icons-material'
import { getSensitiveStyle } from '../utils'
import type { DeviceInfo, SystemStatsResponse } from '@/api/types'

interface DeviceInfoCardProps {
  deviceInfo: DeviceInfo | null
  systemStats: SystemStatsResponse | null
}

export function DeviceInfoCard({ deviceInfo, systemStats }: DeviceInfoCardProps) {
  const [showInfo, setShowInfo] = useState(false)

  return (
    <Card>
      <CardContent sx={{ py: 1.5, '&:last-child': { pb: 1.5 } }}>
        <Box display="flex" alignItems="center" justifyContent="space-between" mb={1}>
          <Box display="flex" alignItems="center" gap={1}>
            <Router fontSize="small" color="primary" />
            <Typography variant="subtitle2" fontWeight="medium">设备信息</Typography>
          </Box>
          <Tooltip title={showInfo ? '隐藏 IMEI' : '显示 IMEI'}>
            <IconButton size="small" onClick={() => setShowInfo(!showInfo)}>
              {showInfo ? <VisibilityOff fontSize="small" /> : <Visibility fontSize="small" />}
            </IconButton>
          </Tooltip>
        </Box>
        <Box 
          display="flex" 
          flexWrap="wrap" 
          gap={2}
          sx={{ 
            '& > div': { 
              minWidth: { xs: '45%', sm: '100px' },
              flex: '1 1 auto',
            } 
          }}
        >
          <Box>
            <Typography variant="caption" color="text.secondary">IMEI</Typography>
            <Typography variant="body2" fontFamily="monospace" fontSize="0.75rem" sx={getSensitiveStyle(showInfo)}>
              {deviceInfo?.imei || 'N/A'}
            </Typography>
          </Box>
          <Box>
            <Typography variant="caption" color="text.secondary">制造商</Typography>
            <Typography variant="body2" fontSize="0.75rem">{deviceInfo?.manufacturer || 'N/A'}</Typography>
          </Box>
          <Box>
            <Typography variant="caption" color="text.secondary">型号</Typography>
            <Typography variant="body2" fontSize="0.75rem">{deviceInfo?.model || 'N/A'}</Typography>
          </Box>
          <Box>
            <Typography variant="caption" color="text.secondary">系统</Typography>
            <Typography variant="body2" fontSize="0.75rem">
              {systemStats?.system_info?.sysname || '-'} / {systemStats?.system_info?.machine || '-'}
            </Typography>
          </Box>
          <Box>
            <Typography variant="caption" color="text.secondary">内核</Typography>
            <Typography variant="body2" fontSize="0.75rem" noWrap>
              {systemStats?.system_info?.release || '-'}
            </Typography>
          </Box>
        </Box>
      </CardContent>
    </Card>
  )
}
