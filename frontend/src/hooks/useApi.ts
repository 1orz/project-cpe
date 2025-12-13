/*
 * @Author: 1orz cloudorzi@gmail.com
 * @Date: 2025-11-22 10:30:41
 * @LastEditors: 1orz cloudorzi@gmail.com
 * @LastEditTime: 2025-12-13 12:44:02
 * @FilePath: /udx710-backend/frontend/src/hooks/useApi.ts
 * @Description: 
 * 
 * Copyright (c) 2025 by 1orz, All Rights Reserved. 
 */
import { useState, useEffect, useCallback } from 'react'

interface UseApiOptions {
  autoFetch?: boolean
  refreshInterval?: number // 毫秒
}

interface UseApiReturn<T> {
  data: T | null
  loading: boolean
  error: Error | null
  refresh: () => Promise<void>
}

export function useApi<T>(
  fetcher: () => Promise<T>,
  options: UseApiOptions = {}
): UseApiReturn<T> {
  const { autoFetch = true, refreshInterval } = options
  const [data, setData] = useState<T | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<Error | null>(null)

  const refresh = useCallback(async () => {
    setLoading(true)
    setError(null)
    try {
      const result = await fetcher()
      setData(result)
    } catch (err) {
      setError(err instanceof Error ? err : new Error(String(err)))
    } finally {
      setLoading(false)
    }
  }, [fetcher])

  useEffect(() => {
    if (autoFetch) {
      void refresh()
    }
  }, [autoFetch, refresh])

  useEffect(() => {
    if (refreshInterval && refreshInterval > 0) {
      const interval = setInterval(refresh, refreshInterval)
      return () => clearInterval(interval)
    }
  }, [refreshInterval, refresh])

  return { data, loading, error, refresh }
}

