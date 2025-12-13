/*
 * @Author: 1orz cloudorzi@gmail.com
 * @Date: 2025-12-10 09:19:05
 * @LastEditors: 1orz cloudorzi@gmail.com
 * @LastEditTime: 2025-12-13 12:44:57
 * @FilePath: /udx710-backend/frontend/src/pages/OtaUpdate.tsx
 * @Description: 
 * 
 * Copyright (c) 2025 by 1orz, All Rights Reserved. 
 */
import { useState, useEffect, useCallback, useRef } from 'react'
import {
  Box,
  Typography,
  Card,
  CardContent,
  Button,
  CircularProgress,
  Alert,
  AlertTitle,
  Chip,
  Stack,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableRow,
  Paper,
  LinearProgress,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogContentText,
  DialogActions,
  Divider,
} from '@mui/material'
import {
  CloudUpload,
  CheckCircle,
  Error as ErrorIcon,
  Warning,
  Info,
  Refresh,
  SystemUpdateAlt,
  Cancel,
  RestartAlt,
} from '@mui/icons-material'
import { api } from '../api'
import type { OtaStatusResponse, OtaUploadResponse } from '../api/types'

export default function OtaUpdate() {
  const [loading, setLoading] = useState(true)
  const [uploading, setUploading] = useState(false)
  const [applying, setApplying] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [success, setSuccess] = useState<string | null>(null)
  
  const [status, setStatus] = useState<OtaStatusResponse | null>(null)
  const [uploadResult, setUploadResult] = useState<OtaUploadResponse | null>(null)
  const [confirmDialog, setConfirmDialog] = useState<'apply' | 'cancel' | null>(null)
  
  const fileInputRef = useRef<HTMLInputElement>(null)

  const loadStatus = useCallback(async () => {
    try {
      const res = await api.getOtaStatus()
      if (res.data) {
        setStatus(res.data)
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    void loadStatus()
  }, [loadStatus])

  const handleFileSelect = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0]
    if (!file) return

    // 验证文件类型（支持 tar.gz 和 zip 格式）
    const validExtensions = ['.tar.gz', '.tgz', '.zip']
    const isValid = validExtensions.some(ext => file.name.endsWith(ext))
    
    if (!isValid) {
      setError('请上传 .tar.gz 或 .zip 格式的 OTA 更新包')
      return
    }

    setUploading(true)
    setError(null)
    setSuccess(null)
    setUploadResult(null)

    try {
      const res = await api.uploadOta(file)
      if (res.status === 'ok' && res.data) {
        setUploadResult(res.data)
        if (res.data.validation.valid) {
          setSuccess('OTA 包上传成功，验证通过')
        } else {
          setError('OTA 包验证失败：' + (res.data.validation.error || '未知错误'))
        }
        await loadStatus()
      } else {
        setError(res.message || '上传失败')
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setUploading(false)
      // 清空文件选择
      if (fileInputRef.current) {
        fileInputRef.current.value = ''
      }
    }
  }

  const handleApply = async (restartNow: boolean) => {
    setConfirmDialog(null)
    setApplying(true)
    setError(null)
    setSuccess(null)

    try {
      const res = await api.applyOta(restartNow)
      if (res.status === 'ok') {
        setSuccess(restartNow 
          ? '更新已应用，系统即将重启...' 
          : '更新已应用，请手动重启服务生效'
        )
        setUploadResult(null)
        await loadStatus()
      } else {
        setError(res.message || '应用更新失败')
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setApplying(false)
    }
  }

  const handleCancel = async () => {
    setConfirmDialog(null)
    setError(null)
    setSuccess(null)

    try {
      const res = await api.cancelOta()
      if (res.status === 'ok') {
        setSuccess('已取消待安装的更新')
        setUploadResult(null)
        await loadStatus()
      } else {
        setError(res.message || '取消失败')
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    }
  }

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
      <Box display="flex" justifyContent="space-between" alignItems="center" mb={3}>
        <Box>
          <Typography variant="h4" gutterBottom fontWeight={600}>
            OTA 更新
          </Typography>
          <Typography variant="body2" color="text.secondary">
            上传并安装系统更新包
          </Typography>
        </Box>
        <Button
          variant="outlined"
          startIcon={<Refresh />}
          onClick={() => void loadStatus()}
          disabled={loading}
        >
          刷新状态
        </Button>
      </Box>

      {/* 错误/成功提示 */}
      {error && (
        <Alert severity="error" sx={{ mb: 2 }} onClose={() => setError(null)}>
          {error}
        </Alert>
      )}
      {success && (
        <Alert severity="success" sx={{ mb: 2 }} onClose={() => setSuccess(null)}>
          {success}
        </Alert>
      )}

      <Stack spacing={3}>
        {/* 当前版本信息 */}
        <Card>
          <CardContent>
            <Box display="flex" alignItems="center" gap={1} mb={2}>
              <Info color="primary" />
              <Typography variant="h6">当前版本</Typography>
            </Box>
            <TableContainer>
              <Table size="small">
                <TableBody>
                  <TableRow>
                    <TableCell component="th" sx={{ width: 150 }}>版本号</TableCell>
                    <TableCell>
                      <Chip label={status?.current_version || 'N/A'} color="primary" size="small" />
                    </TableCell>
                  </TableRow>
                  <TableRow>
                    <TableCell component="th">Commit</TableCell>
                    <TableCell sx={{ fontFamily: 'monospace' }}>
                      {status?.current_commit || 'N/A'}
                    </TableCell>
                  </TableRow>
                </TableBody>
              </Table>
            </TableContainer>
          </CardContent>
        </Card>

        {/* 待安装更新 */}
        {status?.pending_update && status.pending_meta && (
          <Card sx={{ borderColor: 'warning.main', borderWidth: 2, borderStyle: 'solid' }}>
            <CardContent>
              <Box display="flex" alignItems="center" gap={1} mb={2}>
                <Warning color="warning" />
                <Typography variant="h6">待安装更新</Typography>
                <Chip 
                  label={status.pending_meta.version} 
                  color="warning" 
                  size="small" 
                  sx={{ ml: 1 }}
                />
              </Box>
              <TableContainer>
                <Table size="small">
                  <TableBody>
                    <TableRow>
                      <TableCell component="th" sx={{ width: 150 }}>版本号</TableCell>
                      <TableCell>{status.pending_meta.version}</TableCell>
                    </TableRow>
                    <TableRow>
                      <TableCell component="th">Commit</TableCell>
                      <TableCell sx={{ fontFamily: 'monospace' }}>{status.pending_meta.commit}</TableCell>
                    </TableRow>
                    <TableRow>
                      <TableCell component="th">构建时间</TableCell>
                      <TableCell>{status.pending_meta.build_time}</TableCell>
                    </TableRow>
                    <TableRow>
                      <TableCell component="th">架构</TableCell>
                      <TableCell>{status.pending_meta.arch}</TableCell>
                    </TableRow>
                  </TableBody>
                </Table>
              </TableContainer>
              <Divider sx={{ my: 2 }} />
              <Stack direction="row" spacing={2}>
                <Button
                  variant="contained"
                  color="success"
                  startIcon={<SystemUpdateAlt />}
                  onClick={() => setConfirmDialog('apply')}
                  disabled={applying}
                >
                  {applying ? <CircularProgress size={20} /> : '应用更新'}
                </Button>
                <Button
                  variant="outlined"
                  color="error"
                  startIcon={<Cancel />}
                  onClick={() => setConfirmDialog('cancel')}
                >
                  取消更新
                </Button>
              </Stack>
            </CardContent>
          </Card>
        )}

        {/* 上传新版本 */}
        <Card>
          <CardContent>
            <Box display="flex" alignItems="center" gap={1} mb={2}>
              <CloudUpload color="primary" />
              <Typography variant="h6">上传更新包</Typography>
            </Box>
            
            <Alert severity="info" sx={{ mb: 2 }}>
              <AlertTitle>OTA 更新包格式</AlertTitle>
              请上传 <code>.tar.gz</code> 格式的 OTA 更新包. 错误的包会导致系统无法启动.
            </Alert>

            <input
              ref={fileInputRef}
              type="file"
              accept=".gz,.tgz,.zip,application/gzip,application/x-gzip,application/x-tar,application/zip"
              style={{ display: 'none' }}
              onChange={(e) => void handleFileSelect(e)}
            />
            
            <Button
              variant="contained"
              startIcon={uploading ? <CircularProgress size={20} color="inherit" /> : <CloudUpload />}
              onClick={() => fileInputRef.current?.click()}
              disabled={uploading}
              size="large"
            >
              {uploading ? '上传中...' : '选择更新包'}
            </Button>

            {uploading && (
              <Box sx={{ mt: 2 }}>
                <LinearProgress />
              </Box>
            )}
          </CardContent>
        </Card>

        {/* 上传结果 */}
        {uploadResult && (
          <Card>
            <CardContent>
              <Box display="flex" alignItems="center" gap={1} mb={2}>
                {uploadResult.validation.valid ? (
                  <CheckCircle color="success" />
                ) : (
                  <ErrorIcon color="error" />
                )}
                <Typography variant="h6">
                  验证结果
                </Typography>
                <Chip 
                  label={uploadResult.validation.valid ? '通过' : '失败'}
                  color={uploadResult.validation.valid ? 'success' : 'error'}
                  size="small"
                />
              </Box>
              
              <TableContainer component={Paper} variant="outlined">
                <Table size="small">
                  <TableBody>
                    <TableRow>
                      <TableCell component="th" sx={{ width: 180 }}>版本号</TableCell>
                      <TableCell>{uploadResult.meta.version}</TableCell>
                      <TableCell align="right">
                        {uploadResult.validation.is_newer ? (
                          <Chip label="新版本" color="success" size="small" />
                        ) : (
                          <Chip label="旧版本或相同" color="warning" size="small" />
                        )}
                      </TableCell>
                    </TableRow>
                    <TableRow>
                      <TableCell component="th">Commit</TableCell>
                      <TableCell sx={{ fontFamily: 'monospace' }} colSpan={2}>
                        {uploadResult.meta.commit}
                      </TableCell>
                    </TableRow>
                    <TableRow>
                      <TableCell component="th">构建时间</TableCell>
                      <TableCell colSpan={2}>{uploadResult.meta.build_time}</TableCell>
                    </TableRow>
                    <TableRow>
                      <TableCell component="th">二进制 MD5</TableCell>
                      <TableCell sx={{ fontFamily: 'monospace', fontSize: '0.75rem' }}>
                        {uploadResult.meta.binary_md5}
                      </TableCell>
                      <TableCell align="right">
                        {uploadResult.validation.binary_md5_match ? (
                          <CheckCircle color="success" fontSize="small" />
                        ) : (
                          <ErrorIcon color="error" fontSize="small" />
                        )}
                      </TableCell>
                    </TableRow>
                    <TableRow>
                      <TableCell component="th">前端 MD5</TableCell>
                      <TableCell sx={{ fontFamily: 'monospace', fontSize: '0.75rem' }}>
                        {uploadResult.meta.frontend_md5}
                      </TableCell>
                      <TableCell align="right">
                        {uploadResult.validation.frontend_md5_match ? (
                          <CheckCircle color="success" fontSize="small" />
                        ) : (
                          <ErrorIcon color="error" fontSize="small" />
                        )}
                      </TableCell>
                    </TableRow>
                    <TableRow>
                      <TableCell component="th">架构</TableCell>
                      <TableCell>{uploadResult.meta.arch}</TableCell>
                      <TableCell align="right">
                        {uploadResult.validation.arch_match ? (
                          <CheckCircle color="success" fontSize="small" />
                        ) : (
                          <ErrorIcon color="error" fontSize="small" />
                        )}
                      </TableCell>
                    </TableRow>
                  </TableBody>
                </Table>
              </TableContainer>

              {uploadResult.validation.error && (
                <Alert severity="error" sx={{ mt: 2 }}>
                  {uploadResult.validation.error}
                </Alert>
              )}
            </CardContent>
          </Card>
        )}
      </Stack>

      {/* 确认对话框 - 应用更新 */}
      <Dialog open={confirmDialog === 'apply'} onClose={() => setConfirmDialog(null)}>
        <DialogTitle>确认应用更新</DialogTitle>
        <DialogContent>
          <DialogContentText>
            确定要应用此更新吗？更新将替换当前的后端程序和前端文件。
          </DialogContentText>
          <Alert severity="warning" sx={{ mt: 2 }}>
            建议在应用更新后重启服务以确保更新完全生效。
          </Alert>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setConfirmDialog(null)}>取消</Button>
          <Button 
            onClick={() => void handleApply(false)} 
            variant="outlined"
            color="primary"
          >
            仅应用（稍后重启）
          </Button>
          <Button 
            onClick={() => void handleApply(true)} 
            variant="contained"
            color="success"
            startIcon={<RestartAlt />}
          >
            应用并重启
          </Button>
        </DialogActions>
      </Dialog>

      {/* 确认对话框 - 取消更新 */}
      <Dialog open={confirmDialog === 'cancel'} onClose={() => setConfirmDialog(null)}>
        <DialogTitle>确认取消更新</DialogTitle>
        <DialogContent>
          <DialogContentText>
            确定要取消待安装的更新吗？这将删除已上传的更新包。
          </DialogContentText>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setConfirmDialog(null)}>返回</Button>
          <Button 
            onClick={() => void handleCancel()} 
            variant="contained"
            color="error"
          >
            确认取消
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  )
}

