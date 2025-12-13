/*
 * @Author: 1orz cloudorzi@gmail.com
 * @Date: 2025-11-22 10:30:41
 * @LastEditors: 1orz cloudorzi@gmail.com
 * @LastEditTime: 2025-12-13 12:43:54
 * @FilePath: /udx710-backend/frontend/src/contexts/RefreshContext.tsx
 * @Description: 
 * 
 * Copyright (c) 2025 by 1orz, All Rights Reserved. 
 */
import { createContext, useContext } from 'react'

// 刷新间隔 Context
interface RefreshContextType {
  refreshInterval: number
  setRefreshInterval: (interval: number) => void
  refreshKey: number
  triggerRefresh: () => void
}

export const RefreshContext = createContext<RefreshContextType>({
  refreshInterval: 1000,
  setRefreshInterval: () => {},
  refreshKey: 0,
  triggerRefresh: () => {},
})

export const useRefreshInterval = () => useContext(RefreshContext)
