/*
 * @Author: 1orz cloudorzi@gmail.com
 * @Date: 2025-12-10 10:18:32
 * @LastEditors: 1orz cloudorzi@gmail.com
 * @LastEditTime: 2025-12-13 12:44:16
 * @FilePath: /udx710-backend/frontend/src/pages/Dashboard/components/ConnectionStatus.tsx
 * @Description: 
 * 
 * Copyright (c) 2025 by 1orz, All Rights Reserved. 
 */
import { Box, Card, CardContent, Typography, Stack } from '@mui/material'
import type { QosInfo } from '@/api/types'
import type { ConnectivityResult } from '../hooks/useDashboardData'

interface ConnectionStatusProps {
  qosInfo: QosInfo | null
  connectivity: ConnectivityResult | null
}

export function ConnectionStatus({ qosInfo, connectivity }: ConnectionStatusProps) {
  return (
    <Card sx={{ height: '100%' }}>
      <CardContent>
        <Typography variant="subtitle2" color="text.secondary" gutterBottom>
          连接状态
        </Typography>
        <Stack spacing={1}>
          <Box display="flex" justifyContent="space-between" alignItems="center">
            <Typography variant="caption" color="text.secondary">QCI</Typography>
            <Typography variant="body2" fontWeight="medium">{qosInfo?.qci || '-'}</Typography>
          </Box>
          <Box display="flex" justifyContent="space-between" alignItems="center">
            <Typography variant="caption" color="text.secondary">下行</Typography>
            <Typography variant="body2" fontWeight="medium">
              {qosInfo?.dl_speed ? `${(qosInfo.dl_speed / 1000).toFixed(0)} Mbps` : '-'}
            </Typography>
          </Box>
          <Box display="flex" justifyContent="space-between" alignItems="center">
            <Typography variant="caption" color="text.secondary">上行</Typography>
            <Typography variant="body2" fontWeight="medium">
              {qosInfo?.ul_speed ? `${(qosInfo.ul_speed / 1000).toFixed(0)} Mbps` : '-'}
            </Typography>
          </Box>
          {/* 联网检测 */}
          <Box display="flex" justifyContent="space-between" alignItems="center" pt={0.5} borderTop={1} borderColor="divider">
            <Box display="flex" alignItems="center" gap={0.5}>
              <Box sx={{ width: 6, height: 6, borderRadius: '50%', bgcolor: connectivity?.ipv4?.success ? 'success.main' : 'error.main' }} />
              <Typography variant="caption" color="text.secondary">IPv4</Typography>
              <Typography variant="caption" fontWeight="medium" color={connectivity?.ipv4?.success ? 'success.main' : 'error.main'}>
                {connectivity?.ipv4?.success ? `${connectivity.ipv4.latency_ms?.toFixed(0)}ms` : 'x'}
              </Typography>
            </Box>
            <Box display="flex" alignItems="center" gap={0.5}>
              <Box sx={{ width: 6, height: 6, borderRadius: '50%', bgcolor: connectivity?.ipv6?.success ? 'success.main' : 'error.main' }} />
              <Typography variant="caption" color="text.secondary">IPv6</Typography>
              <Typography variant="caption" fontWeight="medium" color={connectivity?.ipv6?.success ? 'success.main' : 'error.main'}>
                {connectivity?.ipv6?.success ? `${connectivity.ipv6.latency_ms?.toFixed(0)}ms` : 'x'}
              </Typography>
            </Box>
          </Box>
        </Stack>
      </CardContent>
    </Card>
  )
}
