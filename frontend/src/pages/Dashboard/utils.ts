/*
 * @Author: 1orz cloudorzi@gmail.com
 * @Date: 2025-12-10 10:16:14
 * @LastEditors: 1orz cloudorzi@gmail.com
 * @LastEditTime: 2025-12-13 12:44:45
 * @FilePath: /udx710-backend/frontend/src/pages/Dashboard/utils.ts
 * @Description: 
 * 
 * Copyright (c) 2025 by 1orz, All Rights Reserved. 
 */
// Dashboard 工具函数

export const getSignalColor = (strength: number) => {
  if (strength >= 75) return 'success'
  if (strength >= 50) return 'primary'
  if (strength >= 25) return 'warning'
  return 'error'
}

export const getTempColor = (temp: number) => {
  if (temp >= 70) return 'error'
  if (temp >= 60) return 'warning'
  return 'success'
}

export const getMemoryColor = (percent: number) => {
  if (percent >= 90) return 'error'
  if (percent >= 70) return 'warning'
  return 'success'
}

export const getCpuColor = (percent: number) => {
  if (percent >= 90) return 'error'
  if (percent >= 70) return 'warning'
  return 'success'
}

export const formatBytes = (bytes: number, decimals = 1): string => {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(decimals)) + ' ' + sizes[i]
}

export const formatSpeed = (bytesPerSec: number): string => {
  if (bytesPerSec === 0) return '0 B/s'
  const k = 1024
  const sizes = ['B/s', 'KB/s', 'MB/s', 'GB/s']
  const i = Math.floor(Math.log(bytesPerSec) / Math.log(k))
  return parseFloat((bytesPerSec / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i]
}

export const convertSignalValue = (value: string | number | undefined): number | null => {
  if (value === undefined || value === null) return null
  const numValue = typeof value === 'string' ? parseFloat(value) : value
  if (isNaN(numValue)) return null
  return numValue / 100
}

export const formatSignalValue = (value: string | number | undefined): string => {
  const converted = convertSignalValue(value)
  if (converted === null) return '-'
  return converted.toFixed(1)
}

export const getSignalChipColor = (rsrp?: string | number) => {
  const value = convertSignalValue(rsrp)
  if (value === null) return 'default'
  if (value >= -80) return 'success'
  if (value >= -100) return 'primary'
  if (value >= -110) return 'warning'
  return 'error'
}

// 根据不同功能块返回对应的敏感信息样式
export const getSensitiveStyle = (show: boolean) => ({
  filter: show ? 'none' : 'blur(5px)',
  transition: 'filter 0.3s ease',
  userSelect: show ? 'auto' as const : 'none' as const,
})
