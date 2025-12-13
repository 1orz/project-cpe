/*
 * @Author: 1orz cloudorzi@gmail.com
 * @Date: 2025-12-10 10:17:25
 * @LastEditors: 1orz cloudorzi@gmail.com
 * @LastEditTime: 2025-12-13 12:44:35
 * @FilePath: /udx710-backend/frontend/src/pages/Dashboard/components/SystemResources.tsx
 * @Description: 
 * 
 * Copyright (c) 2025 by 1orz, All Rights Reserved. 
 */
import { Box, Card, CardContent, Typography, Stack, LinearProgress, Chip, Tooltip } from '@mui/material'
import { Speed, Memory, Storage, Thermostat, Usb, Info } from '@mui/icons-material'
import { formatBytes, getCpuColor, getMemoryColor, getTempColor } from '../utils'
import type { SystemStatsResponse } from '@/api/types'

interface SystemResourcesProps {
  systemStats: SystemStatsResponse | null
}

export function SystemResources({ systemStats }: SystemResourcesProps) {
  // 获取主温度
  const getMainTemp = () => {
    if (systemStats?.temperature && systemStats.temperature.length > 0) {
      const socSensor = systemStats.temperature.find(s => s.type.includes('soc'))
      return socSensor?.temperature || systemStats.temperature[0].temperature
    }
    return null
  }

  const mainTemp = getMainTemp()

  return (
    <Card sx={{ height: '100%' }}>
      <CardContent>
        <Typography variant="subtitle2" color="text.secondary" gutterBottom>
          系统资源
        </Typography>
        <Stack spacing={1.5}>
          {/* CPU 负载 */}
          <Box>
            <Box display="flex" justifyContent="space-between" alignItems="center" mb={0.5}>
              <Box display="flex" alignItems="center" gap={0.5}>
                <Speed fontSize="small" color="action" />
                <Typography variant="caption" color="text.secondary">
                  CPU ({systemStats?.cpu_load?.core_count || '-'}核)
                </Typography>
              </Box>
              <Typography variant="caption" fontWeight="medium">
                {systemStats?.cpu_load ? `${systemStats.cpu_load.load_percent.toFixed(0)}%` : '-'}
              </Typography>
            </Box>
            <LinearProgress
              variant="determinate"
              value={systemStats?.cpu_load?.load_percent || 0}
              color={getCpuColor(systemStats?.cpu_load?.load_percent || 0)}
              sx={{ height: 4, borderRadius: 2 }}
            />
            <Typography variant="caption" color="text.secondary" sx={{ fontSize: '0.65rem' }}>
              负载: {systemStats?.cpu_load?.load_1min.toFixed(2) || '-'} / {systemStats?.cpu_load?.load_5min.toFixed(2) || '-'} / {systemStats?.cpu_load?.load_15min.toFixed(2) || '-'}
            </Typography>
          </Box>

          {/* 内存 */}
          <Box>
            <Box display="flex" justifyContent="space-between" alignItems="center" mb={0.5}>
              <Box display="flex" alignItems="center" gap={0.5}>
                <Memory fontSize="small" color="action" />
                <Typography variant="caption" color="text.secondary">内存</Typography>
                {systemStats?.memory && (
                  <Typography variant="caption" color="text.secondary" sx={{ fontSize: '0.65rem', ml: 0.5 }}>
                    已用 {formatBytes(systemStats.memory.used_bytes)} / 可用 {formatBytes(systemStats.memory.available_bytes)} / 缓存 {formatBytes(systemStats.memory.cached_bytes)}
                  </Typography>
                )}
              </Box>
              <Typography variant="caption" fontWeight="medium">
                {systemStats?.memory ? `${systemStats.memory.used_percent.toFixed(0)}%` : '-'}
              </Typography>
            </Box>
            <LinearProgress
              variant="determinate"
              value={systemStats?.memory?.used_percent || 0}
              color={getMemoryColor(systemStats?.memory?.used_percent || 0)}
              sx={{ height: 4, borderRadius: 2 }}
            />
          </Box>

          {/* 磁盘空间 */}
          {systemStats?.disk && systemStats.disk.length > 0 && (
            <Box>
              <Box display="flex" alignItems="center" gap={0.5} mb={0.5}>
                <Storage fontSize="small" color="action" />
                <Typography variant="caption" color="text.secondary">磁盘</Typography>
              </Box>
              {systemStats.disk.map((disk, idx) => (
                <Box key={idx} sx={{ mb: idx < systemStats.disk.length - 1 ? 0.5 : 0 }}>
                  <Box display="flex" justifyContent="space-between" alignItems="center" mb={0.25}>
                    <Typography variant="caption" color="text.secondary" sx={{ fontSize: '0.65rem' }}>
                      {disk.mount_point}
                    </Typography>
                    <Typography variant="caption" sx={{ fontSize: '0.65rem' }}>
                      {formatBytes(disk.used_bytes)} / {formatBytes(disk.total_bytes)} ({disk.used_percent.toFixed(0)}%)
                    </Typography>
                  </Box>
                  <LinearProgress
                    variant="determinate"
                    value={disk.used_percent}
                    color={getMemoryColor(disk.used_percent)}
                    sx={{ height: 3, borderRadius: 1.5 }}
                  />
                </Box>
              ))}
            </Box>
          )}

          {/* 温度 */}
          <Box display="flex" justifyContent="space-between" alignItems="center">
            <Box display="flex" alignItems="center" gap={0.5}>
              <Thermostat fontSize="small" color="action" />
              <Typography variant="caption" color="text.secondary">温度</Typography>
            </Box>
            {mainTemp !== null ? (
              <Chip
                label={`${mainTemp.toFixed(0)}°C`}
                size="small"
                color={getTempColor(mainTemp)}
              />
            ) : (
              <Typography variant="caption">-</Typography>
            )}
          </Box>

          {/* 运行时间 */}
          <Box display="flex" justifyContent="space-between" alignItems="center">
            <Typography variant="caption" color="text.secondary">运行时间</Typography>
            <Typography variant="caption" fontWeight="medium">
              {systemStats?.uptime?.uptime_formatted || '-'}
            </Typography>
          </Box>

          {/* USB 模式 */}
          <Box display="flex" justifyContent="space-between" alignItems="center">
            <Box display="flex" alignItems="center" gap={0.5}>
              <Usb fontSize="small" color="action" />
              <Typography variant="caption" color="text.secondary">USB 模式</Typography>
            </Box>
            <Box display="flex" alignItems="center" gap={0.5}>
              <Chip
                label={systemStats?.usb_mode?.current_mode_name || 'N/A'}
                size="small"
                color="primary"
                variant="outlined"
              />
              {systemStats?.usb_mode?.needs_reboot && (
                <Tooltip title="需要重启生效">
                  <Info fontSize="small" color="warning" />
                </Tooltip>
              )}
            </Box>
          </Box>
        </Stack>
      </CardContent>
    </Card>
  )
}
