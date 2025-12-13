/*
 * @Author: 1orz cloudorzi@gmail.com
 * @Date: 2025-12-10 10:18:14
 * @LastEditors: 1orz cloudorzi@gmail.com
 * @LastEditTime: 2025-12-13 12:44:30
 * @FilePath: /udx710-backend/frontend/src/pages/Dashboard/components/SimCardInfo.tsx
 * @Description: 
 * 
 * Copyright (c) 2025 by 1orz, All Rights Reserved. 
 */
import { useState } from 'react'
import { Box, Card, CardContent, Typography, Stack, Chip, IconButton, Tooltip } from '@mui/material'
import { SimCard, Visibility, VisibilityOff, Phone, Sms, Language } from '@mui/icons-material'
import { getSensitiveStyle } from '../utils'
import type { SimInfo } from '@/api/types'

interface SimCardInfoProps {
  simInfo: SimInfo | null
}

export function SimCardInfo({ simInfo }: SimCardInfoProps) {
  const [showInfo, setShowInfo] = useState(false)

  return (
    <Card sx={{ height: '100%' }}>
      <CardContent>
        <Box display="flex" alignItems="center" gap={1} mb={2}>
          <SimCard color="primary" />
          <Typography variant="subtitle2" color="text.secondary">SIM 卡信息</Typography>
          <Chip
            label={simInfo?.present ? '已插入' : '未插入'}
            color={simInfo?.present ? 'success' : 'error'}
            size="small"
            variant="outlined"
            sx={{ ml: 'auto' }}
          />
          <Tooltip title={showInfo ? '隐藏敏感信息' : '显示完整信息'}>
            <IconButton size="small" onClick={() => setShowInfo(!showInfo)}>
              {showInfo ? <VisibilityOff fontSize="small" /> : <Visibility fontSize="small" />}
            </IconButton>
          </Tooltip>
        </Box>
        <Stack spacing={1.5}>
          {/* ICCID */}
          <Box display="flex" justifyContent="space-between" alignItems="center">
            <Typography variant="caption" color="text.secondary">ICCID</Typography>
            <Typography variant="body2" fontWeight="medium" fontFamily="monospace" sx={getSensitiveStyle(showInfo)}>
              {simInfo?.iccid || 'N/A'}
            </Typography>
          </Box>
          {/* IMSI */}
          <Box display="flex" justifyContent="space-between" alignItems="center">
            <Typography variant="caption" color="text.secondary">IMSI</Typography>
            <Typography variant="body2" fontWeight="medium" fontFamily="monospace" sx={getSensitiveStyle(showInfo)}>
              {simInfo?.imsi || 'N/A'}
            </Typography>
          </Box>
          {/* 手机号码 */}
          <Box display="flex" justifyContent="space-between" alignItems="center">
            <Box display="flex" alignItems="center" gap={0.5}>
              <Phone fontSize="small" color="action" />
              <Typography variant="caption" color="text.secondary">手机号码</Typography>
            </Box>
            <Typography variant="body2" fontWeight="medium" fontFamily="monospace" sx={getSensitiveStyle(showInfo)}>
              {simInfo?.phone_numbers?.length ? simInfo.phone_numbers[0] : 'N/A'}
            </Typography>
          </Box>
          {/* 短信中心 */}
          <Box display="flex" justifyContent="space-between" alignItems="center">
            <Box display="flex" alignItems="center" gap={0.5}>
              <Sms fontSize="small" color="action" />
              <Typography variant="caption" color="text.secondary">短信中心</Typography>
            </Box>
            <Typography variant="body2" fontWeight="medium" fontFamily="monospace" sx={getSensitiveStyle(showInfo)}>
              {simInfo?.sms_center || 'N/A'}
            </Typography>
          </Box>
          {/* MCC/MNC */}
          <Box display="flex" justifyContent="space-between" alignItems="center">
            <Typography variant="caption" color="text.secondary">MCC/MNC</Typography>
            <Typography variant="body2" fontWeight="medium" fontFamily="monospace">
              {simInfo?.mcc || '?'}/{simInfo?.mnc || '?'}
            </Typography>
          </Box>
          {/* 首选语言 */}
          {simInfo?.preferred_languages && simInfo.preferred_languages.length > 0 && (
            <Box display="flex" justifyContent="space-between" alignItems="center">
              <Box display="flex" alignItems="center" gap={0.5}>
                <Language fontSize="small" color="action" />
                <Typography variant="caption" color="text.secondary">语言</Typography>
              </Box>
              <Stack direction="row" spacing={0.5}>
                {simInfo.preferred_languages.slice(0, 3).map((lang, idx) => (
                  <Chip key={idx} label={lang.toUpperCase()} size="small" variant="outlined" />
                ))}
              </Stack>
            </Box>
          )}
        </Stack>
      </CardContent>
    </Card>
  )
}
