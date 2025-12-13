/*
 * @Author: 1orz cloudorzi@gmail.com
 * @Date: 2025-12-10 09:19:05
 * @LastEditors: 1orz cloudorzi@gmail.com
 * @LastEditTime: 2025-12-13 12:44:54
 * @FilePath: /udx710-backend/frontend/src/pages/DeviceInfo.tsx
 * @Description: 
 * 
 * Copyright (c) 2025 by 1orz, All Rights Reserved. 
 */
import { useEffect, useState } from 'react'
import {
  Box,
  Typography,
  Card,
  CardContent,
  CardHeader,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableRow,
  Chip,
  CircularProgress,
  Button,
  Tooltip,
  Snackbar,
  Alert,
  IconButton,
} from '@mui/material'
import {
  PhoneAndroid,
  Tag,
  SimCard,
  Visibility,
  VisibilityOff,
  SwapHoriz,
} from '@mui/icons-material'
import Grid from '@mui/material/Grid'
import { api } from '../api'
import ErrorSnackbar from '../components/ErrorSnackbar'
import type { DeviceInfo, SimInfo, SimSlotResponse, ImeisvResponse } from '../api/types'

export default function DeviceInfoPage() {
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [success, setSuccess] = useState<string | null>(null)
  // 每个功能块独立的敏感信息显示状态
  const [showDeviceId, setShowDeviceId] = useState(false)
  const [showSimInfo, setShowSimInfo] = useState(false)
  
  // 设备信息（包含 online, powered, manufacturer, model）
  const [deviceInfo, setDeviceInfo] = useState<DeviceInfo | null>(null)
  // SIM 信息（包含所有 SIM 相关数据）
  const [simInfo, setSimInfo] = useState<SimInfo | null>(null)
  
  // 扩展状态
  const [imeisv, setImeisv] = useState<ImeisvResponse | null>(null)
  const [simSlot, setSimSlot] = useState<SimSlotResponse | null>(null)
  const [switchingSlot, setSwitchingSlot] = useState(false)

  const loadData = async () => {
    setLoading(true)
    setError(null)
    
    try {
      const [deviceRes, simRes] = await Promise.all([
        api.getDeviceInfo(),
        api.getSimInfo(),
      ])
      
      if (deviceRes.data) setDeviceInfo(deviceRes.data)
      if (simRes.data) setSimInfo(simRes.data)

      // 加载扩展数据
      try {
        const [imeisvRes, simSlotRes] = await Promise.all([
          api.getImeisv(),
          api.getSimSlot(),
        ])
        if (imeisvRes.data) setImeisv(imeisvRes.data)
        if (simSlotRes.data) setSimSlot(simSlotRes.data)
      } catch (extErr) {
        console.warn('部分扩展信息加载失败:', extErr)
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setLoading(false)
    }
  }

  const handleSwitchSimSlot = () => {
    void switchSimSlot()
  }

  const switchSimSlot = async () => {
    if (!simSlot) return
    const targetSlot = simSlot.active_slot === 1 ? 2 : 1
    setSwitchingSlot(true)
    try {
      const res = await api.switchSimSlot(targetSlot)
      if (res.status === 'ok') {
        setSuccess(`正在切换到卡槽 ${targetSlot}...`)
        setTimeout(loadData, 2000)
      } else {
        setError(res.message || '切换失败')
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setSwitchingSlot(false)
    }
  }

  // 根据不同功能块返回对应的敏感信息样式
  const getSensitiveStyle = (show: boolean) => ({
    filter: show ? 'none' : 'blur(5px)',
    transition: 'filter 0.3s ease',
    userSelect: show ? 'auto' : 'none',
    cursor: show ? 'text' : 'default',
  })

  useEffect(() => {
    void loadData()
  }, [])

  if (loading) {
    return (
      <Box display="flex" justifyContent="center" alignItems="center" minHeight="60vh">
        <CircularProgress />
      </Box>
    )
  }

  return (
    <Box>
      {/* 页面标题 */}
      <Box mb={3}>
        <Typography variant="h4" gutterBottom fontWeight={600}>
          设备信息
        </Typography>
        <Typography variant="body2" color="text.secondary">
          查看设备详细参数和配置
        </Typography>
      </Box>
      {/* 错误和成功提示 Snackbar */}
      <ErrorSnackbar error={error} onClose={() => setError(null)} />
      {success && (
        <Snackbar
          open={true}
          autoHideDuration={3000}
          onClose={() => setSuccess(null)}
          anchorOrigin={{ vertical: 'top', horizontal: 'center' }}
        >
          <Alert severity="success" variant="filled" onClose={() => setSuccess(null)}>
            {success}
          </Alert>
        </Snackbar>
      )}
      <Grid container spacing={3}>
        {/* Modem 基础信息 */}
        <Grid size={{ xs: 12, md: 6 }}>
          <Card>
            <CardHeader
              avatar={<PhoneAndroid color="primary" />}
              title="设备状态"
              titleTypographyProps={{ variant: 'h6' }}
            />
            <CardContent>
              <TableContainer>
                <Table size="small">
                  <TableBody>
                    <TableRow>
                      <TableCell component="th" width="40%">在线状态</TableCell>
                      <TableCell>
                        <Chip
                          label={deviceInfo?.online ? '在线' : '离线'}
                          color={deviceInfo?.online ? 'success' : 'error'}
                          size="small"
                        />
                      </TableCell>
                    </TableRow>
                    <TableRow>
                      <TableCell component="th">制造商</TableCell>
                      <TableCell>{deviceInfo?.manufacturer || 'N/A'}</TableCell>
                    </TableRow>
                    <TableRow>
                      <TableCell component="th">型号</TableCell>
                      <TableCell>{deviceInfo?.model || 'N/A'}</TableCell>
                    </TableRow>
                    <TableRow>
                      <TableCell component="th">固件版本</TableCell>
                      <TableCell>{deviceInfo?.revision || 'N/A'}</TableCell>
                    </TableRow>
                    <TableRow>
                      <TableCell component="th">电源状态</TableCell>
                      <TableCell>
                        <Chip
                          label={deviceInfo?.powered ? '已开启' : '已关闭'}
                          color={deviceInfo?.powered ? 'success' : 'default'}
                          size="small"
                        />
                      </TableCell>
                    </TableRow>
                  </TableBody>
                </Table>
              </TableContainer>
            </CardContent>
          </Card>
        </Grid>

        {/* 设备标识信息 */}
        <Grid size={{ xs: 12, md: 6 }}>
          <Card>
            <CardHeader
              avatar={<Tag color="primary" />}
              title="设备标识"
              titleTypographyProps={{ variant: 'h6' }}
              action={
                <Tooltip title={showDeviceId ? '隐藏敏感信息' : '显示完整信息'}>
                  <IconButton
                    size="small"
                    onClick={() => setShowDeviceId(!showDeviceId)}
                    color="primary"
                  >
                    {showDeviceId ? <VisibilityOff /> : <Visibility />}
                  </IconButton>
                </Tooltip>
              }
            />
            <CardContent>
              <TableContainer>
                <Table size="small">
                  <TableBody>
                    <TableRow>
                      <TableCell component="th" width="40%">IMEI</TableCell>
                      <TableCell 
                        sx={{ 
                          fontFamily: 'monospace', 
                          fontSize: '0.9rem',
                          ...getSensitiveStyle(showDeviceId)
                        }}
                      >
                        {deviceInfo?.imei || 'N/A'}
                      </TableCell>
                    </TableRow>
                    <TableRow>
                      <TableCell component="th">IMEISV (软件版本)</TableCell>
                      <TableCell sx={{ fontFamily: 'monospace', fontSize: '0.9rem' }}>
                        {imeisv?.software_version_number || 'N/A'}
                      </TableCell>
                    </TableRow>
                    <TableRow>
                      <TableCell component="th">ICCID</TableCell>
                      <TableCell 
                        sx={{ 
                          fontFamily: 'monospace', 
                          fontSize: '0.9rem',
                          ...getSensitiveStyle(showDeviceId)
                        }}
                      >
                        {simInfo?.iccid || 'N/A'}
                      </TableCell>
                    </TableRow>
                    <TableRow>
                      <TableCell component="th">IMSI</TableCell>
                      <TableCell 
                        sx={{ 
                          fontFamily: 'monospace', 
                          fontSize: '0.9rem',
                          ...getSensitiveStyle(showDeviceId)
                        }}
                      >
                        {simInfo?.imsi || 'N/A'}
                      </TableCell>
                    </TableRow>
                  </TableBody>
                </Table>
              </TableContainer>
            </CardContent>
          </Card>
        </Grid>

        {/* SIM 卡完整信息 */}
        <Grid size={{ xs: 12, md: 6 }}>
          <Card>
            <CardHeader
              avatar={<SimCard color="primary" />}
              title="SIM 卡信息"
              titleTypographyProps={{ variant: 'h6' }}
              action={
                <Tooltip title={showSimInfo ? '隐藏敏感信息' : '显示完整信息'}>
                  <IconButton
                    size="small"
                    onClick={() => setShowSimInfo(!showSimInfo)}
                    color="primary"
                  >
                    {showSimInfo ? <VisibilityOff /> : <Visibility />}
                  </IconButton>
                </Tooltip>
              }
            />
            <CardContent>
              <TableContainer>
                <Table size="small">
                  <TableBody>
                    <TableRow>
                      <TableCell component="th" width="40%">SIM 卡状态</TableCell>
                      <TableCell>
                        <Chip
                          label={simInfo?.present ? '已插入' : '未插入'}
                          color={simInfo?.present ? 'success' : 'error'}
                          size="small"
                        />
                      </TableCell>
                    </TableRow>
                    <TableRow>
                      <TableCell component="th">PIN 状态</TableCell>
                      <TableCell>{simInfo?.pin_required || 'N/A'}</TableCell>
                    </TableRow>
                    <TableRow>
                      <TableCell component="th">手机号码</TableCell>
                      <TableCell 
                        sx={{ 
                          fontFamily: 'monospace', 
                          fontSize: '0.9rem',
                          ...getSensitiveStyle(showSimInfo)
                        }}
                      >
                        {simInfo?.phone_numbers?.join(', ') || 'N/A'}
                      </TableCell>
                    </TableRow>
                    <TableRow>
                      <TableCell component="th">MCC / MNC</TableCell>
                      <TableCell sx={{ fontFamily: 'monospace', fontSize: '0.9rem' }}>
                        {simInfo?.mcc || 'N/A'} / {simInfo?.mnc || 'N/A'}
                      </TableCell>
                    </TableRow>
                    <TableRow>
                      <TableCell component="th">短信中心号码</TableCell>
                      <TableCell 
                        sx={{ 
                          fontFamily: 'monospace', 
                          fontSize: '0.9rem',
                          ...getSensitiveStyle(showSimInfo)
                        }}
                      >
                        {simInfo?.sms_center || 'N/A'}
                      </TableCell>
                    </TableRow>
                    <TableRow>
                      <TableCell component="th">首选语言</TableCell>
                      <TableCell>
                        {simInfo?.preferred_languages?.map((lang: string) => (
                          <Chip key={lang} label={lang.toUpperCase()} size="small" sx={{ mr: 0.5 }} />
                        )) || 'N/A'}
                      </TableCell>
                    </TableRow>
                  </TableBody>
                </Table>
              </TableContainer>
            </CardContent>
          </Card>
        </Grid>

        {/* SIM 卡槽管理 */}
        <Grid size={{ xs: 12, md: 6 }}>
          <Card>
            <CardHeader
              avatar={<SwapHoriz color="primary" />}
              title="SIM 卡槽"
              titleTypographyProps={{ variant: 'h6' }}
            />
            <CardContent>
              <Box display="flex" alignItems="center" justifyContent="space-between" mb={2}>
                <Box>
                  <Typography variant="body1">
                    当前卡槽: <Chip 
                      label={simSlot?.active_slot ? `卡槽 ${simSlot.active_slot}` : '未知'} 
                      color="primary" 
                      size="small" 
                    />
                  </Typography>
                  {simSlot?.raw_value && (
                    <Typography variant="caption" color="text.secondary">
                      原始值: {simSlot.raw_value}
                    </Typography>
                  )}
                </Box>
                <Button
                  variant="outlined"
                  startIcon={<SwapHoriz />}
                  onClick={() => handleSwitchSimSlot()}
                  disabled={switchingSlot || !simSlot}
                >
                  {switchingSlot ? <CircularProgress size={20} /> : `切换到卡槽 ${simSlot?.active_slot === 1 ? 2 : 1}`}
                </Button>
              </Box>
              <Alert severity="info" variant="outlined">
                切换 SIM 卡槽后，设备可能需要重新注册网络。
              </Alert>
            </CardContent>
          </Card>
        </Grid>

      </Grid>
    </Box>
  );
}
