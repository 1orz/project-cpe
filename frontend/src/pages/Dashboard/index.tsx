/*
 * @Author: 1orz cloudorzi@gmail.com
 * @Date: 2025-12-10 10:22:12
 * @LastEditors: 1orz cloudorzi@gmail.com
 * @LastEditTime: 2025-12-13 12:44:42
 * @FilePath: /udx710-backend/frontend/src/pages/Dashboard/index.tsx
 * @Description: 
 * 
 * Copyright (c) 2025 by 1orz, All Rights Reserved. 
 */
import { Box, CircularProgress } from '@mui/material'
import Grid from '@mui/material/Grid'
import { useRefreshInterval } from '@/contexts/RefreshContext'
import ErrorSnackbar from '@/components/ErrorSnackbar'
import { useDashboardData } from './hooks/useDashboardData'
import {
  StatusOverview,
  QuickControls,
  SystemResources,
  NetworkSpeed,
  ConnectionStatus,
  SimCardInfo,
  TemperatureMonitor,
  CellInfo,
  DeviceInfoCard,
} from './components'

export default function Dashboard() {
  const { refreshInterval, refreshKey } = useRefreshInterval()
  const { initialLoading, error, setError, data, actions } = useDashboardData(refreshInterval, refreshKey)

  if (initialLoading) {
    return (
      <Box display="flex" justifyContent="center" alignItems="center" minHeight="60vh">
        <CircularProgress />
      </Box>
    )
  }

  return (
    <Box>
      <ErrorSnackbar error={error} onClose={() => setError(null)} />

      {/* 顶部状态概览 */}
      <StatusOverview
        deviceInfo={data.deviceInfo}
        networkInfo={data.networkInfo}
        cellsInfo={data.cellsInfo}
        airplaneMode={data.airplaneMode}
        imsStatus={data.imsStatus}
        roaming={data.roaming}
      />

      {/* 主要内容区 - PC端采用更紧凑的多列布局 */}
      <Grid container spacing={2}>
        {/* 第一行: 快捷控制 + 连接状态 + SIM卡 + 系统资源 */}
        <Grid size={{ xs: 12, sm: 6, md: 3 }}>
          <QuickControls
            dataStatus={data.dataStatus}
            airplaneMode={data.airplaneMode}
            roaming={data.roaming}
            onToggleData={() => void actions.toggleData()}
            onToggleAirplaneMode={() => void actions.toggleAirplaneMode()}
            onToggleRoaming={() => void actions.toggleRoaming()}
          />
        </Grid>

        <Grid size={{ xs: 12, sm: 6, md: 3 }}>
          <ConnectionStatus
            qosInfo={data.qosInfo}
            connectivity={data.connectivity}
          />
        </Grid>

        <Grid size={{ xs: 12, sm: 6, md: 3 }}>
          <SimCardInfo simInfo={data.simInfo} />
        </Grid>

        <Grid size={{ xs: 12, sm: 6, md: 3 }}>
          <SystemResources systemStats={data.systemStats} />
        </Grid>

        {/* 第二行: 实时网速 (宽) + 温度监控 */}
        <Grid size={{ xs: 12, md: 8 }}>
          <NetworkSpeed
            systemStats={data.systemStats}
            speedHistory={data.speedHistory}
          />
        </Grid>

        <Grid size={{ xs: 12, md: 4 }}>
          <TemperatureMonitor systemStats={data.systemStats} />
        </Grid>

        {/* 第三行: 小区信息 (全宽) */}
        <Grid size={12}>
          <CellInfo cellsInfo={data.cellsInfo} />
        </Grid>

        {/* 第四行: 设备信息 (全宽) */}
        <Grid size={12}>
          <DeviceInfoCard
            deviceInfo={data.deviceInfo}
            systemStats={data.systemStats}
          />
        </Grid>
      </Grid>
    </Box>
  )
}
