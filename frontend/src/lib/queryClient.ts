/*
 * @Author: 1orz cloudorzi@gmail.com
 * @Date: 2025-12-10 10:28:41
 * @LastEditors: 1orz cloudorzi@gmail.com
 * @LastEditTime: 2025-12-13 12:44:05
 * @FilePath: /udx710-backend/frontend/src/lib/queryClient.ts
 * @Description: 
 * 
 * Copyright (c) 2025 by 1orz, All Rights Reserved. 
 */
import { QueryClient } from '@tanstack/react-query'

export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      // 默认缓存时间 5 分钟
      staleTime: 5 * 60 * 1000,
      // 失败时自动重试 1 次
      retry: 1,
      // 窗口重新聚焦时不自动刷新（因为我们有自己的刷新机制）
      refetchOnWindowFocus: false,
    },
  },
})
