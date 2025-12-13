/*
 * @Author: 1orz cloudorzi@gmail.com
 * @Date: 2025-12-03 13:58:02
 * @LastEditors: 1orz cloudorzi@gmail.com
 * @LastEditTime: 2025-12-13 12:44:49
 * @FilePath: /udx710-backend/frontend/src/pages/ATConsole.tsx
 * @Description: 
 * 
 * Copyright (c) 2025 by 1orz, All Rights Reserved. 
 */
import { useState, useRef, useEffect } from 'react'
import {
  Box,
  Typography,
  Paper,
  TextField,
  Button,
  Chip,
  Alert,
  Divider,
  IconButton,
  Tooltip,
  Collapse,
} from '@mui/material'
import {
  Computer,
  ContentCopy,
  Fingerprint,
  Send,
  Delete,
} from '@mui/icons-material'
import { api } from '../api'
import ErrorSnackbar from '../components/ErrorSnackbar'

interface CommandHistory {
  command: string
  response: string
  timestamp: Date
  success: boolean
}

// 常用 AT 指令
const QUICK_COMMANDS = [
  { label: 'IMEI', cmd: 'AT+CGSN', desc: '查询设备 IMEI' },
  { label: 'ICCID', cmd: 'AT+CCID', desc: '查询 SIM 卡 ICCID' },
  { label: 'IMSI', cmd: 'AT+CIMI', desc: '查询 IMSI' },
  { label: '信号质量', cmd: 'AT+CSQ', desc: '查询信号强度' },
  { label: '网络注册', cmd: 'AT+CREG?', desc: '查询网络注册状态' },
  { label: '运营商', cmd: 'AT+COPS?', desc: '查询当前运营商' },
]

export default function ATConsolePage() {
  const [command, setCommand] = useState('')
  const [history, setHistory] = useState<CommandHistory[]>([])
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const responseEndRef = useRef<HTMLDivElement>(null)
  
  // IMEI 管理状态
  const [currentImei, setCurrentImei] = useState('')
  const [imeiLoading, setImeiLoading] = useState(false)

  // 自动滚动到底部
  useEffect(() => {
    responseEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [history])

  // 初始加载：获取当前 IMEI
  useEffect(() => {
    void fetchCurrentImei()
  }, [])

  // 获取当前 IMEI
  const fetchCurrentImei = async () => {
    setImeiLoading(true)
    try {
      const response = await api.sendAtCommand('AT+SPIMEI?')
      // 从响应中提取 IMEI（格式通常是 +SPIMEI: "123456789012345"）
      const match = response.match(/["']?(\d{15})["']?/)
      if (match) {
        setCurrentImei(match[1])
      }
    } catch (err) {
      console.error('Failed to fetch IMEI:', err)
    } finally {
      setImeiLoading(false)
    }
  }

  // 发送 AT 指令的实际逻辑
  const sendCommand = async (cmd?: string) => {
    const commandToSend = cmd || command
    if (!commandToSend.trim()) return

    setLoading(true)
    setError(null)

    try {
      const response = await api.sendAtCommand(commandToSend)
      
      const newEntry: CommandHistory = {
        command: commandToSend,
        response,
        timestamp: new Date(),
        success: !response.toLowerCase().includes('error'),
      }

      setHistory((prev) => [...prev, newEntry])
      setCommand('')
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setLoading(false)
    }
  }

  // 发送 AT 指令（按钮点击处理）
  const handleSendCommand = () => {
    void sendCommand()
  }

  // 快捷指令按钮点击
  const handleQuickCommand = (cmd: string) => {
    void sendCommand(cmd)
  }

  // 清空历史
  const handleClearHistory = () => {
    setHistory([])
  }

  // 复制响应
  const handleCopyResponse = (text: string) => {
    void navigator.clipboard.writeText(text)
  }

  // 键盘事件处理
  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && e.ctrlKey) {
      handleSendCommand()
    }
  }

  // 提交修改 IMEI
  const handleSubmitImei = async () => {
    if (currentImei.trim().length !== 15) {
      setError('IMEI 必须是 15 位数字')
      return
    }
    
    if (!/^\d+$/.test(currentImei.trim())) {
      setError('IMEI 只能包含数字')
      return
    }
    
    const cmd = `AT+SPIMEI=0,"${currentImei.trim()}"`
    await sendCommand(cmd)
    
    // 等待一下再重新获取，确认修改成功
    setTimeout(() => {
      void fetchCurrentImei()
    }, 1000)
  }

  const [showImeiPanel, setShowImeiPanel] = useState(false)

  return (
    <Box sx={{ height: { md: 'calc(100vh - 100px)' }, display: 'flex', flexDirection: 'column' }}>
      {/* 错误提示 Snackbar */}
      <ErrorSnackbar error={error} onClose={() => setError(null)} />

      {/* 顶部：标题栏 + 清空按钮 */}
      <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', mb: 1 }}>
        <Box display="flex" alignItems="center" gap={1}>
          <Computer fontSize="small" color="primary" />
          <Typography variant="subtitle1" fontWeight={600}>AT 控制台</Typography>
          {history.length > 0 && (
            <Chip label={history.length} size="small" color="primary" variant="outlined" />
          )}
        </Box>
        <Box display="flex" gap={1}>
          <Button
            variant="text"
            size="small"
            onClick={handleClearHistory}
            disabled={history.length === 0}
            startIcon={<Delete />}
            color="error"
          >
            清空
          </Button>
        </Box>
      </Box>

      {/* 中部：执行历史（输出区） - 占据剩余空间 */}
      <Paper 
        sx={{ 
          flex: 1, 
          mb: 1.5, 
          overflow: 'hidden',
          display: 'flex',
          flexDirection: 'column',
          minHeight: { xs: 200, md: 300 },
        }}
      >
        {history.length === 0 ? (
          <Box 
            sx={{ 
              flex: 1,
              display: 'flex', 
              alignItems: 'center', 
              justifyContent: 'center',
              bgcolor: (theme) => theme.palette.mode === 'dark' ? '#1e1e1e' : '#f8f9fa',
            }}
          >
            <Typography variant="body2" color="text.secondary">
              点击快捷指令或输入自定义指令开始
            </Typography>
          </Box>
        ) : (
          <Box
            sx={{
              flex: 1,
              overflowY: 'auto',
              backgroundColor: (theme) => theme.palette.mode === 'dark' ? '#1e1e1e' : '#1a1a2e',
              p: 1.5,
            }}
          >
            {history.map((entry, idx) => (
              <Box key={idx} mb={1.5}>
                {/* 指令头部 */}
                <Box display="flex" justifyContent="space-between" alignItems="center" mb={0.5}>
                  <Box display="flex" alignItems="center" gap={0.5} flexWrap="wrap">
                    <Chip
                      label={entry.timestamp.toLocaleTimeString()}
                      size="small"
                      sx={{
                        backgroundColor: '#2d2d2d',
                        color: '#888',
                        fontFamily: 'monospace',
                        fontSize: '0.65rem',
                        height: 18,
                      }}
                    />
                    <Chip
                      label={entry.success ? 'OK' : 'ERR'}
                      size="small"
                      color={entry.success ? 'success' : 'error'}
                      sx={{ fontFamily: 'monospace', fontSize: '0.65rem', height: 18 }}
                    />
                  </Box>
                  <IconButton
                    size="small"
                    onClick={() => handleCopyResponse(entry.response)}
                    sx={{ color: '#888', p: 0.25 }}
                  >
                    <ContentCopy sx={{ fontSize: 14 }} />
                  </IconButton>
                </Box>

                {/* 指令 */}
                <Box
                  sx={{
                    backgroundColor: '#2d2d2d',
                    borderRadius: 0.5,
                    px: 1,
                    py: 0.5,
                    mb: 0.5,
                  }}
                >
                  <Typography
                    variant="caption"
                    sx={{
                      fontFamily: 'monospace',
                      color: '#4fc3f7',
                      fontSize: '0.8rem',
                    }}
                  >
                    $ {entry.command}
                  </Typography>
                </Box>

                {/* 响应 */}
                <Box
                  sx={{
                    backgroundColor: '#0d1117',
                    borderRadius: 0.5,
                    px: 1,
                    py: 0.5,
                    borderLeft: `2px solid ${entry.success ? '#4caf50' : '#f44336'}`,
                  }}
                >
                  <Typography
                    variant="caption"
                    component="pre"
                    sx={{
                      fontFamily: 'monospace',
                      color: entry.success ? '#a5d6a7' : '#ef9a9a',
                      fontSize: '0.75rem',
                      margin: 0,
                      whiteSpace: 'pre-wrap',
                      wordBreak: 'break-all',
                    }}
                  >
                    {entry.response}
                  </Typography>
                </Box>

                {idx < history.length - 1 && <Divider sx={{ mt: 1, borderColor: '#333' }} />}
              </Box>
            ))}
            <div ref={responseEndRef} />
          </Box>
        )}
      </Paper>

      {/* 底部：输入区 */}
      <Paper sx={{ p: 1.5 }}>
        {/* 快捷指令 */}
        <Box display="flex" flexWrap="wrap" gap={0.5} mb={1.5}>
          {QUICK_COMMANDS.map((item, idx) => (
            <Tooltip key={idx} title={`${item.cmd} - ${item.desc}`} placement="top">
              <Chip
                label={item.label}
                onClick={() => handleQuickCommand(item.cmd)}
                color="primary"
                variant="outlined"
                size="small"
                clickable
                disabled={loading}
              />
            </Tooltip>
          ))}
          {/* IMEI 按钮 */}
          <Chip
            icon={<Fingerprint />}
            label="IMEI"
            onClick={() => setShowImeiPanel(!showImeiPanel)}
            color={showImeiPanel ? 'warning' : 'default'}
            variant={showImeiPanel ? 'filled' : 'outlined'}
            size="small"
            clickable
          />
        </Box>

        {/* IMEI 管理面板（折叠） */}
        <Collapse in={showImeiPanel}>
          <Box sx={{ mb: 1.5, p: 1.5, bgcolor: 'action.hover', borderRadius: 1 }}>
            <Alert severity="warning" sx={{ mb: 1, py: 0 }}>
              <Typography variant="caption">IMEI 必须是 15 位数字</Typography>
            </Alert>
            <Box display="flex" gap={1} flexWrap="wrap" alignItems="center">
              <TextField
                label="IMEI"
                value={currentImei}
                onChange={(e) => {
                  const value = e.target.value.replace(/\D/g, '').slice(0, 15)
                  setCurrentImei(value)
                }}
                placeholder="867164060028129"
                disabled={imeiLoading}
                size="small"
                inputProps={{ maxLength: 15, pattern: '[0-9]*' }}
                sx={{
                  flex: 1,
                  minWidth: 180,
                  '& input': { fontFamily: 'monospace', fontSize: '0.85rem' },
                }}
                helperText={`${currentImei.length}/15`}
              />
              <Button
                variant="contained"
                color="warning"
                size="small"
                onClick={() => void handleSubmitImei()}
                disabled={loading || imeiLoading || currentImei.length !== 15}
              >
                {loading ? '...' : '写入'}
              </Button>
              <Button
                variant="outlined"
                size="small"
                onClick={() => void fetchCurrentImei()}
                disabled={loading || imeiLoading}
              >
                {imeiLoading ? '...' : '读取'}
              </Button>
            </Box>
          </Box>
        </Collapse>

        {/* 自定义指令输入 */}
        <Box display="flex" gap={1} alignItems="flex-end">
          <TextField
            fullWidth
            value={command}
            onChange={(e) => setCommand(e.target.value)}
            onKeyDown={handleKeyPress}
            placeholder="输入 AT 指令，如: AT+CGSN (Ctrl+Enter 发送)"
            disabled={loading}
            variant="outlined"
            size="small"
            sx={{
              '& input': {
                fontFamily: 'monospace',
                fontSize: '0.9rem',
              },
            }}
          />
          <Button
            variant="contained"
            onClick={handleSendCommand}
            disabled={loading || !command.trim()}
            sx={{ minWidth: 80, height: 40 }}
          >
            {loading ? '...' : <Send />}
          </Button>
        </Box>
      </Paper>
    </Box>
  )
}

