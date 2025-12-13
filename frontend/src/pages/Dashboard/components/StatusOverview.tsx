/*
 * @Author: 1orz cloudorzi@gmail.com
 * @Date: 2025-12-10 10:16:39
 * @LastEditors: 1orz cloudorzi@gmail.com
 * @LastEditTime: 2025-12-13 12:44:31
 * @FilePath: /udx710-backend/frontend/src/pages/Dashboard/components/StatusOverview.tsx
 * @Description: 
 * 
 * Copyright (c) 2025 by 1orz, All Rights Reserved. 
 */
import { Box, Chip, Typography, Paper, alpha, useTheme } from '@mui/material'
import {
  SignalCellularAlt,
  WifiTethering,
  Router,
  PowerSettingsNew,
  FlightTakeoff,
  TravelExplore,
} from '@mui/icons-material'
import { formatCarrierName, getCarrierColor, getCarrierLogo } from '@/utils/carriers'
import { getSignalColor } from '../utils'
import type { DeviceInfo, NetworkInfo, CellsResponse, AirplaneModeResponse, ImsStatusResponse, RoamingResponse } from '@/api/types'

interface StatusOverviewProps {
  deviceInfo: DeviceInfo | null
  networkInfo: NetworkInfo | null
  cellsInfo: CellsResponse | null
  airplaneMode: AirplaneModeResponse | null
  imsStatus: ImsStatusResponse | null
  roaming?: RoamingResponse | null
}

export function StatusOverview({
  deviceInfo,
  networkInfo,
  cellsInfo,
  airplaneMode,
  imsStatus,
  roaming,
}: StatusOverviewProps) {
  const theme = useTheme()

  // 获取网络制式显示
  const getNetworkTech = () => {
    if (cellsInfo?.serving_cell?.tech) {
      return cellsInfo.serving_cell.tech.toUpperCase()
    }
    if (networkInfo?.technology_preference) {
      if (networkInfo.technology_preference.includes('NR')) return '5G'
      if (networkInfo.technology_preference.includes('LTE')) return 'LTE'
    }
    return 'N/A'
  }

  return (
    <Paper
      elevation={0}
      sx={{
        p: 2,
        mb: 2,
        borderRadius: 2,
        background: `linear-gradient(135deg, ${alpha(theme.palette.primary.main, 0.08)} 0%, ${alpha(theme.palette.secondary.main, 0.03)} 100%)`,
        border: `1px solid ${alpha(theme.palette.primary.main, 0.1)}`,
      }}
    >
      <Box display="flex" flexWrap="wrap" alignItems="center" gap={2}>
        {/* 运营商 Logo + 信号 */}
        <Box display="flex" alignItems="center" gap={1.5}>
          {(() => {
            const logo = getCarrierLogo(networkInfo?.mcc, networkInfo?.mnc)
            return logo ? (
              <Box
                component="img"
                src={logo}
                alt={formatCarrierName(networkInfo?.mcc, networkInfo?.mnc)}
                sx={{ height: 32, width: 'auto', objectFit: 'contain' }}
              />
            ) : (
              <Chip
                label={formatCarrierName(networkInfo?.mcc, networkInfo?.mnc)}
                color={getCarrierColor(networkInfo?.mcc, networkInfo?.mnc)}
                size="small"
              />
            )
          })()}
          <Box display="flex" alignItems="center" gap={0.5}>
            <SignalCellularAlt sx={{ fontSize: 24, color: `${getSignalColor(networkInfo?.signal_strength || 0)}.main` }} />
            <Typography variant="h6" fontWeight="bold" color={`${getSignalColor(networkInfo?.signal_strength || 0)}.main`}>
              {networkInfo?.signal_strength || 0}%
            </Typography>
          </Box>
        </Box>

        {/* 网络制式 */}
        <Chip
          icon={<WifiTethering />}
          label={getNetworkTech()}
          color={getNetworkTech() === '5G' || getNetworkTech() === 'NR' ? 'success' : 'primary'}
          size="small"
          sx={{ fontWeight: 'bold' }}
        />

        {/* 注册状态 */}
        <Chip
          icon={<Router />}
          label={
            networkInfo?.registration_status === 'registered' ? '已注册' : 
            networkInfo?.registration_status === 'roaming' ? '漫游' :
            networkInfo?.registration_status || '未知'
          }
          color={
            networkInfo?.registration_status === 'registered' ? 'success' : 
            networkInfo?.registration_status === 'roaming' ? 'warning' :
            'default'
          }
          variant="outlined"
          size="small"
        />

        {/* 漫游状态 */}
        {roaming?.is_roaming && (
          <Chip
            icon={<TravelExplore />}
            label={roaming.roaming_allowed ? '漫游数据已开启' : '漫游数据已关闭'}
            color={roaming.roaming_allowed ? 'info' : 'error'}
            size="small"
          />
        )}

        {/* Modem 状态 */}
        <Chip
          icon={<PowerSettingsNew />}
          label={deviceInfo?.online ? '在线' : '离线'}
          color={deviceInfo?.online ? 'success' : 'error'}
          size="small"
        />

        {/* VoLTE */}
        {imsStatus?.registered && (
          <Chip label="VoLTE" color="info" size="small" variant="outlined" />
        )}

        {/* 飞行模式 */}
        {airplaneMode?.enabled && (
          <Chip icon={<FlightTakeoff />} label="飞行模式" color="warning" size="small" />
        )}
      </Box>
    </Paper>
  )
}
