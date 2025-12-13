/*
 * @Author: 1orz cloudorzi@gmail.com
 * @Date: 2025-12-03 13:58:02
 * @LastEditors: 1orz cloudorzi@gmail.com
 * @LastEditTime: 2025-12-13 12:44:59
 * @FilePath: /udx710-backend/frontend/src/pages/Phone.tsx
 * @Description: 
 * 
 * Copyright (c) 2025 by 1orz, All Rights Reserved. 
 */
import { useState, useEffect, useCallback } from 'react'
import {
  Box,
  Card,
  CardContent,
  Typography,
  Button,
  TextField,
  List,
  ListItem,
  ListItemText,
  ListItemAvatar,
  ListItemSecondaryAction,
  IconButton,
  Chip,
  Alert,
  CircularProgress,
  Slider,
  Switch,
  FormControlLabel,
  Divider,
  Accordion,
  AccordionSummary,
  AccordionDetails,
  Select,
  MenuItem,
  FormControl,
  InputLabel,
  Snackbar,
  Tabs,
  Tab,
  Avatar,
  Paper,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Fade,
} from '@mui/material'
import Grid from '@mui/material/Grid'
import {
  Phone as PhoneIcon,
  CallEnd,
  Refresh,
  PhoneCallback,
  VolumeUp,
  Mic,
  MicOff,
  ExpandMore,
  PhoneForwarded,
  Settings,
  CallReceived,
  PhoneMissed,
  PhoneDisabled,
  Dialpad,
  History,
  Delete,
  DeleteSweep,
  CallMade,
  Call,
  Backspace,
} from '@mui/icons-material'
import { api, type CallInfo, type CallVolumeResponse, type CallForwardingResponse, type CallSettingsResponse, type CallRecord, type CallStats } from '../api'

// 拨号盘按键
const dialpadButtons = [
  ['1', '2', '3'],
  ['4', '5', '6'],
  ['7', '8', '9'],
  ['*', '0', '#'],
]

export default function PhonePage() {
  const [tabValue, setTabValue] = useState(0)
  const [calls, setCalls] = useState<CallInfo[]>([])
  const [_loading, setLoading] = useState(false)
  const [dialNumber, setDialNumber] = useState('')
  const [error, setError] = useState<string | null>(null)
  const [success, setSuccess] = useState<string | null>(null)
  const [dialLoading, setDialLoading] = useState(false)

  // 通话记录状态
  const [callHistory, setCallHistory] = useState<CallRecord[]>([])
  const [callStats, setCallStats] = useState<CallStats | null>(null)
  const [historyLoading, setHistoryLoading] = useState(false)
  const [clearDialogOpen, setClearDialogOpen] = useState(false)

  // 音量控制状态
  const [_volume, setVolume] = useState<CallVolumeResponse | null>(null)
  const [speakerVolume, setSpeakerVolume] = useState(50)
  const [micVolume, setMicVolume] = useState(50)
  const [muted, setMuted] = useState(false)
  const [volumeLoading, setVolumeLoading] = useState(false)

  // 呼叫转移状态
  const [forwarding, setForwarding] = useState<CallForwardingResponse | null>(null)
  const [forwardingLoading, setForwardingLoading] = useState(false)
  const [forwardType, setForwardType] = useState('unconditional')
  const [forwardNumber, setForwardNumber] = useState('')
  const [forwardTimeout, setForwardTimeout] = useState(20)

  // 通话设置状态
  const [callSettings, setCallSettings] = useState<CallSettingsResponse | null>(null)
  const [settingsLoading, setSettingsLoading] = useState(false)

  // 获取通话列表
  const fetchCalls = useCallback(async () => {
    setLoading(true)
    try {
      const response = await api.getCalls()
      if (response.status === 'ok' && response.data) {
        setCalls(response.data.calls)
      }
    } catch (err) {
      console.error('获取通话列表失败:', err)
    } finally {
      setLoading(false)
    }
  }, [])

  // 获取通话记录
  const fetchCallHistory = useCallback(async () => {
    setHistoryLoading(true)
    try {
      const response = await api.getCallHistory(100, 0)
      if (response.status === 'ok' && response.data) {
        setCallHistory(response.data.records)
        setCallStats(response.data.stats)
      }
    } catch (err) {
      console.error('获取通话记录失败:', err)
    } finally {
      setHistoryLoading(false)
    }
  }, [])

  // 获取音量设置
  const fetchVolume = async () => {
    try {
      const response = await api.getCallVolume()
      if (response.status === 'ok' && response.data) {
        setVolume(response.data)
        setSpeakerVolume(response.data.speaker_volume)
        setMicVolume(response.data.microphone_volume)
        setMuted(response.data.muted)
      }
    } catch (err) {
      console.warn('获取音量设置失败:', err)
    }
  }

  // 获取呼叫转移设置
  const fetchForwarding = async () => {
    setForwardingLoading(true)
    try {
      const response = await api.getCallForwarding()
      if (response.status === 'ok' && response.data) {
        setForwarding(response.data)
      }
    } catch (err) {
      console.warn('获取呼叫转移设置失败:', err)
    } finally {
      setForwardingLoading(false)
    }
  }

  // 获取通话设置
  const fetchCallSettings = async () => {
    setSettingsLoading(true)
    try {
      const response = await api.getCallSettings()
      if (response.status === 'ok' && response.data) {
        setCallSettings(response.data)
      }
    } catch (err) {
      console.warn('获取通话设置失败:', err)
    } finally {
      setSettingsLoading(false)
    }
  }

  useEffect(() => {
    void fetchCalls()
    void fetchVolume()
    void fetchCallHistory()
    
    // 每3秒自动刷新通话列表
    const interval = setInterval(() => {
      void fetchCalls()
    }, 3000)
    return () => clearInterval(interval)
  }, [fetchCalls, fetchCallHistory])

  // 拨号盘按键点击
  const handleDialpadPress = (digit: string) => {
    setDialNumber((prev) => prev + digit)
  }

  // 删除最后一个字符
  const handleBackspace = () => {
    setDialNumber((prev) => prev.slice(0, -1))
  }

  // 拨打电话
  const handleDial = async () => {
    if (!dialNumber.trim()) {
      setError('请输入电话号码')
      return
    }

    setDialLoading(true)
    setError(null)
    setSuccess(null)

    try {
      const response = await api.dialCall(dialNumber)
      if (response.status === 'ok') {
        setSuccess(`正在拨打 ${dialNumber}`)
        setDialNumber('')
        void fetchCalls()
      } else {
        setError(response.message)
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setDialLoading(false)
    }
  }

  // 挂断所有通话
  const handleHangupAll = async () => {
    setError(null)
    setSuccess(null)

    try {
      const response = await api.hangupAllCalls()
      if (response.status === 'ok') {
        setSuccess(`已挂断所有通话`)
        void fetchCalls()
      } else {
        setError(response.message)
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    }
  }

  // 接听来电
  const handleAnswer = async (path: string, phoneNumber: string) => {
    setError(null)
    setSuccess(null)

    try {
      const response = await api.answerCall(path)
      if (response.status === 'ok') {
        setSuccess(`已接听 ${phoneNumber}`)
        void fetchCalls()
      } else {
        setError(response.message)
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    }
  }

  // 从通话记录拨打
  const handleDialFromHistory = (phoneNumber: string) => {
    setDialNumber(phoneNumber)
    setTabValue(0)
  }

  // 删除单条通话记录
  const handleDeleteRecord = async (id: number) => {
    try {
      const response = await api.deleteCallRecord(id)
      if (response.status === 'ok') {
        setSuccess('记录已删除')
        void fetchCallHistory()
      } else {
        setError(response.message)
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    }
  }

  // 清空所有通话记录
  const handleClearHistory = async () => {
    setClearDialogOpen(false)
    try {
      const response = await api.clearCallHistory()
      if (response.status === 'ok') {
        setSuccess('所有通话记录已清空')
        void fetchCallHistory()
      } else {
        setError(response.message)
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    }
  }

  // 保存音量设置
  const saveVolume = async () => {
    setVolumeLoading(true)
    try {
      const response = await api.setCallVolume({
        speaker_volume: speakerVolume,
        microphone_volume: micVolume,
        muted,
      })
      if (response.status === 'ok') {
        setSuccess('音量设置已保存')
        void fetchVolume()
      } else {
        setError(response.message)
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setVolumeLoading(false)
    }
  }

  // 设置呼叫转移
  const applyForwarding = async () => {
    setForwardingLoading(true)
    try {
      const response = await api.setCallForwarding({
        forward_type: forwardType,
        number: forwardNumber,
        timeout: forwardType === 'noreply' ? forwardTimeout : undefined,
      })
      if (response.status === 'ok') {
        setSuccess('呼叫转移设置已保存')
        void fetchForwarding()
      } else {
        setError(response.message)
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setForwardingLoading(false)
    }
  }

  // 设置通话设置
  const handleSetCallSetting = async (property: string, value: string) => {
    setSettingsLoading(true)
    try {
      const response = await api.setCallSettings({ property, value })
      if (response.status === 'ok') {
        setSuccess('通话设置已保存')
        void fetchCallSettings()
      } else {
        setError(response.message)
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setSettingsLoading(false)
    }
  }

  // 状态翻译
  const getStateLabel = (state: string) => {
    const labels: Record<string, string> = {
      active: '通话中',
      dialing: '拨号中',
      alerting: '响铃中',
      incoming: '来电',
      held: '保持',
    }
    return labels[state] || state
  }

  // 格式化通话时长
  const formatDuration = (seconds: number) => {
    const mins = Math.floor(seconds / 60)
    const secs = seconds % 60
    return `${mins}:${secs.toString().padStart(2, '0')}`
  }

  // 格式化时间
  const formatTime = (timestamp: string) => {
    try {
      const date = new Date(timestamp)
      const now = new Date()
      const isToday = date.toDateString() === now.toDateString()
      if (isToday) {
        return date.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' })
      }
      return date.toLocaleDateString('zh-CN', { month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' })
    } catch {
      return timestamp
    }
  }

  // 获取通话记录图标
  const getCallIcon = (direction: string, answered: boolean) => {
    if (direction === 'missed') return <PhoneMissed color="error" />
    if (direction === 'incoming') return answered ? <CallReceived color="success" /> : <PhoneMissed color="error" />
    return <CallMade color="primary" />
  }

  return (
    <Box>
      <Box display="flex" alignItems="center" gap={1} mb={2}>
        <PhoneIcon color="primary" />
        <Typography variant="h5" fontWeight={600}>
          电话管理
        </Typography>
      </Box>

      {/* 错误和成功提示 */}
      <Snackbar open={!!error} autoHideDuration={4000} onClose={() => setError(null)} anchorOrigin={{ vertical: 'top', horizontal: 'center' }}>
        <Alert severity="error" onClose={() => setError(null)} variant="filled">{error}</Alert>
      </Snackbar>
      <Snackbar open={!!success} autoHideDuration={3000} onClose={() => setSuccess(null)} anchorOrigin={{ vertical: 'top', horizontal: 'center' }}>
        <Alert severity="success" onClose={() => setSuccess(null)} variant="filled">{success}</Alert>
      </Snackbar>

      {/* 当前通话浮层 */}
      <Fade in={calls.length > 0}>
        <Paper 
          elevation={6} 
          sx={{ 
            mb: 2, 
            p: 2, 
            bgcolor: 'success.main', 
            color: 'white',
            display: calls.length > 0 ? 'block' : 'none',
          }}
        >
          <Box display="flex" justifyContent="space-between" alignItems="center">
            <Box display="flex" alignItems="center" gap={2}>
              <PhoneCallback />
              <Box>
                <Typography variant="h6" fontWeight={600}>
                  {calls.length === 1 ? calls[0].phone_number : `${calls.length} 个通话中`}
                </Typography>
                {calls.length === 1 && (
                  <Typography variant="body2" sx={{ opacity: 0.9 }}>
                    {getStateLabel(calls[0].state)} - {calls[0].direction === 'incoming' ? '来电' : '拨出'}
                  </Typography>
                )}
              </Box>
            </Box>
            <Box display="flex" gap={1}>
              {calls.length === 1 && calls[0].state === 'incoming' && (
                <Button
                  variant="contained"
                  color="inherit"
                  sx={{ color: 'success.main', bgcolor: 'white' }}
                  startIcon={<Call />}
                  onClick={() => void handleAnswer(calls[0].path, calls[0].phone_number)}
                >
                  接听
                </Button>
              )}
              <Button
                variant="contained"
                color="error"
                startIcon={<CallEnd />}
                onClick={() => void handleHangupAll()}
              >
                挂断{calls.length > 1 ? '全部' : ''}
              </Button>
            </Box>
          </Box>
        </Paper>
      </Fade>

      {/* Tab 导航 */}
      <Tabs value={tabValue} onChange={(_, v: number) => setTabValue(v)} sx={{ mb: 2 }}>
        <Tab icon={<Dialpad />} label="拨号" iconPosition="start" />
        <Tab icon={<History />} label="通话记录" iconPosition="start" />
        <Tab icon={<Settings />} label="设置" iconPosition="start" />
      </Tabs>

      {/* 拨号盘 Tab */}
      {tabValue === 0 && (
        <Card>
          <CardContent>
            <Box display="flex" flexDirection="column" alignItems="center" maxWidth={320} mx="auto">
              {/* 号码显示 */}
              <TextField
                fullWidth
                variant="standard"
                value={dialNumber}
                onChange={(e) => setDialNumber(e.target.value)}
                placeholder="输入电话号码"
                slotProps={{
                  input: {
                    endAdornment: dialNumber && (
                      <IconButton size="small" onClick={handleBackspace}>
                        <Backspace />
                      </IconButton>
                    ),
                    sx: { fontSize: '1.5rem', textAlign: 'center' },
                  },
                }}
                sx={{ mb: 3 }}
              />

              {/* 拨号盘 */}
              <Box sx={{ width: '100%' }}>
                {dialpadButtons.map((row, rowIndex) => (
                  <Box key={rowIndex} display="flex" justifyContent="center" gap={2} mb={1.5}>
                    {row.map((digit) => (
                      <Button
                        key={digit}
                        variant="outlined"
                        onClick={() => handleDialpadPress(digit)}
                        sx={{
                          width: 72,
                          height: 72,
                          borderRadius: '50%',
                          fontSize: '1.5rem',
                          fontWeight: 500,
                        }}
                      >
                        {digit}
                      </Button>
                    ))}
                  </Box>
                ))}
              </Box>

              {/* 拨打按钮 */}
              <Button
                variant="contained"
                color="success"
                size="large"
                startIcon={dialLoading ? <CircularProgress size={20} color="inherit" /> : <PhoneIcon />}
                onClick={() => void handleDial()}
                disabled={dialLoading || !dialNumber.trim()}
                sx={{ mt: 2, width: 160, height: 56, borderRadius: 28 }}
              >
                {dialLoading ? '拨号中' : '拨打'}
              </Button>
            </Box>
          </CardContent>
        </Card>
      )}

      {/* 通话记录 Tab */}
      {tabValue === 1 && (
        <Card>
          <CardContent>
            {/* 统计信息 */}
            {callStats && (
              <Box display="flex" gap={2} mb={2} flexWrap="wrap">
                <Paper sx={{ p: 1.5, flex: 1, minWidth: 80 }}>
                  <Typography variant="h6" color="primary" fontWeight={600}>{callStats.total}</Typography>
                  <Typography variant="caption" color="text.secondary">总计</Typography>
                </Paper>
                <Paper sx={{ p: 1.5, flex: 1, minWidth: 80 }}>
                  <Typography variant="h6" color="success.main" fontWeight={600}>{callStats.incoming}</Typography>
                  <Typography variant="caption" color="text.secondary">来电</Typography>
                </Paper>
                <Paper sx={{ p: 1.5, flex: 1, minWidth: 80 }}>
                  <Typography variant="h6" color="info.main" fontWeight={600}>{callStats.outgoing}</Typography>
                  <Typography variant="caption" color="text.secondary">拨出</Typography>
                </Paper>
                <Paper sx={{ p: 1.5, flex: 1, minWidth: 80 }}>
                  <Typography variant="h6" color="error.main" fontWeight={600}>{callStats.missed}</Typography>
                  <Typography variant="caption" color="text.secondary">未接</Typography>
                </Paper>
              </Box>
            )}

            {/* 操作栏 */}
            <Box display="flex" justifyContent="space-between" alignItems="center" mb={2}>
              <Typography variant="subtitle1" fontWeight={600}>
                通话记录 ({callHistory.length})
              </Typography>
              <Box display="flex" gap={1}>
                <IconButton color="primary" onClick={() => void fetchCallHistory()} disabled={historyLoading}>
                  <Refresh />
                </IconButton>
                {callHistory.length > 0 && (
                  <Button
                    variant="outlined"
                    color="error"
                    size="small"
                    startIcon={<DeleteSweep />}
                    onClick={() => setClearDialogOpen(true)}
                  >
                    清空
                  </Button>
                )}
              </Box>
            </Box>

            {/* 通话记录列表 */}
            {historyLoading && callHistory.length === 0 ? (
              <Box display="flex" justifyContent="center" py={4}>
                <CircularProgress />
              </Box>
            ) : callHistory.length === 0 ? (
              <Alert severity="info">暂无通话记录</Alert>
            ) : (
              <List sx={{ maxHeight: 400, overflow: 'auto' }}>
                {callHistory.map((record) => (
                  <ListItem key={record.id} divider>
                    <ListItemAvatar>
                      <Avatar sx={{ bgcolor: record.direction === 'missed' ? 'error.light' : record.direction === 'incoming' ? 'success.light' : 'primary.light' }}>
                        {getCallIcon(record.direction, record.answered)}
                      </Avatar>
                    </ListItemAvatar>
                    <ListItemText
                      primary={
                        <Box display="flex" alignItems="center" gap={1}>
                          <Typography variant="body1" fontWeight={600}>{record.phone_number}</Typography>
                          {record.duration > 0 && (
                            <Chip label={formatDuration(record.duration)} size="small" variant="outlined" />
                          )}
                        </Box>
                      }
                      secondary={formatTime(record.start_time)}
                    />
                    <ListItemSecondaryAction>
                      <Box display="flex" gap={0.5}>
                        <IconButton size="small" color="primary" onClick={() => handleDialFromHistory(record.phone_number)}>
                          <PhoneIcon fontSize="small" />
                        </IconButton>
                        <IconButton size="small" color="error" onClick={() => void handleDeleteRecord(record.id)}>
                          <Delete fontSize="small" />
                        </IconButton>
                      </Box>
                    </ListItemSecondaryAction>
                  </ListItem>
                ))}
              </List>
            )}
          </CardContent>
        </Card>
      )}

      {/* 设置 Tab */}
      {tabValue === 2 && (
        <Grid container spacing={2}>
          {/* 音量控制 */}
          <Grid size={{ xs: 12 }}>
            <Accordion defaultExpanded>
              <AccordionSummary expandIcon={<ExpandMore />}>
                <Box display="flex" alignItems="center" gap={1}>
                  <VolumeUp color="primary" />
                  <Typography variant="h6" fontWeight={600}>通话音量</Typography>
                </Box>
              </AccordionSummary>
              <AccordionDetails>
                <Box mb={3}>
                  <Box display="flex" alignItems="center" mb={1}>
                    <VolumeUp sx={{ mr: 2, color: 'text.secondary' }} />
                    <Typography sx={{ minWidth: 80 }}>扬声器</Typography>
                    <Slider
                      value={speakerVolume}
                      onChange={(_, v: number | number[]) => setSpeakerVolume(Array.isArray(v) ? v[0] : v)}
                      min={0}
                      max={100}
                      valueLabelDisplay="auto"
                      sx={{ mx: 2 }}
                    />
                    <Typography sx={{ minWidth: 40 }}>{speakerVolume}%</Typography>
                  </Box>
                  <Box display="flex" alignItems="center" mb={1}>
                    <Mic sx={{ mr: 2, color: 'text.secondary' }} />
                    <Typography sx={{ minWidth: 80 }}>麦克风</Typography>
                    <Slider
                      value={micVolume}
                      onChange={(_, v: number | number[]) => setMicVolume(Array.isArray(v) ? v[0] : v)}
                      min={0}
                      max={100}
                      valueLabelDisplay="auto"
                      sx={{ mx: 2 }}
                    />
                    <Typography sx={{ minWidth: 40 }}>{micVolume}%</Typography>
                  </Box>
                  <FormControlLabel
                    control={<Switch checked={muted} onChange={(e) => setMuted(e.target.checked)} color="error" />}
                    label={<Box display="flex" alignItems="center"><MicOff sx={{ mr: 1 }} />静音</Box>}
                  />
                </Box>
                <Button variant="contained" fullWidth onClick={() => void saveVolume()} disabled={volumeLoading}>
                  {volumeLoading ? <CircularProgress size={20} /> : '保存音量设置'}
                </Button>
              </AccordionDetails>
            </Accordion>
          </Grid>

          {/* 呼叫转移设置 */}
          <Grid size={{ xs: 12, md: 6 }}>
            <Accordion>
              <AccordionSummary expandIcon={<ExpandMore />}>
                <Box display="flex" alignItems="center" gap={1}>
                  <PhoneForwarded color="primary" />
                  <Typography variant="h6" fontWeight={600}>呼叫转移</Typography>
                  <Chip label="查询耗时" size="small" color="warning" variant="outlined" />
                </Box>
              </AccordionSummary>
              <AccordionDetails>
                <Box display="flex" justifyContent="flex-end" mb={2}>
                  <Button
                    startIcon={forwardingLoading ? <CircularProgress size={16} /> : <Refresh />}
                    onClick={() => void fetchForwarding()}
                    disabled={forwardingLoading}
                    size="small"
                    variant="outlined"
                  >
                    {forwardingLoading ? '查询中...' : '查询设置'}
                  </Button>
                </Box>
                {forwardingLoading ? (
                  <Box display="flex" flexDirection="column" alignItems="center" py={2}>
                    <CircularProgress />
                    <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>正在查询...</Typography>
                  </Box>
                ) : (
                  <>
                    {forwarding && (
                      <Box mb={3}>
                        <Typography variant="subtitle2" color="text.secondary" gutterBottom>当前设置:</Typography>
                        <Box display="flex" flexWrap="wrap" gap={1}>
                          <Chip icon={<CallReceived />} label={`无条件: ${forwarding.voice_unconditional || '未设置'}`} size="small" variant="outlined" />
                          <Chip icon={<PhoneMissed />} label={`占线: ${forwarding.voice_busy || '未设置'}`} size="small" variant="outlined" />
                          <Chip icon={<PhoneDisabled />} label={`无应答: ${forwarding.voice_no_reply || '未设置'}`} size="small" variant="outlined" />
                        </Box>
                      </Box>
                    )}
                    <Divider sx={{ my: 2 }} />
                    <FormControl fullWidth sx={{ mb: 2 }}>
                      <InputLabel>转移类型</InputLabel>
                      <Select value={forwardType} label="转移类型" onChange={(e) => setForwardType(e.target.value)}>
                        <MenuItem value="unconditional">无条件转移</MenuItem>
                        <MenuItem value="busy">占线时转移</MenuItem>
                        <MenuItem value="noreply">无应答时转移</MenuItem>
                        <MenuItem value="notreachable">不可达时转移</MenuItem>
                      </Select>
                    </FormControl>
                    <TextField
                      fullWidth
                      label="转移号码"
                      value={forwardNumber}
                      onChange={(e) => setForwardNumber(e.target.value)}
                      placeholder="留空则禁用"
                      sx={{ mb: 2 }}
                    />
                    {forwardType === 'noreply' && (
                      <Box mb={2}>
                        <Typography variant="body2" gutterBottom>超时: {forwardTimeout}秒</Typography>
                        <Slider value={forwardTimeout} onChange={(_, v: number | number[]) => setForwardTimeout(Array.isArray(v) ? v[0] : v)} min={5} max={60} step={5} marks valueLabelDisplay="auto" />
                      </Box>
                    )}
                    <Button variant="contained" fullWidth onClick={() => void applyForwarding()} disabled={forwardingLoading}>
                      保存
                    </Button>
                  </>
                )}
              </AccordionDetails>
            </Accordion>
          </Grid>

          {/* 通话设置 */}
          <Grid size={{ xs: 12, md: 6 }}>
            <Accordion>
              <AccordionSummary expandIcon={<ExpandMore />}>
                <Box display="flex" alignItems="center" gap={1}>
                  <Settings color="primary" />
                  <Typography variant="h6" fontWeight={600}>通话设置</Typography>
                  <Chip label="查询耗时" size="small" color="warning" variant="outlined" />
                </Box>
              </AccordionSummary>
              <AccordionDetails>
                <Box display="flex" justifyContent="flex-end" mb={2}>
                  <Button
                    startIcon={settingsLoading ? <CircularProgress size={16} /> : <Refresh />}
                    onClick={() => void fetchCallSettings()}
                    disabled={settingsLoading}
                    size="small"
                    variant="outlined"
                  >
                    {settingsLoading ? '查询中...' : '查询设置'}
                  </Button>
                </Box>
                {settingsLoading ? (
                  <Box display="flex" flexDirection="column" alignItems="center" py={2}>
                    <CircularProgress />
                    <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>正在查询...</Typography>
                  </Box>
                ) : callSettings ? (
                  <Box>
                    <Box display="flex" justifyContent="space-between" alignItems="center" mb={2}>
                      <Typography>来电显示</Typography>
                      <FormControl size="small" sx={{ minWidth: 120 }}>
                        <Select value={callSettings.hide_caller_id} onChange={(e) => void handleSetCallSetting('HideCallerId', e.target.value)}>
                          <MenuItem value="default">默认</MenuItem>
                          <MenuItem value="enabled">隐藏</MenuItem>
                          <MenuItem value="disabled">显示</MenuItem>
                        </Select>
                      </FormControl>
                    </Box>
                    <Divider sx={{ my: 1 }} />
                    <Box display="flex" justifyContent="space-between" alignItems="center" mb={2}>
                      <Typography>呼叫等待</Typography>
                      <FormControl size="small" sx={{ minWidth: 120 }}>
                        <Select value={callSettings.voice_call_waiting} onChange={(e) => void handleSetCallSetting('VoiceCallWaiting', e.target.value)}>
                          <MenuItem value="enabled">启用</MenuItem>
                          <MenuItem value="disabled">禁用</MenuItem>
                        </Select>
                      </FormControl>
                    </Box>
                  </Box>
                ) : (
                  <Alert severity="info">点击上方按钮查询设置</Alert>
                )}
              </AccordionDetails>
            </Accordion>
          </Grid>
        </Grid>
      )}

      {/* 清空确认对话框 */}
      <Dialog open={clearDialogOpen} onClose={() => setClearDialogOpen(false)}>
        <DialogTitle>确认清空</DialogTitle>
        <DialogContent>
          <Typography>确定要清空所有通话记录吗？此操作不可撤销。</Typography>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setClearDialogOpen(false)}>取消</Button>
          <Button onClick={() => void handleClearHistory()} color="error" variant="contained">确认清空</Button>
        </DialogActions>
      </Dialog>
    </Box>
  )
}
