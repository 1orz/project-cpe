/*
 * @Author: 1orz cloudorzi@gmail.com
 * @Date: 2025-12-09 17:34:01
 * @LastEditors: 1orz cloudorzi@gmail.com
 * @LastEditTime: 2025-12-13 12:45:01
 * @FilePath: /udx710-backend/frontend/src/pages/SMS.tsx
 * @Description: 
 * 
 * Copyright (c) 2025 by 1orz, All Rights Reserved. 
 */
import { useState, useEffect, useRef, useCallback } from 'react'
import {
  Box,
  Card,
  CardContent,
  Typography,
  Button,
  TextField,
  List,
  ListItemText,
  ListItemButton,
  Alert,
  CircularProgress,
  Chip,
  IconButton,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Divider,
  Paper,
  Badge,
  Avatar,
  Snackbar,
  useTheme,
  useMediaQuery,
  InputAdornment,
} from '@mui/material'
import {
  Sms as SmsIcon,
  Send,
  Refresh,
  Person,
  ArrowBack,
  Add,
  DeleteSweep,
} from '@mui/icons-material'
import { api, type SmsMessage, type SmsStats } from '../api'

interface ConversationGroup {
  phoneNumber: string
  messages: SmsMessage[]
  lastMessage: SmsMessage
  unreadCount: number
}

export default function SMSPage() {
  const theme = useTheme()
  const isMobile = useMediaQuery(theme.breakpoints.down('md'))
  
  const [messages, setMessages] = useState<SmsMessage[]>([])
  const [stats, setStats] = useState<SmsStats | null>(null)
  const [loading, setLoading] = useState(false)
  const [sendLoading, setSendLoading] = useState(false)
  const [phoneNumber, setPhoneNumber] = useState('')
  const [content, setContent] = useState('')
  const [error, setError] = useState<string | null>(null)
  const [success, setSuccess] = useState<string | null>(null)
  const [clearDialogOpen, setClearDialogOpen] = useState(false)
  const [newChatDialogOpen, setNewChatDialogOpen] = useState(false)
  const [newChatNumber, setNewChatNumber] = useState('')
  
  // 对话状态
  const [conversations, setConversations] = useState<ConversationGroup[]>([])
  const [selectedConversation, setSelectedConversation] = useState<string | null>(null)
  const [conversationMessages, setConversationMessages] = useState<SmsMessage[]>([])
  const [conversationLoading, setConversationLoading] = useState(false)
  
  // 聊天区域滚动引用
  const chatEndRef = useRef<HTMLDivElement>(null)
  // 输入框焦点状态 - 有焦点时暂停刷新避免失焦
  const inputFocusedRef = useRef(false)

  // 滚动到底部
  const scrollToBottom = useCallback(() => {
    chatEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [])

  // 获取短信列表
  const fetchMessages = useCallback(async () => {
    setLoading(true)
    setError(null)
    try {
      const response = await api.getSmsList({ limit: 100, offset: 0 })
      if (response.status === 'ok' && response.data) {
        setMessages(response.data)
        groupConversations(response.data)
      } else {
        setError(response.message)
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setLoading(false)
    }
  }, [])

  // 按联系人分组对话
  const groupConversations = (msgs: SmsMessage[]) => {
    const groups = new Map<string, SmsMessage[]>()
    
    msgs.forEach(msg => {
      const key = msg.phone_number
      if (!groups.has(key)) {
        groups.set(key, [])
      }
      groups.get(key)?.push(msg)
    })

    const conversationList: ConversationGroup[] = []
    groups.forEach((messages, phoneNumber) => {
      messages.sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime())
      conversationList.push({
        phoneNumber,
        messages,
        lastMessage: messages[0],
        unreadCount: messages.filter(m => m.direction === 'incoming' && m.status === 'received').length,
      })
    })

    conversationList.sort((a, b) => 
      new Date(b.lastMessage.timestamp).getTime() - new Date(a.lastMessage.timestamp).getTime()
    )

    setConversations(conversationList)
  }

  // 获取对话历史
  const fetchConversation = useCallback(async (phone: string) => {
    setConversationLoading(true)
    try {
      const response = await api.getSmsConversation({ phone_number: phone })
      if (response.status === 'ok' && response.data) {
        const sorted = [...response.data].sort((a, b) => 
          new Date(a.timestamp).getTime() - new Date(b.timestamp).getTime()
        )
        setConversationMessages(sorted)
        setTimeout(scrollToBottom, 100)
      }
    } catch {
      const localMsgs = messages.filter(m => m.phone_number === phone)
      const sorted = [...localMsgs].sort((a, b) => 
        new Date(a.timestamp).getTime() - new Date(b.timestamp).getTime()
      )
      setConversationMessages(sorted)
      setTimeout(scrollToBottom, 100)
    } finally {
      setConversationLoading(false)
    }
  }, [messages, scrollToBottom])

  // 获取统计信息
  const fetchStats = useCallback(async () => {
    try {
      const response = await api.getSmsStats()
      if (response.status === 'ok' && response.data) {
        setStats(response.data)
      }
    } catch (err) {
      console.error('获取短信统计失败:', err)
    }
  }, [])

  useEffect(() => {
    void fetchMessages()
    void fetchStats()
    const interval = setInterval(() => {
      // 输入框有焦点时跳过刷新，避免失焦问题
      if (inputFocusedRef.current) {
        return
      }
      void fetchMessages()
      void fetchStats()
    }, 10000)
    return () => clearInterval(interval)
  }, [fetchMessages, fetchStats])

  // 选择对话
  const handleSelectConversation = (phone: string) => {
    setSelectedConversation(phone)
    setPhoneNumber(phone)
    void fetchConversation(phone)
  }

  // 返回对话列表
  const handleBackToList = () => {
    setSelectedConversation(null)
    setConversationMessages([])
  }

  // 开始新对话
  const handleStartNewChat = () => {
    if (!newChatNumber.trim()) {
      setError('请输入电话号码')
      return
    }
    setNewChatDialogOpen(false)
    setSelectedConversation(newChatNumber)
    setPhoneNumber(newChatNumber)
    setConversationMessages([])
    setNewChatNumber('')
  }

  // 发送短信
  const handleSend = async () => {
    if (!phoneNumber.trim()) {
      setError('请输入电话号码')
      return
    }
    if (!content.trim()) {
      setError('请输入短信内容')
      return
    }

    setSendLoading(true)
    setError(null)
    setSuccess(null)

    try {
      const response = await api.sendSms(phoneNumber, content)
      if (response.status === 'ok') {
        setSuccess(`短信已发送到 ${phoneNumber}`)
        setContent('')
        setTimeout(() => {
          void fetchMessages()
          void fetchStats()
          if (selectedConversation) {
            void fetchConversation(selectedConversation)
          }
        }, 1000)
      } else {
        setError(response.message)
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setSendLoading(false)
    }
  }

  // 清空所有短信
  const handleClearAll = async () => {
    setError(null)
    setSuccess(null)
    setClearDialogOpen(false)

    try {
      const response = await api.clearAllSms()
      if (response.status === 'ok') {
        setSuccess('所有短信已清空')
        setMessages([])
        setConversations([])
        setSelectedConversation(null)
        void fetchStats()
      } else {
        setError(response.message)
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    }
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

  // 格式化简短时间
  const formatShortTime = (timestamp: string) => {
    try {
      const date = new Date(timestamp)
      const now = new Date()
      const isToday = date.toDateString() === now.toDateString()
      if (isToday) {
        return date.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' })
      }
      return date.toLocaleDateString('zh-CN', { month: '2-digit', day: '2-digit' })
    } catch {
      return timestamp
    }
  }

  // 对话列表 JSX
  const conversationListContent = (
    <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      {/* 统计信息 */}
      {stats && (
        <Box display="flex" gap={1} p={2} flexWrap="wrap">
          <Paper sx={{ p: 1, flex: 1, minWidth: 60, textAlign: 'center' }}>
            <Typography variant="h6" color="primary" fontWeight={600}>{stats.total}</Typography>
            <Typography variant="caption" color="text.secondary">总计</Typography>
          </Paper>
          <Paper sx={{ p: 1, flex: 1, minWidth: 60, textAlign: 'center' }}>
            <Typography variant="h6" color="success.main" fontWeight={600}>{stats.incoming}</Typography>
            <Typography variant="caption" color="text.secondary">接收</Typography>
          </Paper>
          <Paper sx={{ p: 1, flex: 1, minWidth: 60, textAlign: 'center' }}>
            <Typography variant="h6" color="info.main" fontWeight={600}>{stats.outgoing}</Typography>
            <Typography variant="caption" color="text.secondary">发送</Typography>
          </Paper>
        </Box>
      )}

      {/* 操作栏 */}
      <Box display="flex" justifyContent="space-between" alignItems="center" px={2} pb={1}>
        <Typography variant="subtitle1" fontWeight={600}>
          对话 ({conversations.length})
        </Typography>
        <Box display="flex" gap={0.5}>
          <IconButton size="small" color="primary" onClick={() => setNewChatDialogOpen(true)}>
            <Add />
          </IconButton>
          <IconButton size="small" color="primary" onClick={() => void fetchMessages()} disabled={loading}>
            <Refresh />
          </IconButton>
          {conversations.length > 0 && (
            <IconButton size="small" color="error" onClick={() => setClearDialogOpen(true)}>
              <DeleteSweep />
            </IconButton>
          )}
        </Box>
      </Box>

      <Divider />

      {/* 对话列表 */}
      {loading && conversations.length === 0 ? (
        <Box display="flex" justifyContent="center" py={4}><CircularProgress /></Box>
      ) : conversations.length === 0 ? (
        <Box p={2}><Alert severity="info">暂无对话，点击 + 开始新对话</Alert></Box>
      ) : (
        <List sx={{ flex: 1, overflow: 'auto' }}>
          {conversations.map((conv, idx) => (
            <Box key={conv.phoneNumber}>
              <ListItemButton 
                onClick={() => handleSelectConversation(conv.phoneNumber)}
                selected={selectedConversation === conv.phoneNumber}
              >
                <Avatar sx={{ mr: 2, bgcolor: 'primary.light' }}><Person /></Avatar>
                <ListItemText
                  primary={
                    <Box display="flex" alignItems="center" gap={1}>
                      <Typography fontWeight={600}>{conv.phoneNumber}</Typography>
                      <Badge badgeContent={conv.messages.length} color="primary" max={99} />
                    </Box>
                  }
                  secondary={
                    <Typography variant="body2" color="text.secondary" noWrap sx={{ maxWidth: 180 }}>
                      {conv.lastMessage.direction === 'outgoing' ? '你: ' : ''}{conv.lastMessage.content}
                    </Typography>
                  }
                />
                <Typography variant="caption" color="text.secondary">
                  {formatShortTime(conv.lastMessage.timestamp)}
                </Typography>
              </ListItemButton>
              {idx < conversations.length - 1 && <Divider />}
            </Box>
          ))}
        </List>
      )}
    </Box>
  )

  // 聊天区域 JSX
  const chatAreaContent = (
    <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      {/* 顶部：联系人信息 */}
      <Box 
        sx={{ 
          p: 2, 
          borderBottom: 1, 
          borderColor: 'divider',
          display: 'flex',
          alignItems: 'center',
          gap: 1,
        }}
      >
        {isMobile && (
          <IconButton onClick={handleBackToList} edge="start">
            <ArrowBack />
          </IconButton>
        )}
        <Avatar sx={{ bgcolor: 'primary.main' }}><Person /></Avatar>
        <Typography variant="h6" fontWeight={600}>{selectedConversation}</Typography>
      </Box>

      {/* 中部：消息气泡 */}
      <Box 
        sx={{ 
          flex: 1, 
          overflow: 'auto', 
          p: 2,
          bgcolor: (theme) => theme.palette.mode === 'dark' ? 'grey.900' : 'grey.50',
        }}
      >
        {conversationLoading ? (
          <Box display="flex" justifyContent="center" py={4}><CircularProgress /></Box>
        ) : conversationMessages.length === 0 ? (
          <Box display="flex" justifyContent="center" alignItems="center" height="100%">
            <Typography color="text.secondary">开始发送第一条消息</Typography>
          </Box>
        ) : (
          <>
            {conversationMessages.map((msg, idx) => (
              <Box
                key={msg.id || idx}
                display="flex"
                justifyContent={msg.direction === 'outgoing' ? 'flex-end' : 'flex-start'}
                mb={1.5}
              >
                <Paper
                  elevation={1}
                  sx={{
                    p: 1.5,
                    maxWidth: '75%',
                    bgcolor: msg.direction === 'outgoing' 
                      ? 'primary.main' 
                      : (theme) => theme.palette.mode === 'dark' ? 'grey.800' : 'white',
                    color: msg.direction === 'outgoing' 
                      ? 'white' 
                      : 'text.primary',
                    borderRadius: 2,
                    borderTopRightRadius: msg.direction === 'outgoing' ? 0 : 16,
                    borderTopLeftRadius: msg.direction === 'incoming' ? 0 : 16,
                  }}
                >
                  <Typography variant="body2" sx={{ wordBreak: 'break-word', whiteSpace: 'pre-wrap' }}>
                    {msg.content}
                  </Typography>
                  <Box display="flex" alignItems="center" justifyContent="flex-end" gap={0.5} mt={0.5}>
                    <Typography 
                      variant="caption" 
                      sx={{ opacity: 0.7 }}
                    >
                      {formatTime(msg.timestamp)}
                    </Typography>
                    {msg.direction === 'outgoing' && (
                      msg.status === 'sent' ? (
                        <Chip label="已发送" size="small" sx={{ height: 16, fontSize: '0.65rem', bgcolor: 'rgba(255,255,255,0.2)' }} />
                      ) : msg.status === 'failed' ? (
                        <Chip label="失败" size="small" color="error" sx={{ height: 16, fontSize: '0.65rem' }} />
                      ) : null
                    )}
                  </Box>
                </Paper>
              </Box>
            ))}
            <div ref={chatEndRef} />
          </>
        )}
      </Box>

      {/* 底部：固定输入框 */}
      <Box 
        sx={{ 
          p: 2, 
          borderTop: 1, 
          borderColor: 'divider',
          bgcolor: 'background.paper',
        }}
      >
        <TextField
          fullWidth
          multiline
          maxRows={4}
          value={content}
          onChange={(e) => setContent(e.target.value)}
          placeholder="输入短信内容..."
          disabled={sendLoading}
          onFocus={() => { inputFocusedRef.current = true }}
          onBlur={() => { inputFocusedRef.current = false }}
          onKeyDown={(e) => {
            if (e.key === 'Enter' && !e.shiftKey) {
              e.preventDefault()
              void handleSend()
            }
          }}
          slotProps={{
            input: {
              endAdornment: (
                <InputAdornment position="end">
                  <IconButton
                    color="primary"
                    onClick={() => void handleSend()}
                    disabled={sendLoading || !content.trim()}
                  >
                    {sendLoading ? <CircularProgress size={24} /> : <Send />}
                  </IconButton>
                </InputAdornment>
              ),
            },
          }}
        />
        <Typography variant="caption" color="text.secondary" sx={{ mt: 0.5, display: 'block' }}>
          {content.length} 字符 | Enter 发送，Shift+Enter 换行
        </Typography>
      </Box>
    </Box>
  )

  // 空状态提示 JSX
  const emptyStateContent = (
    <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', p: 4 }}>
      <SmsIcon sx={{ fontSize: 64, color: 'text.secondary', mb: 2 }} />
      <Typography variant="h6" color="text.secondary" gutterBottom>
        选择一个对话开始聊天
      </Typography>
      <Typography variant="body2" color="text.secondary">
        或点击左上角 + 开始新对话
      </Typography>
    </Box>
  )

  return (
    <Box sx={{ height: 'calc(100vh - 140px)', minHeight: 500 }}>
      <Box display="flex" alignItems="center" gap={1} mb={2}>
        <SmsIcon color="primary" />
        <Typography variant="h5" fontWeight={600}>
          短信管理
        </Typography>
      </Box>

      {/* 错误和成功提示 */}
      <Snackbar open={!!error} autoHideDuration={4000} onClose={() => setError(null)} anchorOrigin={{ vertical: 'top', horizontal: 'center' }}>
        <Alert severity="error" onClose={() => setError(null)} variant="filled">{error}</Alert>
      </Snackbar>
      <Snackbar open={!!success} autoHideDuration={3000} onClose={() => setSuccess(null)} anchorOrigin={{ vertical: 'top', horizontal: 'center' }}>
        <Alert severity="success" onClose={() => setSuccess(null)} variant="filled">{success}</Alert>
      </Snackbar>

      {/* 主内容区域 */}
      <Card sx={{ height: 'calc(100% - 48px)' }}>
        <CardContent sx={{ height: '100%', p: 0, '&:last-child': { pb: 0 } }}>
          {isMobile ? (
            // 移动端：对话列表或聊天详情
            selectedConversation ? chatAreaContent : conversationListContent
          ) : (
            // PC端：左右分栏
            <Box display="flex" height="100%">
              {/* 左侧：对话列表 */}
              <Box 
                sx={{ 
                  width: 320, 
                  borderRight: 1, 
                  borderColor: 'divider',
                  flexShrink: 0,
                }}
              >
                {conversationListContent}
              </Box>
              {/* 右侧：聊天区域 */}
              <Box sx={{ flex: 1 }}>
                {selectedConversation ? chatAreaContent : emptyStateContent}
              </Box>
            </Box>
          )}
        </CardContent>
      </Card>

      {/* 清空确认对话框 */}
      <Dialog open={clearDialogOpen} onClose={() => setClearDialogOpen(false)}>
        <DialogTitle>确认清空</DialogTitle>
        <DialogContent>
          <Typography>确定要清空所有短信记录吗？此操作不可撤销。</Typography>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setClearDialogOpen(false)}>取消</Button>
          <Button onClick={() => void handleClearAll()} color="error" variant="contained">确认清空</Button>
        </DialogActions>
      </Dialog>

      {/* 新对话对话框 */}
      <Dialog open={newChatDialogOpen} onClose={() => setNewChatDialogOpen(false)}>
        <DialogTitle>新建对话</DialogTitle>
        <DialogContent>
          <TextField
            autoFocus
            fullWidth
            label="电话号码"
            value={newChatNumber}
            onChange={(e) => setNewChatNumber(e.target.value)}
            placeholder="输入收件人电话号码"
            sx={{ mt: 1 }}
            onKeyDown={(e) => {
              if (e.key === 'Enter') {
                handleStartNewChat()
              }
            }}
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setNewChatDialogOpen(false)}>取消</Button>
          <Button onClick={handleStartNewChat} variant="contained">开始对话</Button>
        </DialogActions>
      </Dialog>
    </Box>
  )
}
