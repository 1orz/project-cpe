/*
 * @Author: 1orz cloudorzi@gmail.com
 * @Date: 2025-12-10 09:19:05
 * @LastEditors: 1orz cloudorzi@gmail.com
 * @LastEditTime: 2025-12-13 12:46:10
 * @FilePath: /udx710-backend/backend/src/models.rs
 * @Description: 
 * 
 * Copyright (c) 2025 by 1orz, All Rights Reserved. 
 */
//! 数据模型定义
//! 
//! 包含所有API的请求和响应数据结构

use serde::{Deserialize, Serialize};

/// 统一的 API 响应结构
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    /// 状态：ok 或 error
    pub status: String,
    /// 消息说明
    pub message: String,
    /// 响应数据（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    /// Create success response with custom message
    pub fn success_with_message(message: impl Into<String>, data: T) -> Self {
        Self {
            status: "ok".to_string(),
            message: message.into(),
            data: Some(data),
        }
    }
}

impl<T> ApiResponse<T>
where
    T: Default,
{
    /// Create error response
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            status: "error".to_string(),
            message: message.into(),
            data: None,
        }
    }
}

/// AT 指令请求
#[derive(Debug, Deserialize)]
pub struct AtCommandRequest {
    /// AT 指令内容
    pub cmd: String,
}

/// 主服务小区信息
#[derive(Debug, Default, Serialize, Clone)]
pub struct ServingCell {
    /// 网络制式：nr, lte, unknown
    pub tech: String,
    /// 小区ID
    pub cell_id: u32,
    /// 跟踪区域码
    pub tac: u32,
}

/// 小区详细信息
/// 
/// 所有信号强度字段均为原始值（×100），前端需要除以100得到实际dBm/dB值
#[derive(Debug, Default, Serialize, Clone)]
pub struct CellInfo {
    /// 是否为主服务小区
    pub is_serving: bool,
    /// 网络制式：nr, lte
    pub tech: String,
    /// 频段编号
    pub band: String,
    /// 绝对频点号（ARFCN）
    pub arfcn: String,
    /// 物理小区标识（PCI）
    pub pci: String,
    /// 参考信号接收功率（RSRP）原始值×100，实际单位dBm，前端需除以100
    pub rsrp: String,
    /// 参考信号接收质量（RSRQ）原始值×100，实际单位dB，前端需除以100  
    pub rsrq: String,
    /// 信噪比（SINR）原始值×100，实际单位dB，前端需除以100
    pub sinr: String,
}

/// 小区信息响应
#[derive(Debug, Serialize, Default)]
pub struct CellsResponse {
    /// 主服务小区
    #[serde(default)]
    pub serving_cell: ServingCell,
    /// 所有小区列表（包含主小区和邻区）
    pub cells: Vec<CellInfo>,
}

/// 设备信息响应（来自 D-Bus Modem 接口）
#[derive(Debug, Serialize, Default)]
pub struct DeviceInfoResponse {
    /// IMEI（设备序列号）
    pub imei: String,
    /// 制造商
    pub manufacturer: String,
    /// 型号
    pub model: String,
    /// 固件版本
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,
    /// 是否在线（射频开启）
    pub online: bool,
    /// 是否上电
    pub powered: bool,
}

/// 数据连接状态请求
#[derive(Debug, Deserialize)]
pub struct DataConnectionRequest {
    /// 是否激活数据连接
    pub active: bool,
}

/// 数据连接状态响应
#[derive(Debug, Serialize, Default)]
pub struct DataConnectionResponse {
    /// 是否激活
    pub active: bool,
}

/// 漫游设置请求
#[derive(Debug, Deserialize)]
pub struct RoamingRequest {
    /// 是否允许漫游数据
    pub allowed: bool,
}

/// 漫游状态响应
#[derive(Debug, Serialize, Default)]
pub struct RoamingResponse {
    /// 是否允许漫游数据
    pub roaming_allowed: bool,
    /// 当前是否处于漫游状态
    pub is_roaming: bool,
}

/// 飞行模式请求
#[derive(Debug, Deserialize)]
pub struct AirplaneModeRequest {
    /// 是否启用飞行模式
    pub enabled: bool,
}

/// 飞行模式响应
#[derive(Debug, Serialize, Default)]
pub struct AirplaneModeResponse {
    /// 飞行模式是否启用
    pub enabled: bool,
    /// Modem 电源状态
    pub powered: bool,
    /// Modem 在线状态（射频状态）
    pub online: bool,
}

/// 单个温度传感器信息
#[derive(Debug, Serialize, Clone)]
pub struct ThermalZone {
    /// 传感器名称（thermal_zone0, thermal_zone1, etc.）
    pub zone: String,
    /// 传感器类型（如 soc-thmzone）
    #[serde(rename = "type")]
    pub sensor_type: String,
    /// 温度值（摄氏度）
    pub temperature: f64,
}

/// SIM 卡信息响应（整合所有 SIM 相关信息）
#[derive(Debug, Serialize, Default)]
pub struct SimInfoResponse {
    /// SIM 卡是否存在
    pub present: bool,
    /// ICCID（集成电路卡识别码）
    pub iccid: String,
    /// IMSI（国际移动用户识别码）
    pub imsi: String,
    /// 手机号码列表
    pub phone_numbers: Vec<String>,
    /// 短信中心号码
    pub sms_center: String,
    /// 移动国家代码
    pub mcc: String,
    /// 移动网络代码
    pub mnc: String,
    /// PIN 状态（none/pin/puk）
    pub pin_required: String,
    /// 首选语言列表
    pub preferred_languages: Vec<String>,
}

/// SIM 卡槽信息
#[derive(Debug, Serialize, Default)]
pub struct SimSlotResponse {
    /// 当前激活的卡槽（1 或 2）
    pub active_slot: u8,
    /// 原始值
    pub raw_value: String,
}

/// 切换 SIM 卡槽请求
#[derive(Debug, Deserialize)]
pub struct SwitchSimSlotRequest {
    /// 目标卡槽（1 或 2）
    pub slot: u8,
}

/// 网络信息响应
#[derive(Debug, Serialize, Default)]
pub struct NetworkInfoResponse {
    /// 运营商名称
    pub operator_name: String,
    /// 注册状态 (registered, searching, denied, etc.)
    pub registration_status: String,
    /// 网络制式偏好
    pub technology_preference: String,
    /// 信号强度 (0-100)
    pub signal_strength: u8,
    /// MCC (移动国家代码)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcc: Option<String>,
    /// MNC (移动网络代码)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mnc: Option<String>,
}


/// QoS信息响应
#[derive(Debug, Serialize, Default)]
pub struct QosInfoResponse {
    /// QCI等级 (Quality of Service Class Identifier)
    pub qci: u8,
    /// 下行速率 (kbit/s)
    pub dl_speed: u32,
    /// 上行速率 (kbit/s)
    pub ul_speed: u32,
    /// 原始AT响应（可选，用于调试）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_response: Option<String>,
}

/// USB模式枚举
/// 1 = CDC-NCM, 2 = CDC-ECM, 3 = RNDIS
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum UsbMode {
    CdcNcm = 1,
    CdcEcm = 2,
    Rndis = 3,
}

/// 设置USB模式请求
#[derive(Debug, Deserialize)]
pub struct SetUsbModeRequest {
    /// USB模式：1=CDC-NCM, 2=CDC-ECM, 3=RNDIS
    pub mode: u8,
    /// 是否永久保存：true=写入/mnt/data/mode.cfg（永久），false=写入/mnt/data/mode_tmp.cfg（临时）
    #[serde(default)]
    pub permanent: bool,
}

/// USB模式查询响应
#[derive(Debug, Serialize, Default)]
pub struct UsbModeResponse {
    /// 当前硬件实际运行的模式（始终从 configfs 读取）
    pub current_mode: Option<u8>,
    /// 当前模式名称
    pub current_mode_name: String,
    /// 永久配置的模式（从 /mnt/data/mode.cfg 读取）
    pub permanent_mode: Option<u8>,
    /// 临时配置的模式（从 /mnt/data/mode_tmp.cfg 读取）
    pub temporary_mode: Option<u8>,
    /// 是否需要重启生效（始终为 true，因为配置文件在启动时读取）
    pub needs_reboot: bool,
    /// 读取来源：hardware=从VID/PID读取, file=从配置文件读取
    pub read_mode: String,
}

/// 系统重启请求
#[derive(Debug, Deserialize)]
pub struct SystemRebootRequest {
    /// 延迟秒数（可选，默认立即重启）
    #[serde(default)]
    pub delay_seconds: u32,
}

/// 单个网络接口的实时网速信息
#[derive(Debug, Serialize, Clone)]
pub struct NetworkSpeed {
    /// 接口名称（如 usb0, eth0, wwan0）
    pub interface: String,
    /// 下载速度 (字节/秒)
    pub rx_bytes_per_sec: u64,
    /// 上传速度 (字节/秒)
    pub tx_bytes_per_sec: u64,
    /// 总接收字节数
    pub total_rx_bytes: u64,
    /// 总发送字节数
    pub total_tx_bytes: u64,
}

/// 网速信息响应
#[derive(Debug, Serialize, Default)]
pub struct NetworkSpeedResponse {
    /// 所有网络接口的速度信息
    pub interfaces: Vec<NetworkSpeed>,
    /// 测量时间间隔（秒）
    pub interval_seconds: f64,
}

/// 内存信息响应
#[derive(Debug, Serialize, Default)]
pub struct MemoryInfo {
    /// 总内存 (字节)
    pub total_bytes: u64,
    /// 可用内存 (字节)
    pub available_bytes: u64,
    /// 已使用内存 (字节)
    pub used_bytes: u64,
    /// 内存使用率 (百分比 0-100)
    pub used_percent: f64,
    /// 缓存内存 (字节)
    pub cached_bytes: u64,
    /// 缓冲区内存 (字节)
    pub buffers_bytes: u64,
}

/// 系统运行时间响应
#[derive(Debug, Serialize, Default)]
pub struct UptimeInfo {
    /// 系统运行时长（秒）
    pub uptime_seconds: u64,
    /// 系统空闲时长（秒）
    pub idle_seconds: u64,
    /// 格式化的运行时间（如 "2天 3小时 45分钟"）
    pub uptime_formatted: String,
}

/// 系统信息（uname）
#[derive(Debug, Serialize, Default)]
pub struct SystemInfo {
    /// 系统名称（如 Linux）
    pub sysname: String,
    /// 主机名
    pub nodename: String,
    /// 内核发行版本（如 5.10.160）
    pub release: String,
    /// 内核版本信息
    pub version: String,
    /// 硬件架构（如 aarch64）
    pub machine: String,
    /// 域名（通常为空）
    #[serde(skip_serializing_if = "String::is_empty")]
    pub domainname: String,
    /// 完整信息（类似 uname -a 的输出）
    pub full_info: String,
}

/// 综合系统状态响应
#[derive(Debug, Serialize, Default)]
pub struct SystemStatsResponse {
    /// 网速信息
    pub network_speed: NetworkSpeedResponse,
    /// 内存信息
    pub memory: MemoryInfo,
    /// 磁盘信息
    pub disk: Vec<DiskInfo>,
    /// CPU 负载信息
    pub cpu_load: CpuLoadInfo,
    /// 运行时间信息
    pub uptime: UptimeInfo,
    /// 系统信息
    pub system_info: SystemInfo,
    /// 温度信息
    pub temperature: Vec<ThermalZone>,
    /// USB 模式信息
    pub usb_mode: UsbModeResponse,
}

/// 磁盘/分区信息
#[derive(Debug, Serialize, Default)]
pub struct DiskInfo {
    /// 挂载点
    pub mount_point: String,
    /// 文件系统类型
    pub fs_type: String,
    /// 总空间（字节）
    pub total_bytes: u64,
    /// 已用空间（字节）
    pub used_bytes: u64,
    /// 可用空间（字节）
    pub available_bytes: u64,
    /// 使用率（百分比）
    pub used_percent: f64,
}

/// 单个 Ping 结果
#[derive(Debug, Serialize, Clone, Default)]
pub struct PingResult {
    /// 是否成功
    pub success: bool,
    /// 延迟（毫秒），失败时为 None
    pub latency_ms: Option<f64>,
    /// 目标地址
    pub target: String,
    /// 错误信息（失败时）
    pub error: Option<String>,
}

/// 联网检测响应
#[derive(Debug, Serialize, Default)]
pub struct ConnectivityCheckResponse {
    /// IPv4 连通性
    pub ipv4: PingResult,
    /// IPv6 连通性
    pub ipv6: PingResult,
}

/// CPU 负载信息
#[derive(Debug, Serialize, Clone, Default)]
pub struct CpuLoadInfo {
    /// 1分钟平均负载
    pub load_1min: f64,
    /// 5分钟平均负载
    pub load_5min: f64,
    /// 15分钟平均负载
    pub load_15min: f64,
    /// CPU 核心数
    pub core_count: u32,
    /// 负载百分比（基于核心数计算）
    pub load_percent: f64,
}

/// CPU 核心信息
#[derive(Debug, Serialize, Clone, Default)]
pub struct CpuCore {
    /// 处理器编号
    pub processor: u32,
    /// BogoMIPS（性能指标）
    pub bogomips: String,
    /// CPU 特性列表
    pub features: Vec<String>,
    /// CPU 实现者标识
    pub implementer: String,
    /// CPU 架构版本
    pub architecture: String,
    /// CPU 变体
    pub variant: String,
    /// CPU 部件号
    pub part: String,
    /// CPU 修订版本
    pub revision: String,
}

/// CPU 信息响应
#[derive(Debug, Serialize, Default)]
pub struct CpuInfo {
    /// CPU 核心数量
    pub core_count: u32,
    /// 所有核心信息
    pub cores: Vec<CpuCore>,
    /// 硬件型号
    pub hardware: String,
    /// 序列号
    pub serial: String,
    /// CPU 型号描述（根据 implementer 和 part 识别）
    pub model_name: String,
}

/// 基站定位参数
/// 用于通过第三方API（如Google Geolocation、OpenCellID等）进行基站定位
#[derive(Debug, Serialize, Default)]
pub struct CellLocationInfo {
    /// 移动国家代码（如460=中国）
    pub mcc: String,
    /// 移动网络代码（如00=移动, 01=联通, 11=电信）
    pub mnc: String,
    /// 位置区码/跟踪区码（LAC/TAC）
    pub lac: u32,
    /// 小区ID（Cell ID）
    pub cid: u32,
    /// 信号强度（RSRP，单位：dBm）
    pub signal_strength: i32,
    /// 网络制式（nr/lte/umts/gsm）
    pub radio_type: String,
    /// 绝对频点号（ARFCN）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arfcn: Option<u32>,
    /// 物理小区标识（PCI）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pci: Option<u32>,
    /// 参考信号接收质量（RSRQ，单位：dB）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rsrq: Option<i32>,
    /// 信噪比（SINR，单位：dB）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sinr: Option<i32>,
}

/// 基站定位信息响应
#[derive(Debug, Serialize, Default)]
pub struct CellLocationResponse {
    /// 是否可用（是否有足够的定位参数）
    pub available: bool,
    /// 主服务小区定位参数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cell_info: Option<CellLocationInfo>,
    /// 邻区定位参数列表（可用于提高定位精度）
    pub neighbor_cells: Vec<CellLocationInfo>,
    /// 使用建议
    pub usage_hint: String,
}

/// IP地址信息
#[derive(Debug, Serialize, Clone)]
pub struct IpAddress {
    /// IP地址
    pub address: String,
    /// 前缀长度（子网掩码位数）
    pub prefix_len: u8,
    /// IP类型：ipv4 或 ipv6
    pub ip_type: String,
    /// 地址范围：private（内网）, public（公网）, loopback（回环）, link-local（链路本地）
    pub scope: String,
}

/// 网络接口详细信息
#[derive(Debug, Serialize, Clone)]
pub struct NetworkInterfaceInfo {
    /// 接口名称（如 eth0, wlan0, usb0）
    pub name: String,
    /// 接口状态：up, down
    pub status: String,
    /// MAC地址
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mac_address: Option<String>,
    /// MTU（最大传输单元）
    pub mtu: u32,
    /// IP地址列表（IPv4和IPv6）
    pub ip_addresses: Vec<IpAddress>,
    /// 接收字节数
    pub rx_bytes: u64,
    /// 发送字节数
    pub tx_bytes: u64,
    /// 接收包数
    pub rx_packets: u64,
    /// 发送包数
    pub tx_packets: u64,
    /// 接收错误数
    pub rx_errors: u64,
    /// 发送错误数
    pub tx_errors: u64,
}

/// 网络接口列表响应
#[derive(Debug, Serialize, Default)]
pub struct NetworkInterfacesResponse {
    /// 网络接口列表
    pub interfaces: Vec<NetworkInterfaceInfo>,
    /// 接口总数
    pub total_count: usize,
}

/// 射频模式枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RadioMode {
    /// 4G/5G 自动切换
    Auto,
    /// 仅 4G LTE
    #[serde(rename = "lte")]
    LteOnly,
    /// 仅 5G NR
    #[serde(rename = "nr")]
    NrOnly,
}

impl RadioMode {
    /// 转换为 ofono TechnologyPreference 字符串
    pub fn to_ofono_value(&self) -> &'static str {
        match self {
            RadioMode::Auto => "NR 5G/LTE auto",
            RadioMode::LteOnly => "LTE only",
            RadioMode::NrOnly => "NR 5G only",
        }
    }

    /// 从 ofono TechnologyPreference 字符串解析
    pub fn from_ofono_value(value: &str) -> Option<Self> {
        match value {
            "NR 5G/LTE auto" | "LTE/GSM/WCDMA auto" | "NR 5G/LTE/GSM/WCDMA auto" => Some(RadioMode::Auto),
            "LTE only" => Some(RadioMode::LteOnly),
            "NR 5G only" => Some(RadioMode::NrOnly),
            _ => None,
        }
    }
}

/// 射频模式响应
#[derive(Debug, Serialize, Default)]
pub struct RadioModeResponse {
    /// 当前射频模式
    pub mode: String,
    /// ofono 原始 TechnologyPreference 值
    pub technology_preference: String,
}

/// 射频模式请求
#[derive(Debug, Deserialize)]
pub struct RadioModeRequest {
    /// 目标射频模式: auto, lte, nr
    pub mode: RadioMode,
}

/// 频段锁定状态（4G/5G 统一结构）
#[derive(Debug, Serialize, Default)]
pub struct BandLockStatus {
    /// 是否已锁定频段
    pub locked: bool,
    /// 锁定的 LTE FDD 频段列表（如 [1, 3, 8]）
    pub lte_fdd_bands: Vec<u8>,
    /// 锁定的 LTE TDD 频段列表（如 [38, 40, 41]）
    pub lte_tdd_bands: Vec<u8>,
    /// 锁定的 NR FDD 频段列表（如 [1, 28]）
    pub nr_fdd_bands: Vec<u8>,
    /// 锁定的 NR TDD 频段列表（如 [41, 77, 78, 79]）
    pub nr_tdd_bands: Vec<u8>,
    /// 原始 AT 响应（用于调试）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_response: Option<String>,
}

/// 频段锁定请求
#[derive(Debug, Deserialize)]
pub struct BandLockRequest {
    /// LTE FDD 频段列表（如 [1, 3, 8]）
    #[serde(default)]
    pub lte_fdd_bands: Vec<u8>,
    /// LTE TDD 频段列表（如 [38, 40, 41]）
    #[serde(default)]
    pub lte_tdd_bands: Vec<u8>,
    /// NR FDD 频段列表（如 [1, 28]）
    #[serde(default)]
    pub nr_fdd_bands: Vec<u8>,
    /// NR TDD 频段列表（如 [41, 77, 78, 79]）
    #[serde(default)]
    pub nr_tdd_bands: Vec<u8>,
}

// ============ 小区锁定模型 ============
// 使用展锐 AT+SPFORCEFRQ 指令实现
// 类型: 12=LTE, 16=NR
// 操作: 0=清除, 2=设置

/// 单个 RAT 的小区锁定状态
#[derive(Debug, Serialize, Default, Clone)]
pub struct CellLockRatStatus {
    /// RAT 类型 (12=LTE, 16=NR)
    pub rat: u8,
    /// RAT 名称
    pub rat_name: String,
    /// 是否启用锁定
    pub enabled: bool,
    /// 锁定类型
    pub lock_type: u8,
    /// 锁定的 PCI（如果有）
    pub pci: Option<u16>,
    /// 锁定的 ARFCN（如果有）
    pub arfcn: Option<u32>,
}

/// 小区锁定状态响应
#[derive(Debug, Serialize, Default)]
pub struct CellLockStatusResponse {
    /// 各 RAT 的锁定状态列表
    pub rat_status: Vec<CellLockRatStatus>,
    /// 是否有任何锁定生效
    pub any_locked: bool,
}

/// 小区锁定请求
/// 
/// 使用 AT+SPFORCEFRQ 指令格式:
/// - 锁定: AT+SPFORCEFRQ=<type>,2,<arfcn>,<pci>
/// - 解锁: AT+SPFORCEFRQ=<type>,0
/// 
/// 其中 type: 12=LTE, 16=NR
#[derive(Debug, Deserialize)]
pub struct CellLockRequest {
    /// RAT 类型
    /// - 12: LTE
    /// - 16: NR (默认)
    /// - 也支持旧值: 1/2=LTE, 5/6/7=NR (会自动转换)
    #[serde(default = "default_nr_rat")]
    pub rat: u8,
    /// 是否启用锁定
    pub enable: bool,
    /// 锁定类型 (保留字段，暂不使用)
    #[serde(default)]
    #[allow(dead_code)]
    pub lock_type: u8,
    /// PCI（物理小区标识），锁定时必填
    #[serde(default)]
    pub pci: Option<u16>,
    /// ARFCN（绝对频点号），锁定时必填
    #[serde(default)]
    pub arfcn: Option<u32>,
}

fn default_nr_rat() -> u8 {
    16 // 默认 NR
}

/// 解除所有小区锁定请求（空请求）
#[derive(Debug, Deserialize, Default)]
pub struct CellUnlockRequest {}

// ============ 电话相关模型 ============

/// 拨打电话请求
#[derive(Debug, Deserialize)]
pub struct MakeCallRequest {
    /// 目标电话号码
    pub phone_number: String,
}

/// 通话信息
#[derive(Debug, Serialize, Clone)]
pub struct CallInfo {
    /// 通话路径（D-Bus 对象路径）
    pub path: String,
    /// 电话号码
    pub phone_number: String,
    /// 通话状态：active, dialing, alerting, incoming, held
    pub state: String,
    /// 通话方向：incoming 或 outgoing
    pub direction: String,
    /// 开始时间（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<String>,
}

/// 通话列表响应
#[derive(Debug, Serialize)]
pub struct CallListResponse {
    /// 当前通话列表
    pub calls: Vec<CallInfo>,
}

impl Default for CallListResponse {
    fn default() -> Self {
        Self { calls: Vec::new() }
    }
}

impl Default for CallInfo {
    fn default() -> Self {
        Self {
            path: String::new(),
            phone_number: String::new(),
            state: String::new(),
            direction: String::new(),
            start_time: None,
        }
    }
}

/// 挂断电话请求
#[derive(Debug, Deserialize)]
pub struct HangupCallRequest {
    /// 通话路径
    pub path: String,
}

// ============ NITZ 网络时间模型 ============

/// NITZ 网络时间响应
#[derive(Debug, Serialize, Default)]
pub struct NitzTimeResponse {
    /// 网络时间字符串（如 "2025-12-02 15:05:47 +08:00 (DST=0)"）
    pub time_string: String,
    /// 是否可用
    pub available: bool,
}

// ============ IMS（VoLTE）模型 ============

/// IMS 状态响应
#[derive(Debug, Serialize, Default)]
pub struct ImsStatusResponse {
    /// 是否已注册到 IMS
    pub registered: bool,
    /// 是否支持语音通话
    pub voice_capable: bool,
    /// 是否支持短信
    pub sms_capable: bool,
}

// ============ 通话音量模型 ============

/// 通话音量响应
#[derive(Debug, Serialize, Default)]
pub struct CallVolumeResponse {
    /// 扬声器音量（0-100）
    pub speaker_volume: u8,
    /// 麦克风音量（0-100）
    pub microphone_volume: u8,
    /// 是否静音
    pub muted: bool,
}

/// 设置通话音量请求
#[derive(Debug, Deserialize)]
pub struct SetCallVolumeRequest {
    /// 扬声器音量（0-100，可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speaker_volume: Option<u8>,
    /// 麦克风音量（0-100，可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub microphone_volume: Option<u8>,
    /// 是否静音（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub muted: Option<bool>,
}

// ============ 语音留言模型 ============

/// 语音留言状态响应
#[derive(Debug, Serialize, Default)]
pub struct VoicemailStatusResponse {
    /// 是否有语音留言等待
    pub waiting: bool,
    /// 留言数量
    pub message_count: u8,
    /// 语音信箱号码
    pub mailbox_number: String,
}

// ============ 运营商模型 ============

/// 运营商信息
#[derive(Debug, Serialize, Clone)]
pub struct OperatorInfo {
    /// D-Bus 对象路径
    pub path: String,
    /// 运营商名称
    pub name: String,
    /// 状态：available, current, forbidden
    pub status: String,
    /// 移动国家代码（MCC）
    pub mcc: String,
    /// 移动网络代码（MNC）
    pub mnc: String,
    /// 支持的技术（如 ["LTE", "NR"]）
    pub technologies: Vec<String>,
}

/// 运营商列表响应
#[derive(Debug, Serialize, Default)]
pub struct OperatorListResponse {
    /// 运营商列表
    pub operators: Vec<OperatorInfo>,
}

/// 手动注册运营商请求
#[derive(Debug, Deserialize)]
pub struct ManualRegisterRequest {
    /// MCCMNC（如 "46001" 表示中国联通）
    pub mccmnc: String,
}

// ============ IMEISV（软件版本号）模型 ============

/// IMEISV 响应
#[derive(Debug, Serialize, Default)]
pub struct ImeisvResponse {
    /// 软件版本号（SVN）
    pub software_version_number: String,
}

// ============ 信号强度详细模型 ============

/// 信号强度详细响应
#[derive(Debug, Serialize, Default)]
pub struct SignalStrengthResponse {
    /// 信号强度（0-100，或负数 dBm）
    pub strength: i32,
}

// ============ 呼叫转移模型 ============

/// 呼叫转移设置响应
#[derive(Debug, Serialize, Default)]
pub struct CallForwardingResponse {
    /// 无条件转移号码
    pub voice_unconditional: String,
    /// 占线时转移号码
    pub voice_busy: String,
    /// 无应答时转移号码
    pub voice_no_reply: String,
    /// 无应答超时时间（秒）
    pub voice_no_reply_timeout: u16,
    /// 不可达时转移号码
    pub voice_not_reachable: String,
    /// SIM 卡上的转移标志
    pub forwarding_flag_on_sim: bool,
}

/// 设置呼叫转移请求
#[derive(Debug, Deserialize)]
pub struct SetCallForwardingRequest {
    /// 转移类型：unconditional, busy, noreply, notreachable
    pub forward_type: String,
    /// 目标号码（空字符串表示禁用）
    pub number: String,
    /// 无应答超时（仅 noreply 类型需要，秒）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u16>,
}

// ============ 通话设置模型 ============

/// 通话设置响应
#[derive(Debug, Serialize, Default)]
pub struct CallSettingsResponse {
    /// 主叫号码显示：enabled, disabled, unknown
    pub calling_line_presentation: String,
    /// 主叫姓名显示：enabled, disabled, unknown
    pub calling_name_presentation: String,
    /// 被叫号码显示：enabled, disabled, unknown
    pub connected_line_presentation: String,
    /// 被叫号码限制：enabled, disabled, unknown
    pub connected_line_restriction: String,
    /// 已拨号码显示：enabled, disabled, unknown
    pub called_line_presentation: String,
    /// 主叫号码限制：enabled, disabled, unknown, on, off
    pub calling_line_restriction: String,
    /// 隐藏来电显示：default, enabled, disabled
    pub hide_caller_id: String,
    /// 呼叫等待：enabled, disabled, unknown
    pub voice_call_waiting: String,
}

/// 设置通话设置请求
#[derive(Debug, Deserialize)]
pub struct SetCallSettingRequest {
    /// 设置项：HideCallerId, VoiceCallWaiting
    pub property: String,
    /// 值：default/enabled/disabled
    pub value: String,
}

// ============ 短信相关模型 ============

/// 发送短信请求
#[derive(Debug, Deserialize)]
pub struct SendSmsRequest {
    /// 目标电话号码
    pub phone_number: String,
    /// 短信内容
    pub content: String,
}

/// 短信列表请求
#[derive(Debug, Deserialize)]
pub struct SmsListRequest {
    /// 每页数量（默认 50）
    #[serde(default = "default_page_size")]
    pub limit: i64,
    /// 偏移量（默认 0）
    #[serde(default)]
    pub offset: i64,
}

fn default_page_size() -> i64 {
    50
}

/// 短信对话请求
#[derive(Debug, Deserialize)]
pub struct SmsConversationRequest {
    /// 电话号码
    pub phone_number: String,
    /// 最多返回条数（默认 50）
    #[serde(default = "default_page_size")]
    pub limit: i64,
}

// ============ APN 管理模型 ============

/// APN Context 信息
#[derive(Debug, Serialize, Default, Clone)]
pub struct ApnContext {
    /// D-Bus 路径 (如 /ril_0/context2)
    pub path: String,
    /// 名称
    pub name: String,
    /// 是否激活
    pub active: bool,
    /// APN 名称 (如 cbnet, cmnet)
    pub apn: String,
    /// 协议: ip/ipv6/dual
    pub protocol: String,
    /// 用户名
    pub username: String,
    /// 密码
    pub password: String,
    /// 认证方式: none/pap/chap
    pub auth_method: String,
    /// 类型: internet/mms/ims
    pub context_type: String,
}

/// APN 列表响应
#[derive(Debug, Serialize, Default)]
pub struct ApnListResponse {
    /// APN context 列表
    pub contexts: Vec<ApnContext>,
}

/// 设置 APN 请求
#[derive(Debug, Deserialize)]
pub struct SetApnRequest {
    /// 要修改的 context 路径 (如 /ril_0/context2)
    pub context_path: String,
    /// APN 名称
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apn: Option<String>,
    /// 协议: ip/ipv6/dual
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<String>,
    /// 用户名
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    /// 密码
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    /// 认证方式: none/pap/chap
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_method: Option<String>,
}

// ============ 通话记录模型 ============

/// 通话记录列表请求
#[derive(Debug, Deserialize, Default)]
pub struct CallHistoryRequest {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    50
}

/// 通话记录列表响应
#[derive(Debug, Serialize, Default)]
pub struct CallHistoryResponse {
    pub records: Vec<crate::db::CallRecord>,
    pub stats: crate::db::CallStats,
}

/// 删除通话记录请求
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct DeleteCallRequest {
    pub id: i64,
}

// ============ Webhook 配置模型 ============

/// Webhook 测试结果
#[derive(Debug, Serialize)]
pub struct WebhookTestResponse {
    pub success: bool,
    pub message: String,
}

// ============ OTA 更新模型 ============

/// OTA 更新包元数据（meta.json 格式）
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct OtaMeta {
    /// 版本号（如 "3.1.0"）
    pub version: String,
    /// Git commit hash（短格式）
    pub commit: String,
    /// 构建时间（ISO 8601）
    pub build_time: String,
    /// 后端二进制 MD5
    pub binary_md5: String,
    /// 前端目录 MD5（所有文件 hash 的 hash）
    pub frontend_md5: String,
    /// 目标架构
    pub arch: String,
    /// 最低兼容版本（可选，用于阻止降级）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_version: Option<String>,
}

/// OTA 更新状态响应
#[derive(Debug, Serialize)]
pub struct OtaStatusResponse {
    /// 当前版本
    pub current_version: String,
    /// 当前 commit
    pub current_commit: String,
    /// 是否有待安装的更新
    pub pending_update: bool,
    /// 待安装的更新信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pending_meta: Option<OtaMeta>,
}

/// OTA 上传响应
#[derive(Debug, Serialize, Default)]
pub struct OtaUploadResponse {
    /// 上传的更新包元数据
    pub meta: OtaMeta,
    /// 验证结果
    pub validation: OtaValidation,
}

/// OTA 验证结果
#[derive(Debug, Serialize, Default)]
pub struct OtaValidation {
    /// 是否验证通过
    pub valid: bool,
    /// 版本是否比当前新
    pub is_newer: bool,
    /// 二进制 MD5 是否匹配
    pub binary_md5_match: bool,
    /// 前端 MD5 是否匹配
    pub frontend_md5_match: bool,
    /// 架构是否匹配
    pub arch_match: bool,
    /// 错误消息（如果验证失败）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// OTA 应用更新请求
#[derive(Debug, Deserialize)]
pub struct OtaApplyRequest {
    /// 是否立即重启
    #[serde(default)]
    pub restart_now: bool,
}

