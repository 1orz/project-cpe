/*
 * @Author: 1orz cloudorzi@gmail.com
 * @Date: 2025-12-10 10:17:51
 * @LastEditors: 1orz cloudorzi@gmail.com
 * @LastEditTime: 2025-12-13 12:44:25
 * @FilePath: /udx710-backend/frontend/src/pages/Dashboard/components/NetworkSpeed.tsx
 * @Description: 
 * 
 * Copyright (c) 2025 by 1orz, All Rights Reserved. 
 */
import { Box, Card, CardContent, Typography, Stack, Chip, Paper, alpha, useTheme } from '@mui/material'
import { Speed, ArrowDownward, ArrowUpward } from '@mui/icons-material'
import { SparkLineChart } from '@mui/x-charts/SparkLineChart'
import { formatBytes, formatSpeed } from '../utils'
import { SPEED_HISTORY_MAX_POINTS, type InterfaceSpeedHistory } from '../hooks/useDashboardData'
import type { SystemStatsResponse } from '@/api/types'

interface NetworkSpeedProps {
  systemStats: SystemStatsResponse | null
  speedHistory: Record<string, InterfaceSpeedHistory>
}

export function NetworkSpeed({ systemStats, speedHistory }: NetworkSpeedProps) {
  const theme = useTheme()

  return (
    <Card sx={{ height: '100%', overflow: 'hidden' }}>
      <CardContent>
        <Box display="flex" alignItems="center" gap={1} mb={2}>
          <Speed color="primary" />
          <Typography variant="subtitle2" color="text.secondary">实时网速</Typography>
          <Typography variant="caption" color="text.disabled" sx={{ ml: 'auto' }}>
            {SPEED_HISTORY_MAX_POINTS}s 趋势
          </Typography>
        </Box>
        {systemStats?.network_speed?.interfaces && systemStats.network_speed.interfaces.length > 0 ? (
          <Stack spacing={2}>
            {systemStats.network_speed.interfaces.map((iface) => {
              const history = speedHistory[iface.interface]
              const rxData = history?.rx || []
              const txData = history?.tx || []
              const maxSpeed = Math.max(
                Math.max(...rxData, 1),
                Math.max(...txData, 1)
              )
              
              return (
                <Paper 
                  key={iface.interface} 
                  variant="outlined" 
                  sx={{ 
                    p: 2, 
                    overflow: 'hidden',
                    background: alpha(theme.palette.background.paper, 0.6),
                  }}
                >
                  {/* 接口名称和总流量 */}
                  <Box display="flex" alignItems="center" justifyContent="space-between" mb={1.5}>
                    <Chip 
                      label={iface.interface} 
                      size="small" 
                      variant="outlined"
                      sx={{ fontFamily: 'monospace', fontWeight: 500 }}
                    />
                    <Typography variant="caption" color="text.secondary">
                      总流量: ↓ {formatBytes(iface.total_rx_bytes)} / ↑ {formatBytes(iface.total_tx_bytes)}
                    </Typography>
                  </Box>
                  
                  {/* 下载速度 */}
                  <Box mb={1.5}>
                    <Box display="flex" alignItems="center" justifyContent="space-between" mb={0.5}>
                      <Box display="flex" alignItems="center" gap={0.5}>
                        <ArrowDownward fontSize="small" sx={{ color: theme.palette.success.main }} />
                        <Typography variant="caption" color="text.secondary">下载</Typography>
                      </Box>
                      <Typography 
                        variant="body1" 
                        fontWeight="bold" 
                        sx={{ 
                          color: theme.palette.success.main,
                          fontFamily: 'monospace',
                          minWidth: 90,
                          textAlign: 'right',
                        }}
                      >
                        {formatSpeed(iface.rx_bytes_per_sec)}
                      </Typography>
                    </Box>
                    {rxData.length > 1 && (
                      <Box sx={{ height: 40, width: '100%' }}>
                        <SparkLineChart
                          data={rxData}
                          height={40}
                          area
                          curve="natural"
                          color={theme.palette.success.main}
                          yAxis={{ min: 0, max: maxSpeed * 1.1 }}
                          margin={{ top: 2, bottom: 2, left: 0, right: 0 }}
                        />
                      </Box>
                    )}
                  </Box>
                  
                  {/* 上传速度 */}
                  <Box>
                    <Box display="flex" alignItems="center" justifyContent="space-between" mb={0.5}>
                      <Box display="flex" alignItems="center" gap={0.5}>
                        <ArrowUpward fontSize="small" sx={{ color: theme.palette.primary.main }} />
                        <Typography variant="caption" color="text.secondary">上传</Typography>
                      </Box>
                      <Typography 
                        variant="body1" 
                        fontWeight="bold" 
                        sx={{ 
                          color: theme.palette.primary.main,
                          fontFamily: 'monospace',
                          minWidth: 90,
                          textAlign: 'right',
                        }}
                      >
                        {formatSpeed(iface.tx_bytes_per_sec)}
                      </Typography>
                    </Box>
                    {txData.length > 1 && (
                      <Box sx={{ height: 40, width: '100%' }}>
                        <SparkLineChart
                          data={txData}
                          height={40}
                          area
                          curve="natural"
                          color={theme.palette.primary.main}
                          yAxis={{ min: 0, max: maxSpeed * 1.1 }}
                          margin={{ top: 2, bottom: 2, left: 0, right: 0 }}
                        />
                      </Box>
                    )}
                  </Box>
                </Paper>
              )
            })}
          </Stack>
        ) : (
          <Typography variant="body2" color="text.secondary">暂无数据</Typography>
        )}
      </CardContent>
    </Card>
  )
}
