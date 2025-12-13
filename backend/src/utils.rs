/*
 * @Author: 1orz cloudorzi@gmail.com
 * @Date: 2025-12-11 17:44:29
 * @LastEditors: 1orz cloudorzi@gmail.com
 * @LastEditTime: 2025-12-13 12:46:21
 * @FilePath: /udx710-backend/backend/src/utils.rs
 * @Description: 
 * 
 * Copyright (c) 2025 by 1orz, All Rights Reserved. 
 */
//! 工具函数模块
//! 
//! 包含 AT 指令解析、数据处理等工具函数

use crate::models::{CellInfo, IpAddress, NetworkInterfaceInfo};
use std::collections::HashMap;
use std::net::IpAddr;

/// 小区信息查询指令配置
#[derive(Debug, Clone)]
pub struct CellCommandConfig {
    /// 主小区查询指令
    pub primary: &'static str,
    /// 邻区查询指令
    pub neighbor: &'static str,
}

/// 获取指定网络制式的小区查询指令
///
/// # Arguments
/// * `tech` - 网络制式 ("nr" 或 "lte")
///
/// # Returns
/// 对应的指令配置，如果制式不支持则返回 None
pub fn get_cell_command_config(tech: &str) -> Option<CellCommandConfig> {
    let mut cmd_map = HashMap::new();
    
    // NR 5G 指令配置
    cmd_map.insert(
        "nr",
        CellCommandConfig {
            primary: "AT+SPENGMD=0,14,1",
            neighbor: "AT+SPENGMD=0,14,2",
        },
    );
    
    // LTE 4G 指令配置
    cmd_map.insert(
        "lte",
        CellCommandConfig {
            primary: "AT+SPENGMD=0,6,0",
            neighbor: "AT+SPENGMD=0,6,6",
        },
    );
    
    cmd_map.get(tech).cloned()
}

/// 根据 NR ARFCN 推算频段（返回完整格式如 n41）
/// 参考 3GPP TS 38.104
fn arfcn_to_nr_band(arfcn: u32) -> String {
    match arfcn {
        // N1 (2100 MHz FDD)
        422000..=434000 => "n1".to_string(),
        // N3 (1800 MHz FDD)
        361000..=376000 => "n3".to_string(),
        // N8 (900 MHz FDD)
        185000..=192000 => "n8".to_string(),
        // N28 (700 MHz FDD)
        151600..=160600 => "n28".to_string(),
        // N41 (2600 MHz TDD)
        499200..=537999 => "n41".to_string(),
        // N77 (3700 MHz TDD) - 与 N78 重叠，优先判断为 N78
        620000..=680000 => "n78".to_string(),
        // N79 (4700 MHz TDD)
        693334..=733333 => "n79".to_string(),
        _ => "".to_string(),
    }
}

/// 根据 LTE EARFCN 推算频段（返回完整格式如 B3）
/// 参考 3GPP TS 36.101
fn earfcn_to_lte_band(earfcn: u32) -> String {
    match earfcn {
        // B1 (2100 MHz FDD)
        0..=599 => "B1".to_string(),
        // B3 (1800 MHz FDD)
        1200..=1949 => "B3".to_string(),
        // B5 (850 MHz FDD)
        2400..=2649 => "B5".to_string(),
        // B7 (2600 MHz FDD)
        2750..=3449 => "B7".to_string(),
        // B8 (900 MHz FDD)
        3450..=3799 => "B8".to_string(),
        // B20 (800 MHz FDD)
        6150..=6449 => "B20".to_string(),
        // B28 (700 MHz FDD)
        9210..=9659 => "B28".to_string(),
        // B38 (2600 MHz TDD)
        37750..=38249 => "B38".to_string(),
        // B39 (1900 MHz TDD)
        38250..=38649 => "B39".to_string(),
        // B40 (2300 MHz TDD)
        38650..=39649 => "B40".to_string(),
        // B41 (2500 MHz TDD)
        39650..=41589 => "B41".to_string(),
        _ => "".to_string(),
    }
}

/// 将 AT 指令返回的字符串解析为二维数组
///
/// # Arguments
/// * `input` - AT 指令返回的原始字符串
///
/// # Returns
/// 解析后的二维字符串数组
pub fn parse_at_response_to_2d_vec(input: &str) -> Vec<Vec<String>> {
    let cleaned_str: String = input
        .trim()
        .trim_end_matches("OK")
        .replace(['\r', '\n'], "");

    let mut result: Vec<Vec<String>> = Vec::new();
    let mut current_part = String::new();
    let mut chars = cleaned_str.chars().peekable();
    let mut prev_char: Option<char> = None;

    while let Some(c) = chars.next() {
        match c {
            '-' => match prev_char {
                Some(',') => current_part.push(c),
                _ => {
                    if !current_part.is_empty() {
                        result.push(
                            current_part
                                .trim()
                                .split(',')
                                .map(|s| s.to_string())
                                .collect(),
                        );
                        current_part.clear();
                    }
                    if let Some('-') = chars.peek() {
                        current_part.push('-');
                        chars.next();
                    }
                }
            },
            _ => current_part.push(c),
        }
        prev_char = Some(c);
    }

    if !current_part.is_empty() {
        result.push(current_part.split(',').map(|s| s.to_string()).collect());
    }

    result
}

/// 解析主小区信息
///
/// # Arguments
/// * `tech` - 网络制式 (nr/lte)
/// * `parsed_data` - 解析后的二维数组
///
/// # Returns
/// 主小区信息结构
///
/// # AT+SPENGMD 数据格式说明
/// 
/// **NR (5G) 主小区 (AT+SPENGMD=0,14,1):**
/// - `[0]`: Band (频段)
/// - `[1]`: ARFCN (绝对频点号)
/// - `[2]`: PCI (物理小区标识)
/// - `[3]`: RSRP (参考信号接收功率，原始值 ×100)
/// - `[4]`: RSRQ (参考信号接收质量，原始值 ×100)
/// - `[15]`: SINR (信号与干扰加噪声比，原始值 ×100)
///
/// **LTE (4G) 主小区 (AT+SPENGMD=0,6,0):**
/// - `[0]`: Band (频段)
/// - `[1]`: ARFCN (绝对频点号)
/// - `[2]`: PCI (物理小区标识)
/// - `[3]`: RSRP (参考信号接收功率，原始值 ×100)
/// - `[4]`: RSRQ (参考信号接收质量，原始值 ×100)
/// - `[33]`: SINR (信号与干扰加噪声比，原始值 ×100)
pub fn parse_primary_cell(tech: &str, parsed_data: &[Vec<String>]) -> CellInfo {
    let mut cell_info = CellInfo {
        is_serving: true, // 主小区标记
        ..Default::default()
    };
    
    match tech {
        "nr" => {
            if parsed_data.len() >= 16 {
                cell_info.tech = tech.to_string();
                // 给 NR 频段加 n 前缀
                let raw_band = parsed_data[0].join(",");
                cell_info.band = if !raw_band.is_empty() && raw_band != "0" {
                    format!("n{}", raw_band)
                } else {
                    raw_band
                };
                cell_info.arfcn = parsed_data[1].join(",");
                cell_info.pci = parsed_data[2].first().cloned().unwrap_or_default();
                // 返回原始值×100，不做除法，让前端处理单位转换
                cell_info.rsrp = parsed_data[3].first().cloned().unwrap_or_default();
                cell_info.rsrq = parsed_data[4].first().cloned().unwrap_or_default();
                cell_info.sinr = parsed_data[15].first().cloned().unwrap_or_default();
            }
        }
        "lte" => {
            if parsed_data.len() >= 34 {
                cell_info.tech = tech.to_string();
                // 给 LTE 频段加 B 前缀
                let raw_band = parsed_data[0].join(",");
                cell_info.band = if !raw_band.is_empty() && raw_band != "0" {
                    format!("B{}", raw_band)
                } else {
                    raw_band
                };
                cell_info.arfcn = parsed_data[1].join(",");
                cell_info.pci = parsed_data[2].join(",");
                // 返回原始值×100，不做除法，让前端处理单位转换
                cell_info.rsrp = parsed_data[3].first().cloned().unwrap_or_default();
                cell_info.rsrq = parsed_data[4].first().cloned().unwrap_or_default();
                cell_info.sinr = parsed_data[33].first().cloned().unwrap_or_default();
            }
        }
        _ => {}
    }
    
    cell_info
}

/// 解析邻区信息列表
///
/// # Arguments
/// * `tech` - 网络制式 (nr/lte)
/// * `parsed_data` - 解析后的二维数组
///
/// # Returns
/// 邻区信息列表
///
/// # AT+SPENGMD 邻区数据格式说明
///
/// **NR (5G) 邻区 (AT+SPENGMD=0,14,2):**
/// - `[0][i]`: Band (频段，第i个邻区)
/// - `[1][i]`: ARFCN (绝对频点号)
/// - `[2][i]`: PCI (物理小区标识)
/// - `[3][i]`: RSRP (参考信号接收功率，原始值 ×100)
/// - `[4][i]`: RSRQ (参考信号接收质量，原始值 ×100)
/// - `[5][i]`: SINR (信号与干扰加噪声比，原始值 ×100)
///
/// **LTE (4G) 邻区 (AT+SPENGMD=0,6,6):**
/// - `row[0]`: ARFCN (绝对频点号)
/// - `row[1]`: PCI (物理小区标识)
/// - `row[2]`: RSRP (参考信号接收功率，原始值 ×100)
/// - `row[3]`: RSRQ (参考信号接收质量，原始值 ×100)
/// - `row[12]`: Band (频段，如果存在)
pub fn parse_neighbor_cells(tech: &str, parsed_data: &[Vec<String>]) -> Vec<CellInfo> {
    let mut result = Vec::new();
    
    match tech {
        "nr" => {
            if parsed_data.is_empty() {
                return result;
            }
            
            let count = parsed_data[0].len();
            for i in 0..count {
                if parsed_data.len() < 6 {
                    break;
                }
                
                let arfcn = parsed_data[1].get(i).map(|s| s.as_str()).unwrap_or("0");
                let pci = parsed_data[2].get(i).map(|s| s.as_str()).unwrap_or("0");
                
                // 如果 arfcn 和 pci 都是 0，说明后续没有有效数据
                if arfcn == "0" && pci == "0" {
                    break;
                }
                
                // 尝试从数据中获取频段，如果为空或"0"则通过 ARFCN 推算
                let raw_band = parsed_data[0].get(i).cloned().unwrap_or_default();
                let band = if raw_band.is_empty() || raw_band == "0" {
                    // 通过 ARFCN 推算频段
                    arfcn.parse::<u32>()
                        .map(arfcn_to_nr_band)
                        .unwrap_or_default()
                } else {
                    raw_band
                };
                
                let cell = CellInfo {
                    is_serving: false, // 邻区标记
                    tech: tech.to_string(),
                    band,
                    arfcn: arfcn.to_string(),
                    pci: pci.to_string(),
                    // 返回原始值×100，不做除法，让前端处理单位转换
                    rsrp: parsed_data[3].get(i).cloned().unwrap_or_default(),
                    rsrq: parsed_data[4].get(i).cloned().unwrap_or_default(),
                    sinr: parsed_data[5].get(i).cloned().unwrap_or_default(),
                };
                
                result.push(cell);
            }
        }
        "lte" => {
            for row in parsed_data {
                if row.len() < 4 {
                    continue;
                }
                
                // 如果 arfcn 和 pci 都是 0，说明空行
                if row[0] == "0" && row[1] == "0" {
                    break;
                }
                
                // 尝试从数据中获取频段，如果不存在或为"0"则通过 EARFCN 推算
                let raw_band = if row.len() > 12 { row[12].clone() } else { String::new() };
                let band = if raw_band.is_empty() || raw_band == "0" {
                    // 通过 EARFCN 推算频段
                    row[0].parse::<u32>()
                        .map(earfcn_to_lte_band)
                        .unwrap_or_default()
                } else {
                    raw_band
                };
                
                let cell = CellInfo {
                    is_serving: false, // 邻区标记
                    tech: tech.to_string(),
                    band,
                    arfcn: row[0].clone(),
                    pci: row[1].clone(),
                    // 返回原始值×100，不做除法，让前端处理单位转换
                    rsrp: row[2].clone(),
                    rsrq: row[3].clone(),
                    sinr: "-".to_string(),  // LTE邻区不提供SINR
                };
                
                result.push(cell);
            }
        }
        _ => {
            // 不支持的网络类型，返回空列表
        }
    }
    
    result
}

/// 从 /proc/meminfo 读取内存信息
///
/// # Returns
/// (total, available, cached, buffers) in bytes
pub fn read_memory_info() -> Result<(u64, u64, u64, u64), String> {
    use std::fs;
    
    let content = fs::read_to_string("/proc/meminfo")
        .map_err(|e| format!("Failed to read /proc/meminfo: {}", e))?;
    
    let mut total = 0u64;
    let mut available = 0u64;
    let mut cached = 0u64;
    let mut buffers = 0u64;
    
    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }
        
        let value = parts[1].parse::<u64>().unwrap_or(0) * 1024; // Convert KB to bytes
        
        match parts[0] {
            "MemTotal:" => total = value,
            "MemAvailable:" => available = value,
            "Cached:" => cached = value,
            "Buffers:" => buffers = value,
            _ => {}
        }
    }
    
    Ok((total, available, cached, buffers))
}

/// 读取磁盘/分区使用情况
///
/// 自适应检测分区，去重处理（相同设备的多个挂载点只保留一个）
///
/// # Returns
/// 包含各个分区信息的 Vec<DiskInfo>
pub fn read_disk_info() -> Vec<crate::models::DiskInfo> {
    use std::collections::HashMap;
    use std::ffi::CString;
    use std::fs;
    
    // 读取 /proc/mounts
    let mounts = match fs::read_to_string("/proc/mounts") {
        Ok(content) => content,
        Err(_) => return Vec::new(),
    };
    
    // 用于设备去重：设备名 -> (挂载点, 文件系统类型, 优先级)
    // 优先级越低越优先显示
    let mut device_map: HashMap<String, (String, String, u8)> = HashMap::new();
    
    // 挂载点优先级（数字越小优先级越高）
    let get_priority = |mount: &str| -> u8 {
        match mount {
            "/" => 0,
            "/home" => 1,
            "/mnt/userdata" => 2,
            "/var" => 3,
            "/run" => 4,
            "/tmp" => 5,
            _ if mount.starts_with("/mnt/") => 10,
            _ if mount.starts_with("/var/") => 15,
            _ => 20,
        }
    };
    
    // 跳过的虚拟文件系统和挂载点
    let skip_fs = ["proc", "sysfs", "devtmpfs", "devpts", "cgroup", "cgroup2", 
        "pstore", "bpf", "tracefs", "debugfs", "securityfs", "configfs", 
        "fusectl", "hugetlbfs", "mqueue", "rpc_pipefs", "autofs", "functionfs"];
    
    let skip_mounts = ["/dev", "/dev/pts", "/sys", "/proc", "/sys/kernel/config",
        "/dev/usb-ffs/adb"];
    
    for line in mounts.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            continue;
        }
        
        let device = parts[0];
        let mount_point = parts[1];
        let fs_type = parts[2];
        
        // 跳过虚拟文件系统
        if skip_fs.contains(&fs_type) {
            continue;
        }
        
        // 跳过特定挂载点
        if skip_mounts.contains(&mount_point) {
            continue;
        }
        
        let priority = get_priority(mount_point);
        
        // 设备去重：同一设备保留优先级最高的挂载点
        let key = device.to_string();
        if let Some((_, _, existing_priority)) = device_map.get(&key) {
            if priority >= *existing_priority {
                continue; // 已有更高优先级的挂载点
            }
        }
        
        device_map.insert(key, (mount_point.to_string(), fs_type.to_string(), priority));
    }
    
    // 收集磁盘信息
    let mut disks = Vec::new();
    
    for (_, (mount_point, fs_type, _)) in device_map {
        let c_path = match CString::new(mount_point.as_str()) {
            Ok(p) => p,
            Err(_) => continue,
        };
        
        let mut stat: libc::statvfs = unsafe { std::mem::zeroed() };
        let result = unsafe { libc::statvfs(c_path.as_ptr(), &mut stat) };
        
        if result != 0 {
            continue;
        }
        
        let block_size = stat.f_frsize as u64;
        let total = stat.f_blocks as u64 * block_size;
        let available = stat.f_bavail as u64 * block_size;
        let free = stat.f_bfree as u64 * block_size;
        let used = total.saturating_sub(free);
        
        // 跳过太小的分区（< 1MB）
        if total < 1024 * 1024 {
            continue;
        }
        
        let used_percent = (used as f64 / total as f64) * 100.0;
        
        disks.push(crate::models::DiskInfo {
            mount_point,
            fs_type,
            total_bytes: total,
            used_bytes: used,
            available_bytes: available,
            used_percent,
        });
    }
    
    // 按挂载点排序：根目录优先，然后按名称
    disks.sort_by(|a, b| {
        let pa = get_priority(&a.mount_point);
        let pb = get_priority(&b.mount_point);
        if pa != pb {
            pa.cmp(&pb)
        } else {
            a.mount_point.cmp(&b.mount_point)
        }
    });
    
    disks
}

/// 从 /proc/uptime 读取系统运行时间
///
/// # Returns
/// (uptime_seconds, idle_seconds)
pub fn read_uptime() -> Result<(u64, u64), String> {
    use std::fs;
    
    let content = fs::read_to_string("/proc/uptime")
        .map_err(|e| format!("Failed to read /proc/uptime: {}", e))?;
    
    let parts: Vec<&str> = content.trim().split_whitespace().collect();
    if parts.len() < 2 {
        return Err("Invalid /proc/uptime format".to_string());
    }
    
    let uptime = parts[0]
        .parse::<f64>()
        .map_err(|e| format!("Failed to parse uptime: {}", e))? as u64;
    
    let idle = parts[1]
        .parse::<f64>()
        .map_err(|e| format!("Failed to parse idle time: {}", e))? as u64;
    
    Ok((uptime, idle))
}

/// 格式化运行时间为人类可读格式
///
/// # Arguments
/// * `seconds` - 总秒数
///
/// # Returns
/// 格式化的字符串，如 "2天 3小时 45分钟"
pub fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    
    let mut parts = Vec::new();
    
    if days > 0 {
        parts.push(format!("{}天", days));
    }
    if hours > 0 {
        parts.push(format!("{}小时", hours));
    }
    if minutes > 0 {
        parts.push(format!("{}分钟", minutes));
    }
    if parts.is_empty() || secs > 0 {
        parts.push(format!("{}秒", secs));
    }
    
    parts.join(" ")
}

/// 读取网络接口的流量统计
///
/// # Arguments
/// * `interface` - 网络接口名称（如 usb0, eth0）
///
/// # Returns
/// (rx_bytes, tx_bytes)
pub fn read_interface_stats(interface: &str) -> Result<(u64, u64), String> {
    use std::fs;
    
    let rx_path = format!("/sys/class/net/{}/statistics/rx_bytes", interface);
    let tx_path = format!("/sys/class/net/{}/statistics/tx_bytes", interface);
    
    let rx_bytes = fs::read_to_string(&rx_path)
        .map_err(|e| format!("Failed to read {}: {}", rx_path, e))?
        .trim()
        .parse::<u64>()
        .map_err(|e| format!("Failed to parse rx_bytes: {}", e))?;
    
    let tx_bytes = fs::read_to_string(&tx_path)
        .map_err(|e| format!("Failed to read {}: {}", tx_path, e))?
        .trim()
        .parse::<u64>()
        .map_err(|e| format!("Failed to parse tx_bytes: {}", e))?;
    
    Ok((rx_bytes, tx_bytes))
}

/// 获取所有活跃的网络接口列表
///
/// # Returns
/// 网络接口名称列表（排除 lo）
pub fn get_active_interfaces() -> Result<Vec<String>, String> {
    use std::fs;
    
    let entries = fs::read_dir("/sys/class/net")
        .map_err(|e| format!("Failed to read /sys/class/net: {}", e))?;
    
    let mut interfaces = Vec::new();
    
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let name = entry.file_name().to_string_lossy().to_string();
        
        // 排除回环接口
        if name != "lo" {
            // 检查接口是否 up
            let operstate_path = format!("/sys/class/net/{}/operstate", name);
            if let Ok(state) = fs::read_to_string(&operstate_path) {
                let state = state.trim();
                // 包含 up 和 unknown 状态的接口（unknown 可能是某些虚拟接口）
                if state == "up" || state == "unknown" {
                    interfaces.push(name);
                }
            }
        }
    }
    
    Ok(interfaces)
}

/// 从 /proc/stat 解析 CPU 时间
/// 返回 (total, idle)
fn parse_cpu_stat() -> Result<(u64, u64), String> {
    use std::fs;
    
    let stat = fs::read_to_string("/proc/stat")
        .map_err(|e| format!("Failed to read /proc/stat: {}", e))?;
    
    for line in stat.lines() {
        if line.starts_with("cpu ") {
            let values: Vec<u64> = line
                .split_whitespace()
                .skip(1) // 跳过 "cpu"
                .filter_map(|s| s.parse::<u64>().ok())
                .collect();
            
            if values.len() >= 4 {
                // user + nice + system + idle + iowait + irq + softirq + steal
                let user = values.first().copied().unwrap_or(0);
                let nice = values.get(1).copied().unwrap_or(0);
                let system = values.get(2).copied().unwrap_or(0);
                let idle = values.get(3).copied().unwrap_or(0);
                let iowait = values.get(4).copied().unwrap_or(0);
                let irq = values.get(5).copied().unwrap_or(0);
                let softirq = values.get(6).copied().unwrap_or(0);
                let steal = values.get(7).copied().unwrap_or(0);
                
                let total = user + nice + system + idle + iowait + irq + softirq + steal;
                let idle_total = idle + iowait;
                
                return Ok((total, idle_total));
            }
        }
    }
    
    Err("Failed to parse /proc/stat".to_string())
}

/// 从 /proc/loadavg 读取负载信息，CPU 使用率需要异步采样
///
/// # Returns
/// CpuLoadInfo 结构（不含实时 CPU 使用率）
pub fn read_cpu_load_sync() -> Result<crate::models::CpuLoadInfo, String> {
    use std::fs;
    use crate::models::CpuLoadInfo;
    
    // 读取 /proc/loadavg 获取负载平均值
    let loadavg = fs::read_to_string("/proc/loadavg")
        .map_err(|e| format!("Failed to read /proc/loadavg: {}", e))?;
    
    let parts: Vec<&str> = loadavg.split_whitespace().collect();
    let load_1min = parts.first().and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
    let load_5min = parts.get(1).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
    let load_15min = parts.get(2).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
    
    // 获取 CPU 核心数
    let core_count = std::thread::available_parallelism()
        .map(|p| p.get() as u32)
        .unwrap_or(1);
    
    Ok(CpuLoadInfo {
        load_1min,
        load_5min,
        load_15min,
        core_count,
        load_percent: 0.0, // 需要异步采样
    })
}

/// 异步采样 CPU 使用率（需要两次采样计算差值）
///
/// # Returns
/// CPU 使用率百分比 (0.0 - 100.0)
pub async fn sample_cpu_usage() -> Result<f64, String> {
    use tokio::time::{sleep, Duration};
    
    // 第一次采样
    let (total1, idle1) = parse_cpu_stat()?;
    
    // 等待 200ms
    sleep(Duration::from_millis(200)).await;
    
    // 第二次采样
    let (total2, idle2) = parse_cpu_stat()?;
    
    // 计算差值
    let total_diff = total2.saturating_sub(total1);
    let idle_diff = idle2.saturating_sub(idle1);
    
    if total_diff == 0 {
        return Ok(0.0);
    }
    
    // 计算 CPU 使用率
    let usage = ((total_diff - idle_diff) as f64 / total_diff as f64) * 100.0;
    
    Ok(usage.clamp(0.0, 100.0))
}

/// 从 /proc/cpuinfo 读取 CPU 信息
///
/// # Returns
/// CpuInfo 结构
pub fn read_cpu_info() -> Result<crate::models::CpuInfo, String> {
    use std::fs;
    use crate::models::{CpuInfo, CpuCore};
    
    let content = fs::read_to_string("/proc/cpuinfo")
        .map_err(|e| format!("Failed to read /proc/cpuinfo: {}", e))?;
    
    let mut cores = Vec::new();
    let mut current_core = CpuCore::default();
    let mut hardware = String::new();
    let mut serial = String::new();
    
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            // 空行表示一个 processor 块结束
            if current_core.processor > 0 || !current_core.bogomips.is_empty() {
                cores.push(current_core.clone());
                current_core = CpuCore::default();
            }
            continue;
        }
        
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            let value = value.trim();
            
            match key {
                "processor" => {
                    if let Ok(num) = value.parse::<u32>() {
                        current_core.processor = num;
                    }
                }
                "BogoMIPS" => {
                    current_core.bogomips = value.to_string();
                }
                "Features" => {
                    current_core.features = value
                        .split_whitespace()
                        .map(|s| s.to_string())
                        .collect();
                }
                "CPU implementer" => {
                    current_core.implementer = value.to_string();
                }
                "CPU architecture" => {
                    current_core.architecture = value.to_string();
                }
                "CPU variant" => {
                    current_core.variant = value.to_string();
                }
                "CPU part" => {
                    current_core.part = value.to_string();
                }
                "CPU revision" => {
                    current_core.revision = value.to_string();
                }
                "Hardware" => {
                    hardware = value.to_string();
                }
                "Serial" => {
                    serial = value.to_string();
                }
                _ => {}
            }
        }
    }
    
    // 处理最后一个核心（如果文件不以空行结尾）
    if current_core.processor > 0 || !current_core.bogomips.is_empty() {
        cores.push(current_core);
    }
    
    // 识别 CPU 型号
    let model_name = if !cores.is_empty() {
        identify_cpu_model(&cores[0].implementer, &cores[0].part)
    } else {
        "Unknown".to_string()
    };
    
    Ok(CpuInfo {
        core_count: cores.len() as u32,
        cores,
        hardware,
        serial,
        model_name,
    })
}

/// 从 uname 系统调用读取系统信息
///
/// # Returns
/// SystemInfo 结构
pub fn read_system_info() -> Result<crate::models::SystemInfo, String> {
    use crate::models::SystemInfo;
    use std::ffi::CStr;
    
    unsafe {
        let mut utsname: libc::utsname = std::mem::zeroed();
        
        if libc::uname(&mut utsname) != 0 {
            return Err("Failed to call uname system call".to_string());
        }
        
        // 将 C 字符串转换为 Rust String
        let sysname = CStr::from_ptr(utsname.sysname.as_ptr())
            .to_string_lossy()
            .to_string();
        
        let nodename = CStr::from_ptr(utsname.nodename.as_ptr())
            .to_string_lossy()
            .to_string();
        
        let release = CStr::from_ptr(utsname.release.as_ptr())
            .to_string_lossy()
            .to_string();
        
        let version = CStr::from_ptr(utsname.version.as_ptr())
            .to_string_lossy()
            .to_string();
        
        let machine = CStr::from_ptr(utsname.machine.as_ptr())
            .to_string_lossy()
            .to_string();
        
        // 注意：domainname 字段在某些平台上不可用，这里留空
        let domainname = String::new();
        
        // 构造类似 uname -a 的完整输出
        let full_info = format!(
            "{} {} {} {} {}",
            sysname, nodename, release, version, machine
        );
        
        Ok(SystemInfo {
            sysname,
            nodename,
            release,
            version,
            machine,
            domainname,
            full_info,
        })
    }
}

/// 根据 implementer 和 part 识别 CPU 型号
///
/// # Arguments
/// * `implementer` - CPU 实现者 ID（如 0x41 表示 ARM）
/// * `part` - CPU 部件号（如 0xd05 表示 Cortex-A55）
///
/// # Returns
/// CPU 型号名称
fn identify_cpu_model(implementer: &str, part: &str) -> String {
    // ARM implementer (0x41)
    if implementer == "0x41" {
        return match part {
            "0xd05" => "ARM Cortex-A55".to_string(),
            "0xd0a" => "ARM Cortex-A75".to_string(),
            "0xd0b" => "ARM Cortex-A76".to_string(),
            "0xd0c" => "ARM Neoverse N1".to_string(),
            "0xd0d" => "ARM Cortex-A77".to_string(),
            "0xd0e" => "ARM Cortex-A76AE".to_string(),
            "0xd40" => "ARM Neoverse V1".to_string(),
            "0xd41" => "ARM Cortex-A78".to_string(),
            "0xd44" => "ARM Cortex-X1".to_string(),
            "0xd46" => "ARM Cortex-A510".to_string(),
            "0xd47" => "ARM Cortex-A710".to_string(),
            "0xd48" => "ARM Cortex-X2".to_string(),
            "0xd49" => "ARM Neoverse N2".to_string(),
            "0xd4a" => "ARM Neoverse E1".to_string(),
            "0xd4b" => "ARM Cortex-A78AE".to_string(),
            "0xd4c" => "ARM Cortex-X1C".to_string(),
            "0xd4d" => "ARM Cortex-A715".to_string(),
            "0xd4e" => "ARM Cortex-X3".to_string(),
            _ => format!("ARM CPU (part: {})", part),
        };
    }
    
    format!("CPU (implementer: {}, part: {})", implementer, part)
}

/// 判断IP地址范围（公网/内网/回环/链路本地）
fn get_ip_scope(ip: &IpAddr) -> String {
    match ip {
        IpAddr::V4(ipv4) => {
            let octets = ipv4.octets();
            if ipv4.is_loopback() {
                "loopback".to_string()
            } else if ipv4.is_private() 
                || (octets[0] == 10)
                || (octets[0] == 172 && octets[1] >= 16 && octets[1] <= 31)
                || (octets[0] == 192 && octets[1] == 168) {
                "private".to_string()
            } else if ipv4.is_link_local() || (octets[0] == 169 && octets[1] == 254) {
                "link-local".to_string()
            } else {
                "public".to_string()
            }
        }
        IpAddr::V6(ipv6) => {
            if ipv6.is_loopback() {
                "loopback".to_string()
            } else if ipv6.is_unicast_link_local() {
                "link-local".to_string()
            } else if ipv6.segments()[0] & 0xfe00 == 0xfc00 {
                // fc00::/7 - Unique Local Address (ULA)
                "private".to_string()
            } else if ipv6.segments()[0] & 0xff00 == 0xfe00 {
                // fe80::/10 - Link-Local
                "link-local".to_string()
            } else {
                "public".to_string()
            }
        }
    }
}

/// 读取网络接口的IP地址信息
fn read_interface_ip_addresses(interface: &str) -> Result<Vec<IpAddress>, String> {
    use std::process::Command;
    
    let mut addresses = Vec::new();
    
    // 使用 ip addr show 命令获取接口的IP地址
    let output = Command::new("ip")
        .args(&["addr", "show", "dev", interface])
        .output()
        .map_err(|e| format!("Failed to execute ip command: {}", e))?;
    
    if !output.status.success() {
        return Ok(addresses); // 接口可能不存在或无IP，返回空列表
    }
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    
    for line in output_str.lines() {
        let line = line.trim();
        
        // 匹配 "inet 192.168.1.1/24" 或 "inet6 fe80::1/64"
        if line.starts_with("inet ") || line.starts_with("inet6 ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 2 {
                continue;
            }
            
            let ip_type = if parts[0] == "inet" { "ipv4" } else { "ipv6" };
            let addr_with_prefix = parts[1];
            
            // 分离IP地址和前缀长度
            if let Some((addr_str, prefix_str)) = addr_with_prefix.split_once('/') {
                if let Ok(ip) = addr_str.parse::<IpAddr>() {
                    let prefix_len = prefix_str.parse::<u8>().unwrap_or(0);
                    let scope = get_ip_scope(&ip);
                    
                    addresses.push(IpAddress {
                        address: addr_str.to_string(),
                        prefix_len,
                        ip_type: ip_type.to_string(),
                        scope,
                    });
                }
            }
        }
    }
    
    Ok(addresses)
}

/// 读取所有网络接口信息
pub fn read_network_interfaces() -> Result<Vec<NetworkInterfaceInfo>, String> {
    use std::fs;
    use std::path::Path;
    
    let sys_class_net = Path::new("/sys/class/net");
    
    if !sys_class_net.exists() {
        return Err("Network interface directory not found".to_string());
    }
    
    let mut interfaces = Vec::new();
    
    // 遍历所有网络接口
    let entries = fs::read_dir(sys_class_net)
        .map_err(|e| format!("Failed to read network interfaces: {}", e))?;
    
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let interface_name = entry.file_name().to_string_lossy().to_string();
        let interface_path = entry.path();
        
        // 读取接口状态
        let status = fs::read_to_string(interface_path.join("operstate"))
            .unwrap_or_else(|_| "unknown".to_string())
            .trim()
            .to_lowercase();
        
        // 读取MAC地址
        let mac_address = fs::read_to_string(interface_path.join("address"))
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty() && s != "00:00:00:00:00:00");
        
        // 读取MTU
        let mtu = fs::read_to_string(interface_path.join("mtu"))
            .ok()
            .and_then(|s| s.trim().parse::<u32>().ok())
            .unwrap_or(0);
        
        // 读取统计信息
        let stats_path = interface_path.join("statistics");
        let rx_bytes = fs::read_to_string(stats_path.join("rx_bytes"))
            .ok()
            .and_then(|s| s.trim().parse::<u64>().ok())
            .unwrap_or(0);
        let tx_bytes = fs::read_to_string(stats_path.join("tx_bytes"))
            .ok()
            .and_then(|s| s.trim().parse::<u64>().ok())
            .unwrap_or(0);
        let rx_packets = fs::read_to_string(stats_path.join("rx_packets"))
            .ok()
            .and_then(|s| s.trim().parse::<u64>().ok())
            .unwrap_or(0);
        let tx_packets = fs::read_to_string(stats_path.join("tx_packets"))
            .ok()
            .and_then(|s| s.trim().parse::<u64>().ok())
            .unwrap_or(0);
        let rx_errors = fs::read_to_string(stats_path.join("rx_errors"))
            .ok()
            .and_then(|s| s.trim().parse::<u64>().ok())
            .unwrap_or(0);
        let tx_errors = fs::read_to_string(stats_path.join("tx_errors"))
            .ok()
            .and_then(|s| s.trim().parse::<u64>().ok())
            .unwrap_or(0);
        
        // 读取IP地址信息
        let ip_addresses = read_interface_ip_addresses(&interface_name).unwrap_or_default();
        
        interfaces.push(NetworkInterfaceInfo {
            name: interface_name,
            status,
            mac_address,
            mtu,
            ip_addresses,
            rx_bytes,
            tx_bytes,
            rx_packets,
            tx_packets,
            rx_errors,
            tx_errors,
        });
    }
    
    // 按接口名称排序
    interfaces.sort_by(|a, b| a.name.cmp(&b.name));
    
    Ok(interfaces)
}

// ============================================================================
// 频段锁定 (AT+SPLBAND) 相关函数
// ============================================================================

/// 将频段号列表转换为位掩码
///
/// # 展锐 UDX710 频段映射规则
///
/// ## LTE FDD (base=1)
/// 线性映射，使用 16 位位掩码：
/// - B1 = bit0 = 1, B2 = bit1 = 2, B3 = bit2 = 4, ..., B16 = bit15 = 32768
/// - 公式：`1 << (band - 1)`
/// - 设备支持: B1, B3, B5, B8
///
/// ## LTE TDD (base=33)
/// 线性映射，使用 16 位位掩码：
/// - B33 = bit0 = 1, B34 = bit1 = 2, ..., B48 = bit15 = 32768
/// - 公式：`1 << (band - 33)`
/// - 设备支持: B39, B41
///
/// ## NR FDD (base=100, 特殊标记)
/// **非线性映射**，展锐特殊定义：
/// - N1=1, N3=4, N28=512
/// - 设备支持: N1, N3, N28
///
/// ## NR TDD (base=41)
/// **非线性映射**，展锐特殊定义：
/// - N34=1, N38=2, N39=4, N40=8, N41=16, N77=128, N78=256, N79=512
/// - 设备支持: N41, N77, N78, N79
///
/// # Arguments
/// * `bands` - 频段号列表（例如 [1, 3, 8]）
/// * `base` - 基准频段号（1=LTE FDD, 33=LTE TDD, 41=NR TDD, 100=NR FDD）
///
/// # Returns
/// 位掩码值
///
/// # Examples
/// ```
/// // LTE FDD B1+B3+B8
/// let mask = bands_to_bitmask(&[1, 3, 8], 1); // 1 + 4 + 128 = 133
/// // NR FDD N1+N3+N28 (展锐特殊映射)
/// let mask = bands_to_bitmask(&[1, 3, 28], 100); // 1 + 4 + 512 = 517
/// // NR TDD N77+N78 (展锐特殊映射)
/// let mask = bands_to_bitmask(&[77, 78], 41); // 128 + 256 = 384
/// // NR TDD N41+N78+N79
/// let mask = bands_to_bitmask(&[41, 78, 79], 41); // 16 + 256 + 512 = 784
/// ```
pub fn bands_to_bitmask(bands: &[u8], base: u8) -> u16 {
    match base {
        1 => {
            // LTE FDD (B1-B16) - 展锐 UDX710 使用 16 位位掩码
            // 简单线性映射：B1 -> bit0, B2 -> bit1, ..., B16 -> bit15
            bands
                .iter()
                .filter(|&&b| b >= 1 && b <= 16)  // 严格限制在 B1-B16
                .map(|&b| 1u16 << (b - 1))
                .sum()
        }
        33 => {
            // LTE TDD (B33-B48) - 展锐 UDX710 使用 16 位位掩码
            // 简单线性映射：B33 -> bit0, B34 -> bit1, ..., B48 -> bit15
            bands
                .iter()
                .filter(|&&b| b >= 33 && b <= 48)  // 严格限制在 B33-B48
                .map(|&b| 1u16 << (b - 33))
                .sum()
        }
        100 => {
            // NR FDD - 展锐模块使用特殊映射（非线性）
            // 根据 AT+SPLBAND=4 返回的 517 = N1(1) + N3(4) + N28(512)
            let nr_fdd_map: &[(u8, u16)] = &[
                (1, 1),     // N1 -> bit 0
                (2, 2),     // N2 -> bit 1
                (3, 4),     // N3 -> bit 2
                (5, 16),    // N5 -> bit 4
                (7, 64),    // N7 -> bit 6
                (8, 128),   // N8 -> bit 7
                (28, 512),  // N28 -> bit 9
            ];
            
            bands
                .iter()
                .filter_map(|&b| {
                    nr_fdd_map.iter()
                        .find(|(band, _)| *band == b)
                        .map(|(_, mask)| *mask)
                })
                .sum()
        }
        41 => {
            // NR TDD - 展锐模块使用特殊映射（非线性）
            let nr_tdd_map: &[(u8, u16)] = &[
                (34, 1),    // N34 -> bit 0
                (38, 2),    // N38 -> bit 1
                (39, 4),    // N39 -> bit 2
                (40, 8),    // N40 -> bit 3
                (41, 16),   // N41 -> bit 4
                (77, 128),  // N77 -> bit 7
                (78, 256),  // N78 -> bit 8
                (79, 512),  // N79 -> bit 9
            ];
            
            bands
                .iter()
                .filter_map(|&b| {
                    nr_tdd_map.iter()
                        .find(|(band, _)| *band == b)
                        .map(|(_, mask)| *mask)
                })
                .sum()
        }
        _ => 0,
    }
}

/// 将位掩码转换为频段号列表
///
/// # Arguments
/// * `mask` - 位掩码值
/// * `base` - 基准频段号（1=LTE FDD, 33=LTE TDD, 41=NR TDD, 100=NR FDD）
///
/// # Returns
/// 频段号列表
///
/// # Examples
/// ```
/// // 位掩码 133 -> B1+B3+B8
/// let bands = bitmask_to_bands(133, 1); // [1, 3, 8]
/// // NR FDD 位掩码 517 -> N1+N3+N28 (展锐特殊映射)
/// let bands = bitmask_to_bands(517, 100); // [1, 3, 28]
/// // NR TDD 位掩码 384 -> N77+N78 (展锐特殊映射)
/// let bands = bitmask_to_bands(384, 41); // [77, 78]
/// ```
pub fn bitmask_to_bands(mask: u16, base: u8) -> Vec<u8> {
    match base {
        1 => {
            // LTE FDD (B1-B16)
            (0..16)
                .filter(|i| (mask & (1 << i)) != 0)
                .map(|i| (i + 1) as u8)
                .collect()
        }
        33 => {
            // LTE TDD (B33-B48)
            (0..16)
                .filter(|i| (mask & (1 << i)) != 0)
                .map(|i| (33 + i) as u8)
                .collect()
        }
        100 => {
            // NR FDD - 展锐模块使用特殊映射（反向查找）
            let nr_fdd_map: &[(u16, u8)] = &[
                (1, 1),     // bit 0 -> N1
                (2, 2),     // bit 1 -> N2
                (4, 3),     // bit 2 -> N3
                (16, 5),    // bit 4 -> N5
                (64, 7),    // bit 6 -> N7
                (128, 8),   // bit 7 -> N8
                (512, 28),  // bit 9 -> N28
            ];
            
            nr_fdd_map
                .iter()
                .filter(|(bit_mask, _)| (mask & bit_mask) != 0)
                .map(|(_, band)| *band)
                .collect()
        }
        41 => {
            // NR TDD - 展锐模块使用特殊映射（反向查找）
            let nr_tdd_map: &[(u16, u8)] = &[
                (1, 34),    // bit 0 -> N34
                (2, 38),    // bit 1 -> N38
                (4, 39),    // bit 2 -> N39
                (8, 40),    // bit 3 -> N40
                (16, 41),   // bit 4 -> N41
                (128, 77),  // bit 7 -> N77
                (256, 78),  // bit 8 -> N78
                (512, 79),  // bit 9 -> N79
            ];
            
            nr_tdd_map
                .iter()
                .filter(|(bit_mask, _)| (mask & bit_mask) != 0)
                .map(|(_, band)| *band)
                .collect()
        }
        _ => Vec::new(),
    }
}

/// 解析 LTE SPLBAND 响应
///
/// # Arguments
/// * `response` - AT+SPLBAND=0 的响应，格式: `+SPLBAND: 0,<TDD>,0,<FDD>,0`
///
/// # Returns
/// (lte_fdd_mask, lte_tdd_mask)
///
/// # Examples
/// ```
/// let response = "+SPLBAND: 0,320,0,149,0\r\nOK";
/// let (fdd, tdd) = parse_splband_lte_response(response);
/// // fdd = 149 (B1+B3+B5+B8), tdd = 320 (B39+B41)
/// ```
pub fn parse_splband_lte_response(response: &str) -> (u16, u16) {
    for line in response.lines() {
        let line = line.trim();
        if line.starts_with("+SPLBAND:") {
            if let Some(data) = line.strip_prefix("+SPLBAND:") {
                let parts: Vec<&str> = data.trim().split(',').collect();
                if parts.len() >= 5 {
                    // +SPLBAND: 0,<TDD>,0,<FDD>,0
                    // parts[0] = "0", parts[1] = TDD, parts[2] = "0", parts[3] = FDD, parts[4] = "0"
                    let tdd = parts.get(1).and_then(|s| s.trim().parse::<u16>().ok()).unwrap_or(0);
                    let fdd = parts.get(3).and_then(|s| s.trim().parse::<u16>().ok()).unwrap_or(0);
                    return (fdd, tdd);
                }
            }
        }
    }
    (0, 0)
}

/// 解析 NR SPLBAND 响应
///
/// # Arguments
/// * `response` - AT+SPLBAND=3 的响应，格式: `+SPLBAND=<NR-FDD>,0,<NR-TDD>,0`
///
/// # Returns
/// (nr_fdd_mask, nr_tdd_mask)
///
/// # Examples
/// ```
/// let response = "+SPLBAND: 1,0,256,0\r\nOK";
/// let (fdd, tdd) = parse_splband_nr_response(response);
/// // fdd = 1 (N1), tdd = 256 (N78)
/// ```
pub fn parse_splband_nr_response(response: &str) -> (u16, u16) {
    for line in response.lines() {
        let line = line.trim();
        if line.starts_with("+SPLBAND:") {
            if let Some(data) = line.strip_prefix("+SPLBAND:") {
                let parts: Vec<&str> = data.trim().split(',').collect();
                if parts.len() >= 4 {
                    // +SPLBAND: <NR-FDD>,0,<NR-TDD>,0
                    let fdd = parts.get(0).and_then(|s| s.trim().parse::<u16>().ok()).unwrap_or(0);
                    let tdd = parts.get(2).and_then(|s| s.trim().parse::<u16>().ok()).unwrap_or(0);
                    return (fdd, tdd);
                }
            }
        }
    }
    (0, 0)
}

/// 构造 LTE SPLBAND 设置指令
///
/// # Arguments
/// * `fdd_mask` - LTE FDD 频段位掩码
/// * `tdd_mask` - LTE TDD 频段位掩码
///
/// # Returns
/// AT 指令字符串
///
/// # Examples
/// ```
/// // 锁定 LTE B1+B3+B39+B41
/// let cmd = build_splband_lte_command(5, 320); 
/// // "AT+SPLBAND=1,0,320,0,5,0"
/// ```
pub fn build_splband_lte_command(fdd_mask: u16, tdd_mask: u16) -> String {
    // 格式: AT+SPLBAND=1,0,<TDD>,0,<FDD>,0 (6 参数)
    format!("AT+SPLBAND=1,0,{},0,{},0", tdd_mask, fdd_mask)
}

/// 构造 NR SPLBAND 设置指令
///
/// # Arguments
/// * `fdd_mask` - NR FDD 频段位掩码
/// * `tdd_mask` - NR TDD 频段位掩码
///
/// # Returns
/// AT 指令字符串
///
/// # Examples
/// ```
/// // 锁定 NR N1+N78
/// let cmd = build_splband_nr_command(1, 256);
/// // "AT+SPLBAND=2,1,0,256,0"
/// ```
pub fn build_splband_nr_command(fdd_mask: u16, tdd_mask: u16) -> String {
    format!("AT+SPLBAND=2,{},0,{},0", fdd_mask, tdd_mask)
}

