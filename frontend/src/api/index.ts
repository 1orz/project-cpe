/*
 * @Author: 1orz cloudorzi@gmail.com
 * @Date: 2025-12-10 09:19:05
 * @LastEditors: 1orz cloudorzi@gmail.com
 * @LastEditTime: 2025-12-13 12:42:03
 * @FilePath: /udx710-backend/frontend/src/api/index.ts
 * @Description: 
 * 
 * Copyright (c) 2025 by 1orz, All Rights Reserved. 
 */
import type {
  ApiResponse,
  DeviceInfo,
  SimInfo,
  NetworkInfo,
  CellsResponse,
  QosInfo,
  DataConnectionStatus,
  RoamingResponse,
  RoamingRequest,
  UsbModeResponse,
  AtCommandRequest,
  SetUsbModeRequest,
  DataConnectionRequest,
  SystemStatsResponse,
  CpuInfo,
  AirplaneModeRequest,
  AirplaneModeResponse,
  CellLocationResponse,
  NetworkInterfacesResponse,
  RadioMode,
  RadioModeRequest,
  RadioModeResponse,
  BandLockStatus,
  BandLockRequest,
  CellLockStatusResponse,
  CellLockRequest,
  CellLockResult,
  CallInfo,
  CallListResponse,
  MakeCallRequest,
  HangupCallRequest,
  SmsMessage,
  SendSmsRequest,
  SmsListRequest,
  SmsConversationRequest,
  SmsStats,
  ImeisvResponse,
  SignalStrengthResponse,
  NitzTimeResponse,
  ImsStatusResponse,
  CallVolumeResponse,
  SetCallVolumeRequest,
  VoicemailStatusResponse,
  OperatorListResponse,
  ManualRegisterRequest,
  CallForwardingResponse,
  SetCallForwardingRequest,
  CallSettingsResponse,
  SetCallSettingRequest,
  SimSlotResponse,
  SwitchSimSlotRequest,
  UsbAdvanceRequest,
  ApnListResponse,
  SetApnRequest,
  ConnectivityCheckResponse,
  CallHistoryResponse,
  WebhookConfig,
  WebhookTestResponse,
  OtaStatusResponse,
  OtaUploadResponse,
} from './types'

// API 基础配置
const API_BASE = '/api'

// 通用请求函数
async function request<T>(
  url: string,
  options: RequestInit & { returnText?: boolean } = {}
): Promise<T> {
  const { returnText, ...fetchOptions } = options
  
  const response = await fetch(`${API_BASE}${url}`, {
    headers: {
      'Content-Type': 'application/json',
      ...fetchOptions.headers,
    },
    ...fetchOptions,
  })

  if (!response.ok) {
    throw new Error(`HTTP error! status: ${response.status}`)
  }

  if (returnText) {
    return (await response.text()) as T
  }

  return await response.json() as T
}

// API 类
class UDX710API {
  // 健康检查
  async health() {
    return request<{ status: string; message: string; version: string }>('/health')
  }

  // 设备信息（IMEI、制造商、型号等）
  async getDeviceInfo() {
    return request<ApiResponse<DeviceInfo>>('/device')
  }

  // SIM 卡信息（ICCID、IMSI、手机号等）
  async getSimInfo() {
    return request<ApiResponse<SimInfo>>('/sim')
  }

  // 网络信息
  async getNetworkInfo() {
    return request<ApiResponse<NetworkInfo>>('/network')
  }

  // 小区信息
  async getCellsInfo() {
    return request<ApiResponse<CellsResponse>>('/cells')
  }

  // QoS 信息
  async getQosInfo() {
    return request<ApiResponse<QosInfo>>('/qos')
  }

  // 获取数据连接状态
  async getDataStatus() {
    return request<ApiResponse<DataConnectionStatus>>('/data')
  }

  // 设置数据连接状态
  async setDataStatus(active: boolean) {
    const body: DataConnectionRequest = { active }
    return request<ApiResponse<DataConnectionStatus>>('/data', {
      method: 'POST',
      body: JSON.stringify(body),
    })
  }

  // 获取漫游状态
  async getRoamingStatus() {
    return request<ApiResponse<RoamingResponse>>('/roaming')
  }

  // 设置漫游开关
  async setRoamingAllowed(allowed: boolean) {
    const body: RoamingRequest = { allowed }
    return request<ApiResponse<RoamingResponse>>('/roaming', {
      method: 'POST',
      body: JSON.stringify(body),
    })
  }

  // 获取飞行模式状态
  async getAirplaneMode() {
    return request<ApiResponse<AirplaneModeResponse>>('/airplane-mode')
  }

  // 设置飞行模式
  async setAirplaneMode(enabled: boolean) {
    const body: AirplaneModeRequest = { enabled }
    return request<ApiResponse<AirplaneModeResponse>>('/airplane-mode', {
      method: 'POST',
      body: JSON.stringify(body),
    })
  }

  // 获取 USB 模式
  async getUsbMode() {
    return request<ApiResponse<UsbModeResponse>>('/usb-mode')
  }

  // 设置 USB 模式（写入配置文件，重启后生效）
  async setUsbMode(mode: number, permanent: boolean = false) {
    const body: SetUsbModeRequest = { mode, permanent }
    return request<ApiResponse<void>>('/usb-mode', {
      method: 'POST',
      body: JSON.stringify(body),
    })
  }

  // 系统重启
  async systemReboot(delaySeconds: number = 3) {
    const body = { delay_seconds: delaySeconds }
    return request<ApiResponse<Record<string, never>>>('/system/reboot', {
      method: 'POST',
      body: JSON.stringify(body),
    })
  }

  // 发送 AT 指令
  async sendAtCommand(cmd: string) {
    const body: AtCommandRequest = { cmd }
    return request<string>('/at', {
      method: 'POST',
      body: JSON.stringify(body),
      returnText: true,
    })
  }

  // 获取实时网速信息
  // 获取 CPU 信息
  async getCpuInfo() {
    return request<ApiResponse<CpuInfo>>('/stats/cpu')
  }

  // 获取综合系统统计（网速+内存+运行时间+系统信息）
  async getSystemStats() {
    return request<ApiResponse<SystemStatsResponse>>('/stats')
  }

  // 获取基站定位参数（用于第三方定位API）
  async getCellLocationInfo() {
    return request<ApiResponse<CellLocationResponse>>('/location/cell-info')
  }

  // 获取所有网络接口详细信息
  async getNetworkInterfaces() {
    return request<ApiResponse<NetworkInterfacesResponse>>('/network/interfaces')
  }

  // 获取当前射频模式
  async getRadioMode() {
    return request<ApiResponse<RadioModeResponse>>('/radio-mode')
  }

  // 设置射频模式
  async setRadioMode(mode: RadioMode) {
    const body: RadioModeRequest = { mode }
    return request<ApiResponse<Record<string, never>>>('/radio-mode', {
      method: 'POST',
      body: JSON.stringify(body),
    })
  }

  // 获取频段锁定状态
  async getBandLockStatus() {
    return request<ApiResponse<BandLockStatus>>('/band-lock')
  }

  // 设置频段锁定
  async setBandLock(config: BandLockRequest) {
    return request<ApiResponse<Record<string, never>>>('/band-lock', {
      method: 'POST',
      body: JSON.stringify(config),
    })
  }

  // ========== 小区锁定功能 ==========

  // 获取小区锁定状态
  async getCellLockStatus() {
    return request<ApiResponse<CellLockStatusResponse>>('/cell-lock')
  }

  // 设置小区锁定
  async setCellLock(config: CellLockRequest) {
    return request<ApiResponse<CellLockResult>>('/cell-lock', {
      method: 'POST',
      body: JSON.stringify(config),
    })
  }

  // 解锁所有小区
  async unlockAllCells() {
    return request<ApiResponse<CellLockResult>>('/cell-lock/unlock-all', {
      method: 'POST',
      body: JSON.stringify({}),
    })
  }

  // ========== 电话功能 ==========

  // 获取当前通话列表
  async getCalls() {
    return request<ApiResponse<CallListResponse>>('/calls')
  }

  // 拨打电话
  async dialCall(phoneNumber: string) {
    const body: MakeCallRequest = { phone_number: phoneNumber }
    return request<ApiResponse<CallInfo>>('/call/dial', {
      method: 'POST',
      body: JSON.stringify(body),
    })
  }

  // 挂断指定通话
  async hangupCall(path: string) {
    const body: HangupCallRequest = { path }
    return request<ApiResponse<Record<string, never>>>('/call/hangup', {
      method: 'POST',
      body: JSON.stringify(body),
    })
  }

  // 挂断所有通话
  async hangupAllCalls() {
    return request<ApiResponse<{ count: number }>>('/call/hangup-all', {
      method: 'POST',
    })
  }

  // 接听来电
  async answerCall(path: string) {
    const body: HangupCallRequest = { path }
    return request<ApiResponse<Record<string, never>>>('/call/answer', {
      method: 'POST',
      body: JSON.stringify(body),
    })
  }

  // ========== 短信功能 ==========

  // 发送短信
  async sendSms(phoneNumber: string, content: string) {
    const body: SendSmsRequest = { phone_number: phoneNumber, content }
    return request<ApiResponse<{ message_path: string; db_id: number }>>('/sms/send', {
      method: 'POST',
      body: JSON.stringify(body),
    })
  }

  // 获取短信列表（分页）
  async getSmsList(params?: SmsListRequest) {
    const query = new URLSearchParams()
    if (params?.limit) query.append('limit', params.limit.toString())
    if (params?.offset) query.append('offset', params.offset.toString())
    const queryStr = query.toString() ? `?${query.toString()}` : ''
    return request<ApiResponse<SmsMessage[]>>(`/sms/list${queryStr}`)
  }

  // 获取与特定号码的对话历史
  async getSmsConversation(params: SmsConversationRequest) {
    const query = new URLSearchParams()
    query.append('phone_number', params.phone_number)
    if (params.limit) query.append('limit', params.limit.toString())
    return request<ApiResponse<SmsMessage[]>>(`/sms/conversation?${query.toString()}`)
  }

  // 获取短信统计
  async getSmsStats() {
    return request<ApiResponse<SmsStats>>('/sms/stats')
  }

  // 清空所有短信
  async clearAllSms() {
    return request<ApiResponse<Record<string, never>>>('/sms/clear', {
      method: 'POST',
    })
  }

  // ========== 新增功能 ==========

  // 获取 IMEISV（软件版本号）
  async getImeisv() {
    return request<ApiResponse<ImeisvResponse>>('/device/imeisv')
  }

  // 获取信号强度详细信息
  async getSignalStrength() {
    return request<ApiResponse<SignalStrengthResponse>>('/network/signal-strength')
  }

  // 获取 NITZ 网络时间
  async getNitzTime() {
    return request<ApiResponse<NitzTimeResponse>>('/network/nitz')
  }

  // 获取 IMS 状态
  async getImsStatus() {
    return request<ApiResponse<ImsStatusResponse>>('/ims/status')
  }

  // 获取通话音量
  async getCallVolume() {
    return request<ApiResponse<CallVolumeResponse>>('/call/volume')
  }

  // 设置通话音量
  async setCallVolume(volume: SetCallVolumeRequest) {
    return request<ApiResponse<Record<string, never>>>('/call/volume', {
      method: 'POST',
      body: JSON.stringify(volume),
    })
  }

  // 获取语音留言状态
  async getVoicemailStatus() {
    return request<ApiResponse<VoicemailStatusResponse>>('/voicemail/status')
  }

  // 获取运营商列表（快速）
  async getOperators() {
    return request<ApiResponse<OperatorListResponse>>('/network/operators')
  }

  // 扫描所有运营商（慢，120秒）
  async scanOperators() {
    return request<ApiResponse<OperatorListResponse>>('/network/operators/scan')
  }

  // 手动注册到指定运营商
  async registerOperatorManual(mccmnc: string) {
    const body: ManualRegisterRequest = { mccmnc }
    return request<ApiResponse<Record<string, never>>>('/network/register-manual', {
      method: 'POST',
      body: JSON.stringify(body),
    })
  }

  // 自动注册运营商
  async registerOperatorAuto() {
    return request<ApiResponse<Record<string, never>>>('/network/register-auto', {
      method: 'POST',
    })
  }

  // 获取呼叫转移设置
  async getCallForwarding() {
    return request<ApiResponse<CallForwardingResponse>>('/call/forwarding')
  }

  // 设置呼叫转移
  async setCallForwarding(forwarding: SetCallForwardingRequest) {
    return request<ApiResponse<Record<string, never>>>('/call/forwarding', {
      method: 'POST',
      body: JSON.stringify(forwarding),
    })
  }

  // 获取通话设置
  async getCallSettings() {
    return request<ApiResponse<CallSettingsResponse>>('/call/settings')
  }

  // 设置通话设置
  async setCallSettings(setting: SetCallSettingRequest) {
    return request<ApiResponse<Record<string, never>>>('/call/settings', {
      method: 'POST',
      body: JSON.stringify(setting),
    })
  }

  // ========== SIM 管理功能 ==========

  // 获取 SIM 卡槽状态
  async getSimSlot() {
    return request<ApiResponse<SimSlotResponse>>('/sim/slot')
  }

  // 切换 SIM 卡槽
  async switchSimSlot(slot: number) {
    const body: SwitchSimSlotRequest = { slot }
    return request<ApiResponse<{ response: string }>>('/sim/slot/switch', {
      method: 'POST',
      body: JSON.stringify(body),
    })
  }

  // ========== USB 高级功能 ==========

  // USB 热切换（立即生效，无需重启）
  async setUsbModeAdvance(mode: number) {
    const body: UsbAdvanceRequest = { mode }
    return request<ApiResponse<Record<string, never>>>('/usb-advance', {
      method: 'POST',
      body: JSON.stringify(body),
    })
  }

  // ========== APN 管理功能 ==========

  // 获取 APN 列表
  async getApnList() {
    return request<ApiResponse<ApnListResponse>>('/apn')
  }

  // 设置 APN 配置
  async setApn(config: SetApnRequest) {
    return request<ApiResponse<Record<string, unknown>>>('/apn', {
      method: 'POST',
      body: JSON.stringify(config),
    })
  }

  // ========== 联网检测功能 ==========

  // 获取联网检测结果 (IPv4/IPv6 ping)
  async getConnectivity() {
    return request<ApiResponse<ConnectivityCheckResponse>>('/connectivity')
  }

  // ========== 通话记录功能 ==========

  // 获取通话记录列表
  async getCallHistory(limit = 50, offset = 0) {
    return request<ApiResponse<CallHistoryResponse>>(`/call/history?limit=${limit}&offset=${offset}`)
  }

  // 删除单条通话记录
  async deleteCallRecord(id: number) {
    return request<ApiResponse<Record<string, unknown>>>(`/call/history/${id}`, {
      method: 'DELETE',
    })
  }

  // 清空所有通话记录
  async clearCallHistory() {
    return request<ApiResponse<Record<string, unknown>>>('/call/history/clear', {
      method: 'POST',
    })
  }

  // ========== Webhook 配置功能 ==========

  // 获取 Webhook 配置
  async getWebhookConfig() {
    return request<ApiResponse<WebhookConfig>>('/webhook/config')
  }

  // 设置 Webhook 配置
  async setWebhookConfig(config: WebhookConfig) {
    return request<ApiResponse<Record<string, unknown>>>('/webhook/config', {
      method: 'POST',
      body: JSON.stringify(config),
    })
  }

  // 测试 Webhook 连接
  async testWebhook() {
    return request<ApiResponse<WebhookTestResponse>>('/webhook/test', {
      method: 'POST',
    })
  }

  // ========== OTA 更新 ==========

  // 获取 OTA 状态
  async getOtaStatus() {
    return request<ApiResponse<OtaStatusResponse>>('/ota/status')
  }

  // 上传 OTA 更新包
  async uploadOta(file: File) {
    const response = await fetch(`${API_BASE}/ota/upload`, {
      method: 'POST',
      body: file,
      headers: {
        'Content-Type': 'application/octet-stream',
      },
    })
    return response.json() as Promise<ApiResponse<OtaUploadResponse>>
  }

  // 应用 OTA 更新
  async applyOta(restartNow: boolean = false) {
    return request<ApiResponse<{ applied: boolean }>>('/ota/apply', {
      method: 'POST',
      body: JSON.stringify({ restart_now: restartNow }),
    })
  }

  // 取消 OTA 更新
  async cancelOta() {
    return request<ApiResponse<Record<string, unknown>>>('/ota/cancel', {
      method: 'POST',
    })
  }

}

// 导出单例
export const api = new UDX710API()

// 导出类型
export * from './types'

