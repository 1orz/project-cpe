/*
 * @Author: 1orz cloudorzi@gmail.com
 * @Date: 2025-12-10 10:18:44
 * @LastEditors: 1orz cloudorzi@gmail.com
 * @LastEditTime: 2025-12-13 12:44:38
 * @FilePath: /udx710-backend/frontend/src/pages/Dashboard/components/TemperatureMonitor.tsx
 * @Description: 
 * 
 * Copyright (c) 2025 by 1orz, All Rights Reserved. 
 */
import { Box, Card, CardContent, Typography, Paper } from '@mui/material'
import Grid from '@mui/material/Grid'
import { Thermostat } from '@mui/icons-material'
import { getTempColor } from '../utils'
import type { SystemStatsResponse } from '@/api/types'

interface TemperatureMonitorProps {
  systemStats: SystemStatsResponse | null
}

export function TemperatureMonitor({ systemStats }: TemperatureMonitorProps) {
  return (
    <Card>
      <CardContent>
        <Box display="flex" alignItems="center" gap={1} mb={2}>
          <Thermostat color="primary" />
          <Typography variant="subtitle2" color="text.secondary">温度监控</Typography>
        </Box>
        {systemStats?.temperature && systemStats.temperature.length > 0 ? (
          <Grid container spacing={1}>
            {systemStats.temperature.map((sensor, idx) => (
              <Grid size={{ xs: 6, sm: 4 }} key={idx}>
                <Paper variant="outlined" sx={{ p: 1.5, textAlign: 'center' }}>
                  <Typography variant="caption" color="text.secondary" display="block" noWrap>
                    {sensor.type}
                  </Typography>
                  <Typography
                    variant="h6"
                    fontWeight="bold"
                    color={`${getTempColor(sensor.temperature)}.main`}
                  >
                    {sensor.temperature.toFixed(1)}°
                  </Typography>
                </Paper>
              </Grid>
            ))}
          </Grid>
        ) : (
          <Typography variant="body2" color="text.secondary">暂无数据</Typography>
        )}
      </CardContent>
    </Card>
  )
}
