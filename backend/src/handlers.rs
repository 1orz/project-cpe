/*
 * @Author: 1orz cloudorzi@gmail.com
 * @Date: 2025-12-11 17:44:29
 * @LastEditors: 1orz cloudorzi@gmail.com
 * @LastEditTime: 2025-12-13 12:46:04
 * @FilePath: /udx710-backend/backend/src/handlers.rs
 * @Description: 
 * 
 * Copyright (c) 2025 by 1orz, All Rights Reserved. 
 */
//! API 处理器模块
//! 
//! 包含所有 HTTP API 的处理函数

use axum::{
    extract::{Query, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    Json,
};
use serde_json::json;
use std::sync::Arc;
use zbus::Connection;

use crate::{
    dbus::{
        get_airplane_mode, get_all_apn_contexts, get_data_connection_status, get_device_info_data,
        get_network_info_data, get_qos_info_data, get_radio_mode, get_roaming_status, get_serving_cell_info,
        get_sim_info_data, send_at_command, set_airplane_mode, set_apn_properties, set_data_connection,
        set_radio_mode, set_roaming_allowed,
    },
    iptables::flush_iptables,
    models::*,
    usb_switch,
    utils::{
        bands_to_bitmask, bitmask_to_bands, build_splband_lte_command, build_splband_nr_command,
        format_uptime, get_active_interfaces, get_cell_command_config,
        parse_at_response_to_2d_vec, parse_neighbor_cells, parse_primary_cell,
        parse_splband_lte_response, parse_splband_nr_response, read_cpu_info, read_cpu_load_sync,
        read_disk_info, read_interface_stats, read_memory_info, read_network_interfaces, read_system_info,
        read_uptime, sample_cpu_usage,
    },
};
use std::process::Command;

/// 处理 OPTIONS 请求（CORS 预检）
pub async fn options_handler() -> impl IntoResponse {
    StatusCode::NO_CONTENT
}

/// POST /api/at - 发送 AT 指令
///
/// # 请求体
/// ```json
/// {
///   "cmd": "AT+CGSN"
/// }
/// ```
pub async fn post_at_command(
    State(conn): State<Arc<Connection>>,
    Json(payload): Json<AtCommandRequest>,
) -> impl IntoResponse {
    let (status, body_text) = match send_at_command(&conn, &payload.cmd).await {
        Ok(result) => (StatusCode::OK, result),
        Err(e) => (StatusCode::OK, format!("Error: {}", e)),
    };

    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::CONTENT_TYPE,
        HeaderValue::from_static("text/plain; charset=utf-8"),
    );

    (status, headers, body_text)
}

/// 获取主小区信息
///
/// # Arguments
/// * `conn` - D-Bus 连接
/// * `cmd` - AT 指令
/// * `tech` - 网络制式
///
/// # Returns
/// 解析后的主小区信息
async fn fetch_primary_cell(
    conn: &Connection,
    cmd: &str,
    tech: &str,
) -> Result<CellInfo, String> {
    let response = send_at_command(conn, cmd)
        .await
        .map_err(|e| format!("Primary cell AT command failed: {}", e))?;

    let parsed = parse_at_response_to_2d_vec(&response);
    let cell = parse_primary_cell(tech, &parsed);

    Ok(cell)
}

/// 获取邻区信息列表
///
/// # Arguments
/// * `conn` - D-Bus 连接
/// * `cmd` - AT 指令
/// * `tech` - 网络制式
///
/// # Returns
/// 解析后的邻区信息列表
async fn fetch_neighbor_cells(
    conn: &Connection,
    cmd: &str,
    tech: &str,
) -> Result<Vec<CellInfo>, String> {
    let response = send_at_command(conn, cmd)
        .await
        .map_err(|e| format!("Neighbor cell AT command failed: {}", e))?;

    let parsed = parse_at_response_to_2d_vec(&response);
    let cells = parse_neighbor_cells(tech, &parsed);

    Ok(cells)
}

/// GET /api/cells - Get cell information
///
/// # Response example
/// ```json
/// {
///   "status": "ok",
///   "message": "Success",
///   "data": {
///     "serving_cell": {
///       "tech": "nr",
///       "cell_id": 12345,
///       "tac": 100
///     },
///     "cells": [...]
///   }
/// }
/// ```
pub async fn get_cells(State(conn): State<Arc<Connection>>) -> impl IntoResponse {
    let result = async {
        // 1. 获取服务小区信息（包含网络制式）
        let serving_cell = get_serving_cell_info(&conn)
            .await
            .map_err(|e| format!("Failed to get serving cell info: {}", e))?;

        let tech = serving_cell.tech.as_str();

        // 2. 根据网络制式获取对应的 AT 指令配置
        let cmd_config = get_cell_command_config(tech)
            .ok_or_else(|| format!("Unsupported network type: {}", tech))?;

        // 3. 顺序获取主小区和邻区信息
        // 注意：ofono D-Bus 不支持并发 AT 指令，必须串行执行
        let primary_cell = fetch_primary_cell(&conn, cmd_config.primary, tech).await?;
        let neighbor_cells = fetch_neighbor_cells(&conn, cmd_config.neighbor, tech).await?;

        // 4. 合并主小区和邻区
        let mut all_cells = vec![primary_cell];
        all_cells.extend(neighbor_cells);

        Ok::<_, String>(CellsResponse {
            serving_cell,
            cells: all_cells,
        })
    }
    .await;

    match result {
        Ok(data) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", data)),
        ),
        Err(msg) => (
            StatusCode::OK,
            Json(ApiResponse::<CellsResponse>::error(msg)),
        ),
    }
}

/// GET /api/device - 获取设备信息（来自 D-Bus Modem 接口）
///
/// # Response example
/// ```json
/// {
///   "status": "ok",
///   "message": "Success",
///   "data": {
///     "imei": "123456789012345",
///     "manufacturer": "UNISOC",
///     "model": "UDX710",
///     "revision": "1.0.0",
///     "online": true,
///     "powered": true
///   }
/// }
/// ```
pub async fn get_device_info(State(conn): State<Arc<Connection>>) -> impl IntoResponse {
    match get_device_info_data(&conn).await {
        Ok(data) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", data)),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<DeviceInfoResponse>::error(format!(
                "Failed to get device info: {}",
                e
            ))),
        ),
    }
}

/// POST /api/data - Set data connection status
///
/// # Request body
/// ```json
/// {
///   "active": true
/// }
/// ```
///
/// # Response example
/// ```json
/// {
///   "status": "ok",
///   "message": "Data connection updated successfully"
/// }
/// ```
///
/// # 说明
/// 每次切换数据连接状态时，会自动清空 iptables 规则（flush），
/// 以确保网络配置处于干净状态
pub async fn set_data_status(
    State(conn): State<Arc<Connection>>,
    Json(payload): Json<DataConnectionRequest>,
) -> impl IntoResponse {
    // 1. 先清空 iptables 规则
    if let Err(_e) = flush_iptables().await {
        // 清空规则失败不应阻止数据连接操作，静默处理
    }

    // 2. 设置数据连接状态
    match set_data_connection(&conn, payload.active).await {
        Ok(_) => {
            
            (
                StatusCode::OK,
                Json(ApiResponse::success_with_message(
                    "Data connection updated successfully",
                    DataConnectionResponse { active: payload.active },
                )),
            )
        }
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<DataConnectionResponse>::error(format!(
                "Failed to set data connection: {}",
                e
            ))),
        ),
    }
}

/// GET /api/data - Get data connection status
///
/// # Response example
/// ```json
/// {
///   "status": "ok",
///   "message": "Success",
///   "data": {
///     "active": true
///   }
/// }
/// ```
pub async fn get_data_status(State(conn): State<Arc<Connection>>) -> impl IntoResponse {
    match get_data_connection_status(&conn).await {
        Ok(active) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "Success",
                DataConnectionResponse { active },
            )),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<DataConnectionResponse>::error(format!(
                "Failed to get data connection status: {}",
                e
            ))),
        ),
    }
}

/// GET /api/roaming - Get roaming status
///
/// # Response example
/// ```json
/// {
///   "status": "ok",
///   "message": "Success",
///   "data": {
///     "roaming_allowed": true,
///     "is_roaming": false
///   }
/// }
/// ```
pub async fn get_roaming_status_handler(
    State(conn): State<Arc<Connection>>,
) -> impl IntoResponse {
    match get_roaming_status(&conn).await {
        Ok((roaming_allowed, is_roaming)) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "Success",
                RoamingResponse { roaming_allowed, is_roaming },
            )),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<RoamingResponse>::error(format!(
                "Failed to get roaming status: {}",
                e
            ))),
        ),
    }
}

/// POST /api/roaming - Set roaming allowed
///
/// # Request body
/// ```json
/// {
///   "allowed": true
/// }
/// ```
///
/// # Response example
/// ```json
/// {
///   "status": "ok",
///   "message": "Roaming enabled successfully",
///   "data": {
///     "roaming_allowed": true,
///     "is_roaming": false
///   }
/// }
/// ```
pub async fn set_roaming_status_handler(
    State(conn): State<Arc<Connection>>,
    Json(payload): Json<RoamingRequest>,
) -> impl IntoResponse {
    match set_roaming_allowed(&conn, payload.allowed).await {
        Ok(_) => {
            // Read back the status to confirm
            match get_roaming_status(&conn).await {
                Ok((roaming_allowed, is_roaming)) => {
                    let msg = if payload.allowed {
                        "Roaming enabled successfully"
                    } else {
                        "Roaming disabled successfully"
                    };
                    (
                        StatusCode::OK,
                        Json(ApiResponse::success_with_message(
                            msg,
                            RoamingResponse { roaming_allowed, is_roaming },
                        )),
                    )
                }
                Err(e) => (
                    StatusCode::OK,
                    Json(ApiResponse::<RoamingResponse>::error(format!(
                        "Roaming setting updated, but failed to read status: {}",
                        e
                    ))),
                ),
            }
        }
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<RoamingResponse>::error(format!(
                "Failed to set roaming: {}",
                e
            ))),
        ),
    }
}

/// POST /api/airplane-mode - Set airplane mode
///
/// # Request body
/// ```json
/// {
///   "enabled": true
/// }
/// ```
///
/// # Response example
/// ```json
/// {
///   "status": "ok",
///   "message": "Airplane mode enabled successfully",
///   "data": {
///     "enabled": true,
///     "powered": true,
///     "online": false
///   }
/// }
/// ```
pub async fn set_airplane_mode_handler(
    State(conn): State<Arc<Connection>>,
    Json(payload): Json<AirplaneModeRequest>,
) -> impl IntoResponse {
    match set_airplane_mode(&conn, payload.enabled).await {
        Ok(_) => {
            // 读取当前状态确认
            match get_airplane_mode(&conn).await {
                Ok(status) => {
                    let msg = if payload.enabled {
                        "Airplane mode enabled successfully"
                    } else {
                        "Airplane mode disabled successfully"
                    };
                    (
                        StatusCode::OK,
                        Json(ApiResponse::success_with_message(msg, status)),
                    )
                }
                Err(e) => (
                    StatusCode::OK,
                    Json(ApiResponse::<AirplaneModeResponse>::error(format!(
                        "Airplane mode set but failed to read status: {}",
                        e
                    ))),
                ),
            }
        }
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<AirplaneModeResponse>::error(format!(
                "Failed to set airplane mode: {}",
                e
            ))),
        ),
    }
}

/// GET /api/airplane-mode - Get airplane mode status
///
/// # Response example
/// ```json
/// {
///   "status": "ok",
///   "message": "Success",
///   "data": {
///     "enabled": false,
///     "powered": true,
///     "online": true
///   }
/// }
/// ```
pub async fn get_airplane_mode_handler(State(conn): State<Arc<Connection>>) -> impl IntoResponse {
    match get_airplane_mode(&conn).await {
        Ok(status) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", status)),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<AirplaneModeResponse>::error(format!(
                "Failed to get airplane mode status: {}",
                e
            ))),
        ),
    }
}

/// GET /api/health - Health check endpoint
///
/// # Response example
/// ```json
/// {
///   "status": "ok",
///   "message": "Service is running"
/// }
/// ```
pub async fn health_check() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "status": "ok",
            "message": "Service is running",
            "version": env!("CARGO_PKG_VERSION"),
        })),
    )
}

/// GET /api/sim - Get SIM card information
///
/// # Response example
/// ```json
/// {
///   "status": "ok",
///   "message": "Success",
///   "data": {
///     "present": true,
///     "pin_required": "none",
///     "service_center_address": "+8613800200569",
///     "subscriber_identity": "460123456789012"
///   }
/// }
/// ```
pub async fn get_sim_info(State(conn): State<Arc<Connection>>) -> impl IntoResponse {
    match get_sim_info_data(&conn).await {
        Ok(data) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", data)),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<SimInfoResponse>::error(format!(
                "Failed to get SIM info: {}",
                e
            ))),
        ),
    }
}

/// GET /api/network - Get network information
///
/// # Response example
/// ```json
/// {
///   "status": "ok",
///   "message": "Success",
///   "data": {
///     "operator_name": "CMCC",
///     "registration_status": "registered",
///     "technology_preference": "NR 5G/LTE auto",
///     "signal_strength": 85
///   }
/// }
/// ```
pub async fn get_network_info(State(conn): State<Arc<Connection>>) -> impl IntoResponse {
    match get_network_info_data(&conn).await {
        Ok(data) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", data)),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<NetworkInfoResponse>::error(format!(
                "Failed to get network info: {}",
                e
            ))),
        ),
    }
}

/// GET /api/qos - Get QoS information
///
/// # Response example
/// ```json
/// {
///   "status": "ok",
///   "message": "Success",
///   "data": {
///     "qci": 5,
///     "dl_speed": 30000,
///     "ul_speed": 30000
///   }
/// }
/// ```
pub async fn get_qos_info(State(conn): State<Arc<Connection>>) -> impl IntoResponse {
    match get_qos_info_data(&conn).await {
        Ok(data) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", data)),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<QosInfoResponse>::error(format!(
                "Failed to get QoS info: {}",
                e
            ))),
        ),
    }
}

/// 读取温度传感器数据（内部工具函数）
fn read_temperature_sensors() -> Vec<ThermalZone> {
    use std::fs;
    use std::path::Path;

    let thermal_path = Path::new("/sys/class/thermal");
    let mut sensors = Vec::new();

    if let Ok(entries) = fs::read_dir(thermal_path) {
        for entry in entries.flatten() {
            let file_name = entry.file_name();
            let name = file_name.to_string_lossy();

            if name.starts_with("thermal_zone") {
                let zone_path = entry.path();
                
                let sensor_type = fs::read_to_string(zone_path.join("type"))
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default();

                let temperature = fs::read_to_string(zone_path.join("temp"))
                    .ok()
                    .and_then(|s| s.trim().parse::<i32>().ok())
                    .map(|t| t as f64 / 1000.0)
                    .unwrap_or(0.0);

                sensors.push(ThermalZone {
                    zone: name.to_string(),
                    sensor_type,
                    temperature,
                });
            }
        }
    }

    sensors.sort_by(|a, b| a.zone.cmp(&b.zone));
    sensors
}

/// 获取USB模式名称
fn get_mode_name(mode: Option<u8>) -> String {
    match mode {
        Some(1) => "CDC-NCM".to_string(),
        Some(2) => "CDC-ECM".to_string(),
        Some(3) => "RNDIS".to_string(),
        _ => "Unknown".to_string(),
    }
}

/// GET /api/usb-mode - 查询USB模式配置
///
/// 返回当前硬件实际运行的模式、永久配置和临时配置
///
/// # Response example
/// ```json
/// {
///   "status": "ok",
///   "message": "Success",
///   "data": {
///     "current_mode": 1,
///     "current_mode_name": "CDC-NCM",
///     "permanent_mode": 1,
///     "temporary_mode": null,
///     "needs_reboot": true,
///     "read_mode": "hardware"
///   }
/// }
/// ```
pub async fn get_usb_mode() -> impl IntoResponse {
    // 读取 USB 模式配置（包括硬件状态和配置文件）
    match usb_switch::get_usb_mode_config() {
        Ok(config) => {
            let response = UsbModeResponse {
                current_mode: config.current_mode,
                current_mode_name: get_mode_name(config.current_mode),
                permanent_mode: config.permanent_mode,
                temporary_mode: config.temporary_mode,
                needs_reboot: true, // 始终需要重启
                read_mode: "hardware".to_string(),
            };
            (
                StatusCode::OK,
                Json(ApiResponse::success_with_message("Success", response)),
            )
        }
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<UsbModeResponse>::error(format!(
                "Failed to get USB mode: {}",
                e
            ))),
        ),
    }
}

/// POST /api/usb-mode - 设置USB模式配置（写入配置文件，重启后生效）
///
/// # Request body
/// ```json
/// {
///   "mode": 1,
///   "permanent": true  // true=永久模式, false=临时模式
/// }
/// ```
///
/// # 支持的模式
/// - 1: CDC-NCM
/// - 2: CDC-ECM
/// - 3: RNDIS
///
/// # Response example
/// ```json
/// {
///   "status": "ok",
///   "message": "USB mode configuration saved. Please reboot to apply changes."
/// }
/// ```
pub async fn set_usb_mode(Json(payload): Json<SetUsbModeRequest>) -> impl IntoResponse {
    // 验证模式值
    if !(1..=3).contains(&payload.mode) {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(
                "Invalid mode: must be 1 (CDC-NCM), 2 (CDC-ECM), or 3 (RNDIS)",
            )),
        );
    }
    
    // 写入配置文件
    match usb_switch::set_usb_mode_config(payload.mode, payload.permanent) {
        Ok(_) => {
            let mode_name = get_mode_name(Some(payload.mode));
            let mode_type = if payload.permanent { "永久" } else { "临时" };
            (
                StatusCode::OK,
                Json(ApiResponse::success_with_message(
                    format!("USB 模式已设置为 {} ({})，请重启设备后生效", mode_name, mode_type),
                    (),
                )),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Failed to set USB mode: {}", e))),
        ),
    }
}

/// POST /api/usb-advance - 热切换USB模式（立即生效，无需重启）
///
/// 高级接口：直接操作 configfs 实现热切换
///
/// # Request body
/// ```json
/// {
///   "mode": 1
/// }
/// ```
///
/// # 支持的模式
/// - 1: CDC-NCM
/// - 2: CDC-ECM
/// - 3: RNDIS
///
/// # 注意事项
/// - 热切换会导致 USB 连接短暂断开（约 1-2 秒）
/// - macOS 可能需要更长时间识别新设备
/// - 模式 3 (RNDIS) 在 macOS/Linux 上可能需要额外驱动
/// - 建议使用模式 1 (NCM) 以获得最佳跨平台兼容性
pub async fn set_usb_mode_advanced(Json(payload): Json<SetUsbModeRequest>) -> impl IntoResponse {
    // 验证模式值
    if !(1..=3).contains(&payload.mode) {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(
                "Invalid mode: must be 1 (CDC-NCM), 2 (CDC-ECM), or 3 (RNDIS)",
            )),
        );
    }
    
    // 执行热切换
    match usb_switch::switch_usb_mode_advanced(payload.mode) {
        Ok(_) => {
            let mode_name = get_mode_name(Some(payload.mode));
            (
                StatusCode::OK,
                Json(ApiResponse::success_with_message(
                    format!("USB 模式已热切换为 {} (无需重启)", mode_name),
                    (),
                )),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("USB 模式切换失败: {}", e))),
        ),
    }
}

/// GET /api/stats/cpu - 获取 CPU 信息
///
/// # Response example
/// ```json
/// {
///   "status": "ok",
///   "message": "Success",
///   "data": {
///     "core_count": 2,
///     "cores": [
///       {
///         "processor": 0,
///         "bogomips": "52.00",
///         "features": ["fp", "asimd", "evtstrm", "aes"],
///         "implementer": "0x41",
///         "architecture": "8",
///         "variant": "0x1",
///         "part": "0xd05",
///         "revision": "0"
///       }
///     ],
///     "hardware": "Unisoc UDX710",
///     "serial": "0000000000000000",
///     "model_name": "ARM Cortex-A55"
///   }
/// }
/// ```
pub async fn get_cpu_info() -> impl IntoResponse {
    match read_cpu_info() {
        Ok(data) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", data)),
        ),
        Err(msg) => (
            StatusCode::OK,
            Json(ApiResponse::<CpuInfo>::error(msg)),
        ),
    }
}

/// GET /api/stats/system - 获取综合系统状态（包括网速、内存、运行时间）
///
/// 一次性获取所有系统监控信息，适合仪表板使用
///
/// # Response example
/// ```json
/// {
///   "status": "ok",
///   "message": "Success",
///   "data": {
///     "network_speed": { ... },
///     "memory": { ... },
///     "uptime": { ... }
///   }
/// }
/// ```
pub async fn get_system_stats() -> impl IntoResponse {
    use std::time::{Duration, Instant};
    use tokio::time::sleep;
    
    let result: Result<SystemStatsResponse, String> = async {
        // 获取网速和 CPU 使用率（并行异步采样）
        let interfaces = get_active_interfaces()?;
        let mut first_samples = Vec::new();
        for interface in &interfaces {
            match read_interface_stats(interface) {
                Ok((rx, tx)) => first_samples.push((interface.clone(), rx, tx)),
                Err(_) => continue,
            }
        }
        
        // 同时开始 CPU 采样
        let cpu_usage_future = sample_cpu_usage();
        
        let start = Instant::now();
        // 等待 1 秒采样网速（CPU 采样只需 200ms，会先完成）
        let cpu_usage = cpu_usage_future.await.unwrap_or(0.0);
        
        // 补足剩余时间到 1 秒
        let elapsed_so_far = start.elapsed();
        if elapsed_so_far < Duration::from_secs(1) {
            sleep(Duration::from_secs(1) - elapsed_so_far).await;
        }
        let elapsed = start.elapsed().as_secs_f64();
        
        let mut speed_data = Vec::new();
        for (interface, rx1, tx1) in first_samples {
            if let Ok((rx2, tx2)) = read_interface_stats(&interface) {
                let rx_speed = ((rx2.saturating_sub(rx1)) as f64 / elapsed) as u64;
                let tx_speed = ((tx2.saturating_sub(tx1)) as f64 / elapsed) as u64;
                
                speed_data.push(NetworkSpeed {
                    interface,
                    rx_bytes_per_sec: rx_speed,
                    tx_bytes_per_sec: tx_speed,
                    total_rx_bytes: rx2,
                    total_tx_bytes: tx2,
                });
            }
        }
        
        // 获取内存信息
        let (total, available, cached, buffers) = read_memory_info()?;
        let used = total.saturating_sub(available);
        let used_percent = if total > 0 {
            (used as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        
        // 获取磁盘信息
        let disk = read_disk_info();
        
        // 获取 CPU 负载（使用之前采样的 CPU 使用率）
        let mut cpu_load = read_cpu_load_sync().unwrap_or_default();
        cpu_load.load_percent = cpu_usage;
        
        // 获取运行时间
        let (uptime, idle) = read_uptime()?;
        let formatted = format_uptime(uptime);
        
        // 获取系统信息（uname）
        let system_info = read_system_info()?;
        
        // 获取温度
        let temperature = read_temperature_sensors();
        
        // 获取 USB 模式
        let usb_mode = match usb_switch::get_usb_mode_config() {
            Ok(config) => UsbModeResponse {
                current_mode: config.current_mode,
                current_mode_name: get_mode_name(config.current_mode),
                permanent_mode: config.permanent_mode,
                temporary_mode: config.temporary_mode,
                needs_reboot: true,
                read_mode: "hardware".to_string(),
            },
            Err(_) => UsbModeResponse::default(),
        };
        
        Ok(SystemStatsResponse {
            network_speed: NetworkSpeedResponse {
                interfaces: speed_data,
                interval_seconds: elapsed,
            },
            memory: MemoryInfo {
                total_bytes: total,
                available_bytes: available,
                used_bytes: used,
                used_percent,
                cached_bytes: cached,
                buffers_bytes: buffers,
            },
            disk,
            cpu_load,
            uptime: UptimeInfo {
                uptime_seconds: uptime,
                idle_seconds: idle,
                uptime_formatted: formatted,
            },
            system_info,
            temperature,
            usb_mode,
        })
    }
    .await;
    
    match result {
        Ok(data) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", data)),
        ),
        Err(msg) => (
            StatusCode::OK,
            Json(ApiResponse::<SystemStatsResponse>::error(msg)),
        ),
    }
}

/// GET /api/location/cell-info - 获取基站定位参数
/// 
/// 返回格式化的基站定位参数，可用于调用第三方定位API（如Google Geolocation、OpenCellID等）
pub async fn get_cell_location_info(State(conn): State<Arc<Connection>>) -> impl IntoResponse {
    // 获取网络信息（MCC、MNC）
    let network_info = match get_network_info_data(&conn).await {
        Ok(info) => info,
        Err(e) => {
            return (
                StatusCode::OK,
                Json(ApiResponse::<CellLocationResponse>::error(format!(
                    "Failed to get network info: {}",
                    e
                ))),
            );
        }
    };

    // 检查是否有 MCC 和 MNC
    let mcc = match network_info.mcc {
        Some(ref m) if !m.is_empty() => m.clone(),
        _ => {
            return (
                StatusCode::OK,
                Json(ApiResponse::success_with_message(
                    "Cell location unavailable: MCC not available",
                    CellLocationResponse {
                        available: false,
                        cell_info: None,
                        neighbor_cells: vec![],
                        usage_hint: "Network not registered or MCC/MNC not available. Please ensure device is connected to cellular network.".to_string(),
                    },
                )),
            );
        }
    };

    let mnc = match network_info.mnc {
        Some(ref m) if !m.is_empty() => m.clone(),
        _ => {
            return (
                StatusCode::OK,
                Json(ApiResponse::success_with_message(
                    "Cell location unavailable: MNC not available",
                    CellLocationResponse {
                        available: false,
                        cell_info: None,
                        neighbor_cells: vec![],
                        usage_hint: "Network not registered or MCC/MNC not available. Please ensure device is connected to cellular network.".to_string(),
                    },
                )),
            );
        }
    };

    // 获取服务小区信息（TAC、CID）
    let serving_cell = match get_serving_cell_info(&conn).await {
        Ok(cell) => cell,
        Err(e) => {
            return (
                StatusCode::OK,
                Json(ApiResponse::<CellLocationResponse>::error(format!(
                    "Failed to get serving cell info: {}",
                    e
                ))),
            );
        }
    };

    // 获取详细的小区信息（信号强度等）
    let tech = serving_cell.tech.as_str();
    let cmd_config = match get_cell_command_config(tech) {
        Some(config) => config,
        None => {
            // 如果不支持当前网络制式，返回基本信息（不含信号强度）
            let cell_info = if serving_cell.cell_id > 0 {
                Some(CellLocationInfo {
                    mcc: mcc.clone(),
                    mnc: mnc.clone(),
                    lac: serving_cell.tac,
                    cid: serving_cell.cell_id,
                    signal_strength: -100, // 默认信号强度
                    radio_type: serving_cell.tech.clone(),
                    arfcn: None,
                    pci: None,
                    rsrq: None,
                    sinr: None,
                })
            } else {
                None
            };

            let usage_hint = format!(
                "Cell location data available (limited). Unsupported network type: {}.\n\
                Network: {} (MCC={}, MNC={}), Cell ID={}, TAC={}",
                tech,
                network_info.operator_name,
                mcc,
                mnc,
                serving_cell.cell_id,
                serving_cell.tac
            );

            return (
                StatusCode::OK,
                Json(ApiResponse::success_with_message(
                    "Success (limited info)",
                    CellLocationResponse {
                        available: cell_info.is_some(),
                        cell_info,
                        neighbor_cells: vec![],
                        usage_hint,
                    },
                )),
            );
        }
    };

    // 获取主小区和邻区详细信息
    let serving_cell_detail = fetch_primary_cell(&conn, cmd_config.primary, tech).await.ok();
    let neighbor_cells = fetch_neighbor_cells(&conn, cmd_config.neighbor, tech).await.unwrap_or_default();

    // 构建主服务小区定位信息
    let cell_info = if serving_cell.cell_id > 0 {
        let signal_strength = if let Some(ref detail) = serving_cell_detail {
            // RSRP 是原始值×100，需要除以100
            detail.rsrp.parse::<i32>().unwrap_or(-140) / 100
        } else {
            -100 // 默认信号强度
        };

        let arfcn = serving_cell_detail.as_ref().and_then(|d| d.arfcn.parse::<u32>().ok());
        let pci = serving_cell_detail.as_ref().and_then(|d| d.pci.parse::<u32>().ok());
        let rsrq = serving_cell_detail.as_ref().and_then(|d| d.rsrq.parse::<i32>().ok().map(|v| v / 100));
        let sinr = serving_cell_detail.as_ref().and_then(|d| d.sinr.parse::<i32>().ok().map(|v| v / 100));

        Some(CellLocationInfo {
            mcc: mcc.clone(),
            mnc: mnc.clone(),
            lac: serving_cell.tac,
            cid: serving_cell.cell_id,
            signal_strength,
            radio_type: serving_cell.tech.clone(),
            arfcn,
            pci,
            rsrq,
            sinr,
        })
    } else {
        None
    };

    // 构建邻区定位信息列表
    let neighbor_location_cells: Vec<CellLocationInfo> = neighbor_cells
        .iter()
        .filter_map(|cell| {
            let signal_strength = cell.rsrp.parse::<i32>().unwrap_or(-140) / 100;
            let pci = cell.pci.parse::<u32>().ok()?;
            
            Some(CellLocationInfo {
                mcc: mcc.clone(),
                mnc: mnc.clone(),
                lac: serving_cell.tac, // 邻区通常与主小区在同一 TAC
                cid: 0, // 邻区可能没有完整的 CID，只有 PCI
                signal_strength,
                radio_type: cell.tech.clone(),
                arfcn: cell.arfcn.parse::<u32>().ok(),
                pci: Some(pci),
                rsrq: cell.rsrq.parse::<i32>().ok().map(|v| v / 100),
                sinr: cell.sinr.parse::<i32>().ok().map(|v| v / 100),
            })
        })
        .collect();

    // 构建使用建议
    let usage_hint = if cell_info.is_some() {
        format!(
            "Cell location data available. You can use this data with geolocation APIs:\n\
            - Google Geolocation API: https://developers.google.com/maps/documentation/geolocation/overview\n\
            - OpenCellID: https://opencellid.org/\n\
            - Unwired Labs: https://unwiredlabs.com/\n\
            Network: {} (MCC={}, MNC={}), Cell ID={}, TAC={}, Signal={}dBm",
            network_info.operator_name,
            mcc,
            mnc,
            serving_cell.cell_id,
            serving_cell.tac,
            cell_info.as_ref().map(|c| c.signal_strength).unwrap_or(-100)
        )
    } else {
        "Cell location unavailable: No serving cell found.".to_string()
    };

    let response = CellLocationResponse {
        available: cell_info.is_some(),
        cell_info,
        neighbor_cells: neighbor_location_cells,
        usage_hint,
    };

    (
        StatusCode::OK,
        Json(ApiResponse::success_with_message("Success", response)),
    )
}

/// GET /api/network/interfaces - 获取所有网络接口详细信息
/// 
/// 返回所有网络接口的详细信息，包括：
/// - 接口名称、状态、MAC地址、MTU
/// - IPv4和IPv6地址列表
/// - 公网/内网地址分类
/// - 流量统计（接收/发送字节数、包数、错误数）
pub async fn get_network_interfaces_info() -> impl IntoResponse {
    let result = tokio::task::spawn_blocking(|| {
        let interfaces = read_network_interfaces()?;
        let total_count = interfaces.len();
        
        Ok::<_, String>(NetworkInterfacesResponse {
            interfaces,
            total_count,
        })
    })
    .await;
    
    match result {
        Ok(Ok(data)) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", data)),
        ),
        Ok(Err(msg)) => (
            StatusCode::OK,
            Json(ApiResponse::<NetworkInterfacesResponse>::error(msg)),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<NetworkInterfacesResponse>::error(format!(
                "Task execution failed: {}",
                e
            ))),
        ),
    }
}

/// GET /api/radio-mode - 获取当前射频模式
///
/// # 返回
/// ```json
/// {
///   "status": "ok",
///   "message": "Success",
///   "data": {
///     "mode": "auto",
///     "technology_preference": "NR 5G/LTE auto"
///   }
/// }
/// ```
pub async fn get_radio_mode_handler(State(conn): State<Arc<Connection>>) -> impl IntoResponse {
    match get_radio_mode(&conn).await {
        Ok(data) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", data)),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<RadioModeResponse>::error(format!(
                "Failed to get radio mode: {}",
                e
            ))),
        ),
    }
}

/// POST /api/radio-mode - 设置射频模式
///
/// # 请求体
/// ```json
/// {
///   "mode": "auto"  // auto | lte | nr
/// }
/// ```
///
/// # 说明
/// - auto: 4G/5G 自动切换
/// - lte: 仅 4G LTE
/// - nr: 仅 5G NR
pub async fn set_radio_mode_handler(
    State(conn): State<Arc<Connection>>,
    Json(payload): Json<RadioModeRequest>,
) -> impl IntoResponse {
    match set_radio_mode(&conn, payload.mode.clone()).await {
        Ok(_) => {
            let mode_str = match payload.mode {
                RadioMode::Auto => "4G/5G Auto",
                RadioMode::LteOnly => "4G LTE Only",
                RadioMode::NrOnly => "5G NR Only",
            };
            (
                StatusCode::OK,
                Json(ApiResponse::success_with_message(
                    format!("Radio mode set to {}", mode_str),
                    json!({}),
                )),
            )
        }
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<serde_json::Value>::error(format!(
                "Failed to set radio mode: {}",
                e
            ))),
        ),
    }
}

/// GET /api/band-lock - 获取当前频段锁定状态
///
/// # 返回
/// ```json
/// {
///   "status": "ok",
///   "message": "Success",
///   "data": {
///     "locked": true,
///     "lte_fdd_bands": [1, 3, 8],
///     "lte_tdd_bands": [38, 40, 41],
///     "nr_fdd_bands": [1, 28],
///     "nr_tdd_bands": [41, 77, 78, 79]
///   }
/// }
/// ```
pub async fn get_band_lock_handler(State(conn): State<Arc<Connection>>) -> impl IntoResponse {
    // 读取 LTE 频段锁定状态
    let lte_result = send_at_command(&conn, "AT+SPLBAND=0").await;
    let (lte_fdd_mask, lte_tdd_mask, lte_raw) = match lte_result {
        Ok(response) => {
            let (fdd, tdd) = parse_splband_lte_response(&response);
            (fdd, tdd, Some(response))
        }
        Err(e) => (0, 0, Some(format!("Error: {}", e))),
    };

    // 读取 NR 频段锁定状态
    let nr_result = send_at_command(&conn, "AT+SPLBAND=3").await;
    let (nr_fdd_mask, nr_tdd_mask, nr_raw) = match nr_result {
        Ok(response) => {
            let (fdd, tdd) = parse_splband_nr_response(&response);
            (fdd, tdd, Some(response))
        }
        Err(e) => (0, 0, Some(format!("Error: {}", e))),
    };

    // 将位掩码转换为频段号列表
    let lte_fdd_bands = bitmask_to_bands(lte_fdd_mask, 1); // LTE FDD: B1-B16
    let lte_tdd_bands = bitmask_to_bands(lte_tdd_mask, 33); // LTE TDD: B33-B48
    let nr_fdd_bands = bitmask_to_bands(nr_fdd_mask, 100); // NR FDD: 展锐特殊映射 (N1, N3, N28)
    let nr_tdd_bands = bitmask_to_bands(nr_tdd_mask, 41); // NR TDD: 展锐特殊映射 (N41, N77, N78, N79)

    // UDX710 设备支持的全部频段掩码
    // LTE: FDD=149 (B1+B3+B5+B8), TDD=320 (B39+B41)
    // NR: FDD=517 (N1+N3+N28), TDD=912 (N41+N77+N78+N79)
    const LTE_FDD_ALL: u16 = 149;
    const LTE_TDD_ALL: u16 = 320;
    const NR_FDD_ALL: u16 = 517;
    const NR_TDD_ALL: u16 = 912;
    
    // 判断是否有频段锁定
    // 如果返回的频段等于设备支持的全部频段，则认为"未锁定"（全部可用）
    // 如果返回 0 或小于全部，则认为"已锁定"（限制了可用频段）
    let lte_is_all_or_zero = (lte_fdd_mask == LTE_FDD_ALL && lte_tdd_mask == LTE_TDD_ALL) 
                            || (lte_fdd_mask == 0 && lte_tdd_mask == 0);
    let nr_is_all_or_zero = (nr_fdd_mask == NR_FDD_ALL && nr_tdd_mask == NR_TDD_ALL)
                           || (nr_fdd_mask == 0 && nr_tdd_mask == 0);
    let locked = !(lte_is_all_or_zero && nr_is_all_or_zero);

    // 构建调试信息
    let raw_response = Some(format!(
        "LTE(fdd={},tdd={}): {}\nNR(fdd={},tdd={}): {}",
        lte_fdd_mask, lte_tdd_mask, lte_raw.unwrap_or_default().trim(),
        nr_fdd_mask, nr_tdd_mask, nr_raw.unwrap_or_default().trim()
    ));

    let status = BandLockStatus {
        locked,
        lte_fdd_bands,
        lte_tdd_bands,
        nr_fdd_bands,
        nr_tdd_bands,
        raw_response,
    };

    (
        StatusCode::OK,
        Json(ApiResponse::success_with_message("Success", status)),
    )
}

/// POST /api/system/reboot - 系统重启
///
/// # Request body（可选）
/// ```json
/// {
///   "delay_seconds": 3  // 延迟秒数，默认为3秒
/// }
/// ```
///
/// # Response example
/// ```json
/// {
///   "status": "ok",
///   "message": "System will reboot in 3 seconds"
/// }
/// ```
pub async fn system_reboot(
    Json(payload): Json<Option<SystemRebootRequest>>,
) -> impl IntoResponse {
    let delay = payload.map(|p| p.delay_seconds).unwrap_or(3);
    
    // 使用 tokio 异步执行重启命令
    tokio::spawn(async move {
        // 等待指定的延迟时间
        tokio::time::sleep(tokio::time::Duration::from_secs(delay as u64)).await;
        
        // 执行重启命令
        let _ = Command::new("reboot").output();
    });
    
    (
        StatusCode::OK,
        Json(ApiResponse::success_with_message(
            format!("系统将在 {} 秒后重启", delay),
            json!({"delay_seconds": delay}),
        )),
    )
}

/// POST /api/band-lock - 设置频段锁定
///
/// # 请求体
/// ```json
/// {
///   "lte_fdd_bands": [1, 3, 8],
///   "lte_tdd_bands": [38, 40, 41],
///   "nr_fdd_bands": [1, 28],
///   "nr_tdd_bands": [41, 77, 78, 79]
/// }
/// ```
///
/// # 说明
/// - 传入空数组表示不锁定对应类型的频段
/// - 所有数组都为空时，表示解除所有频段锁定
/// - LTE FDD: B1-B16, TDD: B33-B48
/// - NR FDD: N1-N16, TDD: N41-N56 (实际支持 N41-N79)
pub async fn set_band_lock_handler(
    State(conn): State<Arc<Connection>>,
    Json(payload): Json<BandLockRequest>,
) -> impl IntoResponse {
    // LTE 频段锁定
    let lte_fdd_mask = bands_to_bitmask(&payload.lte_fdd_bands, 1);
    let lte_tdd_mask = bands_to_bitmask(&payload.lte_tdd_bands, 33);
    
    if lte_fdd_mask != 0 || lte_tdd_mask != 0 {
        let lte_cmd = build_splband_lte_command(lte_fdd_mask, lte_tdd_mask);
        if let Err(e) = send_at_command(&conn, &lte_cmd).await {
            return (
                StatusCode::OK,
                Json(ApiResponse::<serde_json::Value>::error(format!(
                    "Failed to set LTE band lock: {}",
                    e
                ))),
            );
        }
    }

    // NR 频段锁定
    let nr_fdd_mask = bands_to_bitmask(&payload.nr_fdd_bands, 100); // NR FDD: 展锐特殊映射
    let nr_tdd_mask = bands_to_bitmask(&payload.nr_tdd_bands, 41);  // NR TDD: 展锐特殊映射
    
    if nr_fdd_mask != 0 || nr_tdd_mask != 0 {
        let nr_cmd = build_splband_nr_command(nr_fdd_mask, nr_tdd_mask);
        if let Err(e) = send_at_command(&conn, &nr_cmd).await {
            return (
                StatusCode::OK,
                Json(ApiResponse::<serde_json::Value>::error(format!(
                    "Failed to set NR band lock: {}",
                    e
                ))),
            );
        }
    }

    // 如果所有频段都为空，则解除锁定
    if payload.lte_fdd_bands.is_empty()
        && payload.lte_tdd_bands.is_empty()
        && payload.nr_fdd_bands.is_empty()
        && payload.nr_tdd_bands.is_empty()
    {
        let mut lte_unlocked = false;
        let mut nr_unlocked = false;
        
        // 先读取当前 LTE 锁定状态
        let lte_result = send_at_command(&conn, "AT+SPLBAND=0").await;
        if let Ok(lte_response) = lte_result {
            let (lte_fdd_mask, lte_tdd_mask) = parse_splband_lte_response(&lte_response);
            
            // 只有当前有 LTE 锁定时才执行解锁
            if lte_fdd_mask != 0 || lte_tdd_mask != 0 {
                // 格式: AT+SPLBAND=1,0,<TDD>,0,<FDD>,0 (6 参数)
                if let Err(e) = send_at_command(&conn, "AT+SPLBAND=1,0,0,0,0,0").await {
                    return (
                        StatusCode::OK,
                        Json(ApiResponse::<serde_json::Value>::error(format!(
                            "Failed to unlock LTE bands: {}",
                            e
                        ))),
                    );
                }
                lte_unlocked = true;
            }
        }
        
        // 先读取当前 NR 锁定状态
        let nr_result = send_at_command(&conn, "AT+SPLBAND=3").await;
        if let Ok(nr_response) = nr_result {
            let (nr_fdd_mask, nr_tdd_mask) = parse_splband_nr_response(&nr_response);
            
            // 只有当前有 NR 锁定时才执行解锁
            if nr_fdd_mask != 0 || nr_tdd_mask != 0 {
                if let Err(e) = send_at_command(&conn, "AT+SPLBAND=2,0,0,0,0").await {
                    return (
                        StatusCode::OK,
                        Json(ApiResponse::<serde_json::Value>::error(format!(
                            "Failed to unlock NR bands: {}",
                            e
                        ))),
                    );
                }
                nr_unlocked = true;
            }
        }
        
        // 根据实际执行的解锁操作返回友好的提示信息
        let message = if lte_unlocked || nr_unlocked {
            if lte_unlocked && nr_unlocked {
                "已解除所有频段锁定（LTE + NR）"
            } else if lte_unlocked {
                "已解除 LTE 频段锁定（NR 未锁定）"
            } else {
                "已解除 NR 频段锁定（LTE 未锁定）"
            }
        } else {
            "当前没有锁定的频段，无需解锁"
        };
        
        return (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                message,
                json!({}),
            )),
        );
    }

    // 生成友好的提示信息
    let has_lte = lte_fdd_mask != 0 || lte_tdd_mask != 0;
    let has_nr = nr_fdd_mask != 0 || nr_tdd_mask != 0;
    let message = if has_lte && has_nr {
        "已同时锁定 LTE 和 NR 频段"
    } else if has_lte {
        "LTE 频段锁定已应用"
    } else {
        "NR 频段锁定已应用"
    };

    (
        StatusCode::OK,
        Json(ApiResponse::success_with_message(
            message,
            json!({}),
        )),
    )
}

// ============ 小区锁定 API ============
// 使用 AT+SPFORCEFRQ 指令实现小区锁定
// 发现来源：通过 dbus-monitor 监听实际锁频操作

use crate::models::{CellLockStatusResponse, CellLockRatStatus, CellLockRequest, CellUnlockRequest};

/// SPFORCEFRQ 网络类型常量
const FORCEFRQ_TYPE_LTE: u8 = 12;
const FORCEFRQ_TYPE_NR: u8 = 16;

/// 获取 RAT 类型名称
fn get_rat_name(rat: u8) -> String {
    match rat {
        12 => "LTE".to_string(),
        16 => "NR".to_string(),
        _ => format!("Unknown({})", rat),
    }
}

/// 解析 AT+SPFORCEFRQ 查询响应
/// 
/// 响应格式:
/// - 未锁定: +SPFORCEFRQ: 16,3
/// - 已锁定: +SPFORCEFRQ: 16,3,633984,597
fn parse_spforcefrq_query_response(response: &str, rat: u8) -> CellLockRatStatus {
    let prefix = format!("+SPFORCEFRQ: {},3", rat);
    
    if let Some(line) = response.lines().find(|l| l.starts_with(&prefix)) {
        let data = line.strip_prefix(&format!("+SPFORCEFRQ: {},3", rat)).unwrap_or("");
        let data = data.trim_start_matches(',');
        
        if data.is_empty() {
            // 未锁定
            CellLockRatStatus {
                rat,
                rat_name: get_rat_name(rat),
                enabled: false,
                lock_type: 0,
                pci: None,
                arfcn: None,
            }
        } else {
            // 已锁定，解析 arfcn,pci
            let parts: Vec<&str> = data.split(',').collect();
            let arfcn = parts.first().and_then(|s| s.trim().parse::<u32>().ok());
            let pci = parts.get(1).and_then(|s| s.trim().parse::<u16>().ok());
            
            CellLockRatStatus {
                rat,
                rat_name: get_rat_name(rat),
                enabled: arfcn.is_some() && pci.is_some(),
                lock_type: 3,
                pci,
                arfcn,
            }
        }
    } else {
        // 解析失败，返回未锁定状态
        CellLockRatStatus {
            rat,
            rat_name: get_rat_name(rat),
            enabled: false,
            lock_type: 0,
            pci: None,
            arfcn: None,
        }
    }
}

/// GET /api/cell-lock - 获取小区锁定状态
/// 
/// 使用 AT+SPFORCEFRQ=<type>,3 查询锁定状态
/// 
/// ## 响应示例
/// ```json
/// {
///   "status": "ok",
///   "message": "Success",
///   "data": {
///     "rat_status": [
///       { "rat": 16, "rat_name": "NR", "enabled": true, "lock_type": 3, "pci": 597, "arfcn": 633984 }
///     ],
///     "any_locked": true
///   }
/// }
/// ```
pub async fn get_cell_lock_handler(State(conn): State<Arc<Connection>>) -> impl IntoResponse {
    let mut rat_status = Vec::new();
    let mut any_locked = false;
    
    // 查询 NR 锁定状态
    let nr_cmd = format!("AT+SPFORCEFRQ={},3", FORCEFRQ_TYPE_NR);
    if let Ok(response) = send_at_command(&conn, &nr_cmd).await {
        let status = parse_spforcefrq_query_response(&response, FORCEFRQ_TYPE_NR);
        if status.enabled {
            any_locked = true;
        }
        rat_status.push(status);
    } else {
        rat_status.push(CellLockRatStatus {
            rat: FORCEFRQ_TYPE_NR,
            rat_name: "NR".to_string(),
            enabled: false,
            lock_type: 0,
            pci: None,
            arfcn: None,
        });
    }
    
    // 查询 LTE 锁定状态
    let lte_cmd = format!("AT+SPFORCEFRQ={},3", FORCEFRQ_TYPE_LTE);
    if let Ok(response) = send_at_command(&conn, &lte_cmd).await {
        let status = parse_spforcefrq_query_response(&response, FORCEFRQ_TYPE_LTE);
        if status.enabled {
            any_locked = true;
        }
        rat_status.push(status);
    } else {
        rat_status.push(CellLockRatStatus {
            rat: FORCEFRQ_TYPE_LTE,
            rat_name: "LTE".to_string(),
            enabled: false,
            lock_type: 0,
            pci: None,
            arfcn: None,
        });
    }
    
    let response = CellLockStatusResponse {
        rat_status,
        any_locked,
    };
    
    (
        StatusCode::OK,
        Json(ApiResponse::success_with_message("Success", response)),
    )
}

/// POST /api/cell-lock - 设置小区锁定
/// 
/// 使用 AT+SPFORCEFRQ 指令锁定到指定小区
/// 
/// ## 锁定流程
/// 1. AT+SFUN=5 - 进入工程模式
/// 2. AT+SPFORCEFRQ=16,0 - 清空 NR 锁定
/// 3. AT+SPFORCEFRQ=12,0 - 清空 LTE 锁定
/// 4. AT+SPFORCEFRQ=<type>,2,<arfcn>,<pci> - 设置锁定
/// 5. AT+SFUN=4 - 恢复正常模式
/// 
/// ## 请求示例
/// ```json
/// {
///   "rat": 16,        // 16=NR, 12=LTE
///   "enable": true,
///   "pci": 599,
///   "arfcn": 633984
/// }
/// ```
pub async fn set_cell_lock_handler(
    State(conn): State<Arc<Connection>>,
    Json(payload): Json<CellLockRequest>,
) -> impl IntoResponse {
    // 确定网络类型
    let forcefrq_type = if payload.rat == FORCEFRQ_TYPE_LTE || payload.rat == FORCEFRQ_TYPE_NR {
        payload.rat
    } else {
        // 根据 rat 值推断类型
        match payload.rat {
            1 | 2 => FORCEFRQ_TYPE_LTE,  // LTE FDD/TDD
            5 | 6 | 7 => FORCEFRQ_TYPE_NR, // NR SA/NSA
            _ => FORCEFRQ_TYPE_NR,
        }
    };
    
    if payload.enable {
        // 锁定小区需要 ARFCN 和 PCI
        let (arfcn, pci) = match (payload.arfcn, payload.pci) {
            (Some(a), Some(p)) => (a, p),
            _ => {
                return (
                    StatusCode::OK,
                    Json(ApiResponse::<serde_json::Value>::error(
                        "锁定小区需要同时提供 arfcn 和 pci 参数"
                    )),
                );
            }
        };
        
        // 执行锁定流程
        let steps = vec![
            ("AT+SFUN=5", "进入工程模式"),
            ("AT+SPFORCEFRQ=16,0", "清空 NR 锁定"),
            ("AT+SPFORCEFRQ=12,0", "清空 LTE 锁定"),
        ];
        
        for (cmd, desc) in &steps {
            if let Err(e) = send_at_command(&conn, cmd).await {
                // 恢复正常模式
                let _ = send_at_command(&conn, "AT+SFUN=4").await;
                return (
                    StatusCode::OK,
                    Json(ApiResponse::<serde_json::Value>::error(format!(
                        "{}失败: {}",
                        desc, e
                    ))),
                );
            }
        }
        
        // 设置锁定
        let lock_cmd = format!("AT+SPFORCEFRQ={},2,{},{}", forcefrq_type, arfcn, pci);
        if let Err(e) = send_at_command(&conn, &lock_cmd).await {
            // 恢复正常模式
            let _ = send_at_command(&conn, "AT+SFUN=4").await;
            return (
                StatusCode::OK,
                Json(ApiResponse::<serde_json::Value>::error(format!(
                    "设置锁定失败: {}",
                    e
                ))),
            );
        }
        
        // 恢复正常模式
        if let Err(e) = send_at_command(&conn, "AT+SFUN=4").await {
            return (
                StatusCode::OK,
                Json(ApiResponse::<serde_json::Value>::error(format!(
                    "恢复正常模式失败: {}",
                    e
                ))),
            );
        }
        
        (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                format!("{} 小区锁定已设置 (ARFCN={}, PCI={})", get_rat_name(forcefrq_type), arfcn, pci),
                json!({
                    "locked": true,
                    "tech": get_rat_name(forcefrq_type),
                    "arfcn": arfcn,
                    "pci": pci
                }),
            )),
        )
    } else {
        // 解锁：清空指定类型的锁定
        // 1. 进入工程模式
        if let Err(e) = send_at_command(&conn, "AT+SFUN=5").await {
            return (
                StatusCode::OK,
                Json(ApiResponse::<serde_json::Value>::error(format!(
                    "进入工程模式失败: {}",
                    e
                ))),
            );
        }
        
        // 2. 清空锁定
        let clear_cmd = format!("AT+SPFORCEFRQ={},0", forcefrq_type);
        if let Err(e) = send_at_command(&conn, &clear_cmd).await {
            // 恢复正常模式
            let _ = send_at_command(&conn, "AT+SFUN=4").await;
            return (
                StatusCode::OK,
                Json(ApiResponse::<serde_json::Value>::error(format!(
                    "清空锁定失败: {}",
                    e
                ))),
            );
        }
        
        // 3. 恢复正常模式
        if let Err(e) = send_at_command(&conn, "AT+SFUN=4").await {
            return (
                StatusCode::OK,
                Json(ApiResponse::<serde_json::Value>::error(format!(
                    "恢复正常模式失败: {}",
                    e
                ))),
            );
        }
        
        (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                format!("{} 小区锁定已解除", get_rat_name(forcefrq_type)),
                json!({
                    "locked": false,
                    "tech": get_rat_name(forcefrq_type)
                }),
            )),
        )
    }
}

/// POST /api/cell-lock/unlock-all - 解除所有小区锁定
/// 
/// 清除 NR 和 LTE 的小区锁定
pub async fn unlock_all_cells_handler(
    State(conn): State<Arc<Connection>>,
    Json(_payload): Json<CellUnlockRequest>,
) -> impl IntoResponse {
    // 完整的解锁流程
    let steps = vec![
        ("AT+SFUN=5", "进入工程模式"),
        ("AT+SPFORCEFRQ=16,0", "清空 NR 锁定"),
        ("AT+SPFORCEFRQ=12,0", "清空 LTE 锁定"),
        ("AT+SFUN=4", "恢复正常模式"),
    ];
    
    let mut success_steps = Vec::new();
    let mut errors = Vec::new();
    
    for (cmd, desc) in &steps {
        match send_at_command(&conn, cmd).await {
            Ok(_) => success_steps.push(*desc),
            Err(e) => {
                errors.push(format!("{}: {}", desc, e));
                // 尝试恢复正常模式
                if !cmd.contains("SFUN=4") {
                    let _ = send_at_command(&conn, "AT+SFUN=4").await;
                }
                break;
            }
        }
    }
    
    if errors.is_empty() {
        (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "已解除所有小区锁定 (NR + LTE)",
                json!({
                    "success": true,
                    "steps": success_steps
                }),
            )),
        )
    } else {
        (
            StatusCode::OK,
            Json(ApiResponse::<serde_json::Value>::error(format!(
                "解锁失败: {}",
                errors.join("; ")
            ))),
        )
    }
}

// ============ 电话相关 API ============

use crate::db::Database;

/// GET /api/calls - 获取当前通话列表
pub async fn get_calls_handler(
    State(conn): State<Arc<Connection>>,
) -> (StatusCode, Json<ApiResponse<CallListResponse>>) {
    // 获取 VoiceCallManager 接口下的所有通话
    match crate::dbus::get_active_calls(&conn).await {
        Ok(calls) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", CallListResponse { calls })),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::error(format!("Failed to get calls: {}", e))),
        ),
    }
}


/// POST /api/call/dial - 拨打电话
pub async fn dial_call_handler(
    State(conn): State<Arc<Connection>>,
    Json(req): Json<MakeCallRequest>,
) -> (StatusCode, Json<ApiResponse<CallInfo>>) {
    match crate::dbus::dial_call(&conn, &req.phone_number).await {
        Ok(call_info) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Call initiated", call_info)),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::error(format!("Failed to dial: {}", e))),
        ),
    }
}

/// POST /api/call/hangup - 挂断电话
pub async fn hangup_call_handler(
    State(conn): State<Arc<Connection>>,
    Json(req): Json<HangupCallRequest>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    match crate::dbus::hangup_call(&conn, &req.path).await {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Call ended", json!({}))),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::error(format!("Failed to hangup: {}", e))),
        ),
    }
}

/// POST /api/call/hangup-all - 挂断所有电话
pub async fn hangup_all_calls_handler(
    State(conn): State<Arc<Connection>>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    match crate::dbus::hangup_all_calls(&conn).await {
        Ok(count) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                format!("Ended {} call(s)", count),
                json!({ "count": count }),
            )),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::error(format!("Failed to hangup all: {}", e))),
        ),
    }
}

/// POST /api/call/answer - 接听来电
pub async fn answer_call_handler(
    State(conn): State<Arc<Connection>>,
    Json(req): Json<HangupCallRequest>, // 复用结构，只需要 path
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    match crate::dbus::answer_call(&conn, &req.path).await {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Call answered", json!({}))),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::error(format!("Failed to answer: {}", e))),
        ),
    }
}

// ============ 短信相关 API ============

/// POST /api/sms/send - 发送短信
pub async fn send_sms_handler(
    State((conn, db)): State<(Arc<Connection>, Arc<Database>)>,
    Json(req): Json<SendSmsRequest>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    // 发送短信
    match crate::dbus::send_sms(&conn, &req.phone_number, &req.content).await {
        Ok(message_path) => {
            // 存储到数据库
            match db.insert_sms("outgoing", &req.phone_number, &req.content, "sent", None) {
                Ok(id) => (
                    StatusCode::OK,
                    Json(ApiResponse::success_with_message(
                        "SMS sent successfully",
                        json!({
                            "message_path": message_path,
                            "db_id": id,
                        }),
                    )),
                ),
                Err(_e) => {
                    (
                        StatusCode::OK,
                        Json(ApiResponse::success_with_message(
                            "SMS sent but failed to save to database",
                            json!({ "message_path": message_path }),
                        )),
                    )
                }
            }
        }
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::error(format!("Failed to send SMS: {}", e))),
        ),
    }
}

/// GET /api/sms/list - 获取短信列表
pub async fn get_sms_list_handler(
    State(db): State<Arc<Database>>,
    axum::extract::Query(req): axum::extract::Query<SmsListRequest>,
) -> (StatusCode, Json<ApiResponse<Vec<crate::db::SmsMessage>>>) {
    match db.get_sms_messages(req.limit, req.offset) {
        Ok(messages) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                format!("Retrieved {} messages", messages.len()),
                messages,
            )),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::error(format!("Failed to get messages: {}", e))),
        ),
    }
}

/// GET /api/sms/conversation - 获取与特定号码的对话历史
pub async fn get_sms_conversation_handler(
    State(db): State<Arc<Database>>,
    axum::extract::Query(req): axum::extract::Query<SmsConversationRequest>,
) -> (StatusCode, Json<ApiResponse<Vec<crate::db::SmsMessage>>>) {
    match db.get_sms_conversation(&req.phone_number, req.limit) {
        Ok(messages) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                format!("Retrieved {} messages", messages.len()),
                messages,
            )),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::error(format!("Failed to get conversation: {}", e))),
        ),
    }
}

/// GET /api/sms/stats - 获取短信统计
pub async fn get_sms_stats_handler(
    State(db): State<Arc<Database>>,
) -> (StatusCode, Json<ApiResponse<crate::db::SmsStats>>) {
    match db.get_sms_stats() {
        Ok(stats) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", stats)),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::error(format!("Failed to get stats: {}", e))),
        ),
    }
}

impl Default for crate::db::SmsStats {
    fn default() -> Self {
        Self {
            total: 0,
            incoming: 0,
            outgoing: 0,
        }
    }
}

/// DELETE /api/sms/clear - 清空所有短信
pub async fn clear_sms_handler(
    State(db): State<Arc<Database>>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    match db.clear_all_sms() {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("All messages cleared", json!({}))),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::error(format!("Failed to clear messages: {}", e))),
        ),
    }
}

// ============ 新增功能 API ============

/// GET /api/device/imeisv - 获取 IMEISV（软件版本号）
pub async fn get_imeisv_handler(
    State(conn): State<Arc<Connection>>,
) -> (StatusCode, Json<ApiResponse<crate::models::ImeisvResponse>>) {
    match crate::dbus::get_imeisv(&conn).await {
        Ok(imeisv) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", imeisv)),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::error(format!("Failed to get IMEISV: {}", e))),
        ),
    }
}

/// GET /api/network/signal-strength - 获取信号强度详细信息
pub async fn get_signal_strength_handler(
    State(conn): State<Arc<Connection>>,
) -> (StatusCode, Json<ApiResponse<crate::models::SignalStrengthResponse>>) {
    match crate::dbus::get_signal_strength(&conn).await {
        Ok(signal) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", signal)),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::error(format!("Failed to get signal strength: {}", e))),
        ),
    }
}

/// GET /api/network/nitz - 获取 NITZ 网络时间
pub async fn get_nitz_handler(
    State(conn): State<Arc<Connection>>,
) -> (StatusCode, Json<ApiResponse<crate::models::NitzTimeResponse>>) {
    match crate::dbus::get_nitz_time(&conn).await {
        Ok(nitz) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", nitz)),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::error(format!("Failed to get NITZ time: {}", e))),
        ),
    }
}

/// GET /api/ims/status - 获取 IMS 状态
pub async fn get_ims_status_handler(
    State(conn): State<Arc<Connection>>,
) -> (StatusCode, Json<ApiResponse<crate::models::ImsStatusResponse>>) {
    match crate::dbus::get_ims_status(&conn).await {
        Ok(ims) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", ims)),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::error(format!("Failed to get IMS status: {}", e))),
        ),
    }
}

/// GET /api/call/volume - 获取通话音量
pub async fn get_call_volume_handler(
    State(conn): State<Arc<Connection>>,
) -> (StatusCode, Json<ApiResponse<crate::models::CallVolumeResponse>>) {
    match crate::dbus::get_call_volume(&conn).await {
        Ok(volume) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", volume)),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::error(format!("Failed to get call volume: {}", e))),
        ),
    }
}

/// POST /api/call/volume - 设置通话音量
pub async fn set_call_volume_handler(
    State(conn): State<Arc<Connection>>,
    Json(req): Json<crate::models::SetCallVolumeRequest>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    match crate::dbus::set_call_volume(&conn, req.speaker_volume, req.microphone_volume, req.muted).await {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Call volume updated", json!({}))),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::error(format!("Failed to set call volume: {}", e))),
        ),
    }
}

/// GET /api/voicemail/status - 获取语音留言状态
pub async fn get_voicemail_status_handler(
    State(conn): State<Arc<Connection>>,
) -> (StatusCode, Json<ApiResponse<crate::models::VoicemailStatusResponse>>) {
    match crate::dbus::get_voicemail_status(&conn).await {
        Ok(voicemail) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", voicemail)),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::error(format!("Failed to get voicemail status: {}", e))),
        ),
    }
}

/// GET /api/network/operators - 获取运营商列表（快速）
pub async fn get_operators_handler(
    State(conn): State<Arc<Connection>>,
) -> (StatusCode, Json<ApiResponse<crate::models::OperatorListResponse>>) {
    match crate::dbus::get_operators(&conn).await {
        Ok(operators) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", operators)),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::error(format!("Failed to get operators: {}", e))),
        ),
    }
}

/// GET /api/network/operators/scan - 扫描所有运营商（慢，120秒）
pub async fn scan_operators_handler(
    State(conn): State<Arc<Connection>>,
) -> (StatusCode, Json<ApiResponse<crate::models::OperatorListResponse>>) {
    match crate::dbus::scan_operators(&conn).await {
        Ok(operators) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Scan completed", operators)),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::error(format!("Failed to scan operators: {}", e))),
        ),
    }
}

/// POST /api/network/register-manual - 手动注册运营商
pub async fn register_operator_manual_handler(
    State(conn): State<Arc<Connection>>,
    Json(req): Json<crate::models::ManualRegisterRequest>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    match crate::dbus::register_operator_manual(&conn, &req.mccmnc).await {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                format!("Registered to operator {}", req.mccmnc),
                json!({}),
            )),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::error(format!("Failed to register operator: {}", e))),
        ),
    }
}

/// POST /api/network/register-auto - 自动注册运营商
pub async fn register_operator_auto_handler(
    State(conn): State<Arc<Connection>>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    match crate::dbus::register_operator_auto(&conn).await {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Automatic registration initiated", json!({}))),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::error(format!("Failed to register automatically: {}", e))),
        ),
    }
}

/// GET /api/call/forwarding - 获取呼叫转移设置
pub async fn get_call_forwarding_handler(
    State(conn): State<Arc<Connection>>,
) -> (StatusCode, Json<ApiResponse<crate::models::CallForwardingResponse>>) {
    match crate::dbus::get_call_forwarding(&conn).await {
        Ok(forwarding) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", forwarding)),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::error(format!("Failed to get call forwarding: {}", e))),
        ),
    }
}

/// POST /api/call/forwarding - 设置呼叫转移
pub async fn set_call_forwarding_handler(
    State(conn): State<Arc<Connection>>,
    Json(req): Json<crate::models::SetCallForwardingRequest>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    match crate::dbus::set_call_forwarding(&conn, &req.forward_type, &req.number, req.timeout).await {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Call forwarding updated", json!({}))),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::error(format!("Failed to set call forwarding: {}", e))),
        ),
    }
}

/// GET /api/call/settings - 获取通话设置
pub async fn get_call_settings_handler(
    State(conn): State<Arc<Connection>>,
) -> (StatusCode, Json<ApiResponse<crate::models::CallSettingsResponse>>) {
    match crate::dbus::get_call_settings(&conn).await {
        Ok(settings) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", settings)),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::error(format!("Failed to get call settings: {}", e))),
        ),
    }
}

/// POST /api/call/settings - 设置通话设置
pub async fn set_call_settings_handler(
    State(conn): State<Arc<Connection>>,
    Json(req): Json<crate::models::SetCallSettingRequest>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    match crate::dbus::set_call_setting(&conn, &req.property, &req.value).await {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Call settings updated", json!({}))),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::error(format!("Failed to set call settings: {}", e))),
        ),
    }
}

// ============ SIM 卡槽功能 ============

/// GET /api/sim/slot - 获取 SIM 卡槽信息
pub async fn get_sim_slot_handler(
    State(conn): State<Arc<Connection>>,
) -> (StatusCode, Json<ApiResponse<crate::models::SimSlotResponse>>) {
    match crate::dbus::get_sim_slot(&conn).await {
        Ok(slot_info) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", slot_info)),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::error(format!("Failed to get SIM slot info: {}", e))),
        ),
    }
}

/// POST /api/sim/slot/switch - 切换 SIM 卡槽
pub async fn switch_sim_slot_handler(
    State(conn): State<Arc<Connection>>,
    Json(req): Json<crate::models::SwitchSimSlotRequest>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    match crate::dbus::switch_sim_slot(&conn, req.slot).await {
        Ok(response) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                format!("Switched to SIM slot {}", req.slot),
                json!({"response": response}),
            )),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::error(format!("Failed to switch SIM slot: {}", e))),
        ),
    }
}

// ============ APN 管理功能 ============

/// GET /api/apn - 获取 APN 列表
///
/// 返回所有 internet 类型的 APN context 配置
pub async fn get_apn_list_handler(
    State(conn): State<Arc<Connection>>,
) -> (StatusCode, Json<ApiResponse<ApnListResponse>>) {
    match get_all_apn_contexts(&conn).await {
        Ok(contexts) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "Success",
                ApnListResponse { contexts },
            )),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::error(format!("Failed to get APN list: {}", e))),
        ),
    }
}

/// POST /api/apn - 设置 APN 配置
///
/// # 请求体
/// ```json
/// {
///   "context_path": "/ril_0/context2",
///   "apn": "cbnet",
///   "protocol": "dual",
///   "username": "",
///   "password": "",
///   "auth_method": "chap"
/// }
/// ```
pub async fn set_apn_handler(
    State(conn): State<Arc<Connection>>,
    Json(req): Json<SetApnRequest>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    // 验证 context_path
    if req.context_path.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error("context_path is required")),
        );
    }
    
    // 调用 D-Bus 设置 APN 属性
    match set_apn_properties(
        &conn,
        &req.context_path,
        req.apn.as_deref(),
        req.protocol.as_deref(),
        req.username.as_deref(),
        req.password.as_deref(),
        req.auth_method.as_deref(),
    ).await {
        Ok(_) => {
            // 获取更新后的 APN 配置
            match get_all_apn_contexts(&conn).await {
                Ok(contexts) => {
                    // 找到刚刚修改的 context
                    let updated_context = contexts
                        .iter()
                        .find(|c| c.path == req.context_path)
                        .cloned();
                    
                    (
                        StatusCode::OK,
                        Json(ApiResponse::success_with_message(
                            "APN configuration updated successfully",
                            json!({
                                "updated_context": updated_context,
                            }),
                        )),
                    )
                }
                Err(_) => (
                    StatusCode::OK,
                    Json(ApiResponse::success_with_message(
                        "APN configuration updated successfully",
                        json!({}),
                    )),
                ),
            }
        }
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::error(format!("Failed to set APN: {}", e))),
        ),
    }
}

/// GET /api/connectivity - 联网检测
///
/// 通过 ping 检测 IPv4 和 IPv6 连通性
pub async fn get_connectivity_check() -> (StatusCode, Json<ApiResponse<ConnectivityCheckResponse>>) {
    let ipv4_result = ping_host("223.5.5.5", false);
    let ipv6_result = ping_host("2400:3200::1", true);
    
    let response = ConnectivityCheckResponse {
        ipv4: ipv4_result,
        ipv6: ipv6_result,
    };
    
    (
        StatusCode::OK,
        Json(ApiResponse::success_with_message("Connectivity check completed", response)),
    )
}

/// 执行 ping 检测
fn ping_host(target: &str, is_ipv6: bool) -> PingResult {
    let cmd = if is_ipv6 { "ping6" } else { "ping" };
    
    // 使用 -c 1 只发一个包，-W 2 设置超时 2 秒
    let output = Command::new(cmd)
        .args(["-c", "1", "-W", "2", target])
        .output();
    
    match output {
        Ok(result) => {
            if result.status.success() {
                // 解析延迟时间
                let stdout = String::from_utf8_lossy(&result.stdout);
                let latency = parse_ping_latency(&stdout);
                
                PingResult {
                    success: true,
                    latency_ms: latency,
                    target: target.to_string(),
                    error: None,
                }
            } else {
                let stderr = String::from_utf8_lossy(&result.stderr);
                PingResult {
                    success: false,
                    latency_ms: None,
                    target: target.to_string(),
                    error: Some(if stderr.is_empty() {
                        "Host unreachable".to_string()
                    } else {
                        stderr.trim().to_string()
                    }),
                }
            }
        }
        Err(e) => PingResult {
            success: false,
            latency_ms: None,
            target: target.to_string(),
            error: Some(format!("Failed to execute ping: {}", e)),
        },
    }
}

/// 从 ping 输出中解析延迟时间
fn parse_ping_latency(output: &str) -> Option<f64> {
    // 匹配 "time=XX.XX ms" 或 "time=XX ms"
    for line in output.lines() {
        if let Some(time_pos) = line.find("time=") {
            let after_time = &line[time_pos + 5..];
            // 找到数字部分
            let num_str: String = after_time
                .chars()
                .take_while(|c| c.is_ascii_digit() || *c == '.')
                .collect();
            if let Ok(latency) = num_str.parse::<f64>() {
                return Some(latency);
            }
        }
    }
    None
}

// ============ 通话记录 API ============

use crate::config::ConfigManager;
use crate::webhook::WebhookSender;

/// GET /api/call/history - 获取通话记录
pub async fn get_call_history_handler(
    State(db): State<Arc<Database>>,
    Query(params): Query<crate::models::CallHistoryRequest>,
) -> (StatusCode, Json<ApiResponse<crate::models::CallHistoryResponse>>) {
    let limit = if params.limit > 0 { params.limit } else { 50 };
    let offset = if params.offset >= 0 { params.offset } else { 0 };
    
    match db.get_call_history(limit, offset) {
        Ok(records) => {
            let stats = db.get_call_stats().unwrap_or_default();
            (
                StatusCode::OK,
                Json(ApiResponse::success_with_message(
                    "Success",
                    crate::models::CallHistoryResponse { records, stats },
                )),
            )
        }
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::error(format!("Failed to get call history: {}", e))),
        ),
    }
}

/// DELETE /api/call/history/{id} - 删除单条通话记录
pub async fn delete_call_history_handler(
    State(db): State<Arc<Database>>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    match db.delete_call(id) {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Call record deleted", json!({}))),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::error(format!("Failed to delete call record: {}", e))),
        ),
    }
}

/// POST /api/call/history/clear - 清空所有通话记录
pub async fn clear_call_history_handler(
    State(db): State<Arc<Database>>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    match db.clear_all_calls() {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("All call records cleared", json!({}))),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::error(format!("Failed to clear call history: {}", e))),
        ),
    }
}

// ============ Webhook 配置 API ============

/// GET /api/webhook/config - 获取 Webhook 配置
pub async fn get_webhook_config_handler(
    State(config_manager): State<Arc<ConfigManager>>,
) -> (StatusCode, Json<ApiResponse<crate::config::WebhookConfig>>) {
    let config = config_manager.get_webhook();
    (
        StatusCode::OK,
        Json(ApiResponse::success_with_message("Success", config)),
    )
}

/// POST /api/webhook/config - 设置 Webhook 配置
pub async fn set_webhook_config_handler(
    State(config_manager): State<Arc<ConfigManager>>,
    Json(webhook_config): Json<crate::config::WebhookConfig>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    match config_manager.set_webhook(webhook_config) {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Webhook config updated", json!({}))),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::error(format!("Failed to update webhook config: {}", e))),
        ),
    }
}

/// POST /api/webhook/test - 测试 Webhook 连接
pub async fn test_webhook_handler(
    State(webhook_sender): State<Arc<WebhookSender>>,
) -> (StatusCode, Json<ApiResponse<crate::models::WebhookTestResponse>>) {
    match webhook_sender.test_webhook().await {
        Ok(message) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "Webhook test successful",
                crate::models::WebhookTestResponse {
                    success: true,
                    message,
                },
            )),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "Webhook test failed",
                crate::models::WebhookTestResponse {
                    success: false,
                    message: e,
                },
            )),
        ),
    }
}

// ============ OTA 更新功能 ============

/// GET /api/ota/status - 获取 OTA 更新状态
pub async fn get_ota_status_handler() -> impl IntoResponse {
    let status = crate::ota::get_ota_status();
    (
        StatusCode::OK,
        Json(ApiResponse::success_with_message("Success", status)),
    )
}

/// POST /api/ota/upload - 上传 OTA 更新包
pub async fn upload_ota_handler(
    body: axum::body::Bytes,
) -> impl IntoResponse {
    match crate::ota::handle_ota_upload(&body) {
        Ok(response) => {
            let message = if response.validation.valid {
                "OTA package uploaded and validated"
            } else {
                "OTA package uploaded but validation failed"
            };
            (
                StatusCode::OK,
                Json(ApiResponse::success_with_message(message, response)),
            )
        }
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<crate::models::OtaUploadResponse>::error(format!(
                "Failed to process OTA package: {}",
                e
            ))),
        ),
    }
}

/// POST /api/ota/apply - 应用 OTA 更新
pub async fn apply_ota_handler(
    Json(req): Json<crate::models::OtaApplyRequest>,
) -> impl IntoResponse {
    match crate::ota::apply_ota_update(req.restart_now) {
        Ok(message) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(&message, json!({ "applied": true }))),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<serde_json::Value>::error(format!(
                "Failed to apply OTA update: {}",
                e
            ))),
        ),
    }
}

/// POST /api/ota/cancel - 取消待安装的更新
pub async fn cancel_ota_handler() -> impl IntoResponse {
    match crate::ota::cancel_pending_update() {
        Ok(()) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Pending update cancelled", json!({}))),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<serde_json::Value>::error(format!(
                "Failed to cancel update: {}",
                e
            ))),
        ),
    }
}

