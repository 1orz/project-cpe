/*
 * @Author: 1orz cloudorzi@gmail.com
 * @Date: 2025-12-11 17:44:29
 * @LastEditors: 1orz cloudorzi@gmail.com
 * @LastEditTime: 2025-12-13 12:46:08
 * @FilePath: /udx710-backend/backend/src/main.rs
 * @Description: 
 * 
 * Copyright (c) 2025 by 1orz, All Rights Reserved. 
 */
//! Project CPE - UDX710 5G/LTE Module Management Service
//!
//! A backend service for managing UDX710 5G/LTE modules via ofono D-Bus interface.
//! Built with Rust + Axum + zbus.
//!
//! Copyright (c) 2025 1orz
//! GitHub: https://github.com/1orz
//! Project: https://github.com/1orz/project-cpe
//!
//! Licensed under the MIT License.

use anyhow::Result;
use axum::{
    routing::get, 
    routing::post, 
    Router,
    response::{IntoResponse, Response},
    http::{StatusCode, Uri},
    extract::DefaultBodyLimit,
};
use clap::Parser;
use std::sync::Arc;
use std::path::PathBuf;
use tower_http::cors::{CorsLayer, Any};
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use zbus::Connection;

mod config;
mod db;
mod dbus;
mod handlers;
mod iptables;
mod models;
mod ota;
mod serial;
mod sms_listener;
mod state;
mod usb_switch;
mod utils;
mod webhook;

use config::{ConfigManager, get_default_config_path};
use dbus::init_data_connection;
use handlers::*;
use db::Database;
use state::AppState;
use webhook::WebhookSender;

/// 获取二进制文件同级目录下的 www 目录路径
fn get_www_dir() -> PathBuf {
    // 获取当前可执行文件的路径
    let exe_path = std::env::current_exe()
        .expect("Failed to get executable path");
    
    // 获取可执行文件所在目录
    let exe_dir = exe_path.parent()
        .expect("Failed to get executable directory");
    
    // 拼接 www 目录
    exe_dir.join("www")
}

/// SPA fallback handler - 对于所有前端路由返回 index.html
async fn spa_fallback(uri: Uri) -> Response {
    let path = uri.path();
    
    // 如果是 API 路由，返回 404（不应该走到这里，但作为保险）
    if path.starts_with("/api/") {
        return (StatusCode::NOT_FOUND, "API endpoint not found").into_response();
    }
    
    // 获取 www 目录的绝对路径
    let www_dir = get_www_dir();
    
    // 构建请求文件的完整路径
    let requested_path = if path == "/" { "/index.html" } else { path };
    let file_path = www_dir.join(requested_path.trim_start_matches('/'));
    
    // 如果文件存在，返回文件内容
    if let Ok(content) = tokio::fs::read(&file_path).await {
        // 根据文件扩展名设置正确的 Content-Type
        let content_type = match file_path
            .extension()
            .and_then(|ext| ext.to_str())
        {
            Some("html") => "text/html; charset=utf-8",
            Some("css") => "text/css; charset=utf-8",
            Some("js") => "application/javascript; charset=utf-8",
            Some("json") => "application/json",
            Some("png") => "image/png",
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("gif") => "image/gif",
            Some("svg") => "image/svg+xml",
            Some("ico") => "image/x-icon",
            _ => "application/octet-stream",
        };
        
        return (
            StatusCode::OK,
            [(axum::http::header::CONTENT_TYPE, content_type)],
            content
        ).into_response();
    }
    
    // 如果文件不存在，返回 index.html（SPA 路由）
    let index_path = www_dir.join("index.html");
    match tokio::fs::read(&index_path).await {
        Ok(content) => (
            StatusCode::OK,
            [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
            content
        ).into_response(),
        Err(_) => (
            StatusCode::NOT_FOUND,
            format!("index.html not found at {:?}. Please build the frontend first.", index_path)
        ).into_response(),
    }
}

/// R106 Backend Service - UDX710 5G/LTE 模块管理服务
#[derive(Parser, Debug)]
#[command(name = "udx710")]
#[command(author, version, about, long_about = None)]
struct Args {
    /// 监听端口 (默认: 3000)
    #[arg(short, long, default_value = "3000", env = "PORT")]
    port: u16,

    /// 监听地址 (默认: 0.0.0.0)
    #[arg(short = 'H', long, default_value = "0.0.0.0", env = "HOST")]
    host: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化 tracing 日志框架
    // 通过 RUST_LOG 环境变量控制日志级别，默认为 info
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();

    // 解析命令行参数
    let args = Args::parse();
    let bind_addr = format!("{}:{}", args.host, args.port);

    // Connect to system D-Bus
    let dbus_conn = Arc::new(Connection::system().await?);
    
    // 创建 SMS 数据库（存储在可执行文件同级目录）
    let exe_dir = std::env::current_exe()
        .expect("Failed to get executable path")
        .parent()
        .expect("Failed to get executable directory")
        .to_path_buf();
    let db_path = exe_dir.join("data.db");
    let app_db = Arc::new(Database::new(db_path)?);
    
    // 初始化配置管理器
    let config_path = get_default_config_path();
    info!(path = ?config_path, "Loading config");
    let config_manager = Arc::new(ConfigManager::new(config_path));
    
    // 初始化 Webhook 发送器
    let webhook_sender = Arc::new(WebhookSender::new(Arc::clone(&config_manager)));
    
    // 启动 SMS 监听线程
    {
        let conn_clone = Connection::system().await?;
        let db_clone = Arc::clone(&app_db);
        let webhook_clone = Arc::clone(&webhook_sender);
        tokio::spawn(async move {
            let _ = sms_listener::start_sms_listener(conn_clone, db_clone, webhook_clone).await;
        });
    }
    
    // 启动电话监听线程（包括通话记录存储）
    {
        let conn_clone = Connection::system().await?;
        let db_clone = Arc::clone(&app_db);
        let webhook_clone = Arc::clone(&webhook_sender);
        tokio::spawn(async move {
            let _ = sms_listener::start_call_listener(conn_clone, db_clone, webhook_clone).await;
        });
    }
    
    // 自动初始化数据连接
    {
        let conn_clone = Arc::clone(&dbus_conn);
        tokio::spawn(async move {
            // 等待 2 秒让 modem 完全初始化
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            let result = init_data_connection(&conn_clone).await;
            tracing::info!(result = %result, "Auto-connect completed");
        });
    }
    
    // 启动数据连接 Watchdog（每 15 秒检查一次）
    {
        let conn_clone = Arc::clone(&dbus_conn);
        tokio::spawn(async move {
            // 初始延迟 5 秒，等待系统稳定
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            tracing::info!(interval = 15, "Watchdog started");
            dbus::data_connection_watchdog(conn_clone, 5).await;
        });
    }

    // CORS 配置：允许前端开发服务器跨域访问
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // 创建统一的应用状态
    let app_state = AppState::new(
        dbus_conn,
        app_db,
        config_manager,
        webhook_sender,
    );

    // Build routes - 使用统一的 AppState
    let app = Router::new()
        // ========== AT 指令接口 ==========
        .route("/api/at", post(post_at_command).options(options_handler))
        // ========== 设备信息接口 ==========
        .route("/api/device", get(get_device_info).options(options_handler))
        .route("/api/device/imeisv", get(get_imeisv_handler).options(options_handler))
        // ========== SIM 卡接口 ==========
        .route("/api/sim", get(get_sim_info).options(options_handler))
        .route("/api/sim/slot", get(get_sim_slot_handler).options(options_handler))
        .route("/api/sim/slot/switch", post(switch_sim_slot_handler).options(options_handler))
        // ========== 网络接口 ==========
        .route("/api/network", get(get_network_info).options(options_handler))
        .route("/api/network/interfaces", get(get_network_interfaces_info).options(options_handler))
        .route("/api/network/signal-strength", get(get_signal_strength_handler).options(options_handler))
        .route("/api/network/nitz", get(get_nitz_handler).options(options_handler))
        .route("/api/network/operators", get(get_operators_handler).options(options_handler))
        .route("/api/network/operators/scan", get(scan_operators_handler).options(options_handler))
        .route("/api/network/register-manual", post(register_operator_manual_handler).options(options_handler))
        .route("/api/network/register-auto", post(register_operator_auto_handler).options(options_handler))
        // ========== 小区信息接口 ==========
        .route("/api/cells", get(get_cells).options(options_handler))
        .route("/api/location/cell-info", get(get_cell_location_info).options(options_handler))
        // ========== QoS 接口 ==========
        .route("/api/qos", get(get_qos_info).options(options_handler))
        // ========== 数据连接接口 ==========
        .route("/api/data", get(get_data_status).post(set_data_status).options(options_handler))
        .route("/api/roaming", get(get_roaming_status_handler).post(set_roaming_status_handler).options(options_handler))
        .route("/api/airplane-mode", get(get_airplane_mode_handler).post(set_airplane_mode_handler).options(options_handler))
        // ========== 射频模式接口 ==========
        .route("/api/radio-mode", get(get_radio_mode_handler).post(set_radio_mode_handler).options(options_handler))
        .route("/api/band-lock", get(get_band_lock_handler).post(set_band_lock_handler).options(options_handler))
        .route("/api/cell-lock", get(get_cell_lock_handler).post(set_cell_lock_handler).options(options_handler))
        .route("/api/cell-lock/unlock-all", post(unlock_all_cells_handler).options(options_handler))
        // ========== APN 管理接口 ==========
        .route("/api/apn", get(get_apn_list_handler).post(set_apn_handler).options(options_handler))
        // ========== 电话功能接口 ==========
        .route("/api/calls", get(get_calls_handler).options(options_handler))
        .route("/api/call/dial", post(dial_call_handler).options(options_handler))
        .route("/api/call/hangup", post(hangup_call_handler).options(options_handler))
        .route("/api/call/hangup-all", post(hangup_all_calls_handler).options(options_handler))
        .route("/api/call/answer", post(answer_call_handler).options(options_handler))
        .route("/api/call/volume", get(get_call_volume_handler).post(set_call_volume_handler).options(options_handler))
        .route("/api/call/forwarding", get(get_call_forwarding_handler).post(set_call_forwarding_handler).options(options_handler))
        .route("/api/call/settings", get(get_call_settings_handler).post(set_call_settings_handler).options(options_handler))
        .route("/api/call/history", get(get_call_history_handler).options(options_handler))
        .route("/api/call/history/{id}", axum::routing::delete(delete_call_history_handler).options(options_handler))
        .route("/api/call/history/clear", post(clear_call_history_handler).options(options_handler))
        // ========== 短信功能接口 ==========
        .route("/api/sms/send", post(send_sms_handler).options(options_handler))
        .route("/api/sms/list", get(get_sms_list_handler).options(options_handler))
        .route("/api/sms/conversation", get(get_sms_conversation_handler).options(options_handler))
        .route("/api/sms/stats", get(get_sms_stats_handler).options(options_handler))
        .route("/api/sms/clear", post(clear_sms_handler).options(options_handler))
        // ========== IMS/VoLTE 接口 ==========
        .route("/api/ims/status", get(get_ims_status_handler).options(options_handler))
        .route("/api/voicemail/status", get(get_voicemail_status_handler).options(options_handler))
        // ========== USB 模式接口 ==========
        .route("/api/usb-mode", get(get_usb_mode).post(set_usb_mode).options(options_handler))
        .route("/api/usb-advance", post(set_usb_mode_advanced).options(options_handler))
        // ========== 系统接口 ==========
        .route("/api/stats", get(get_system_stats).options(options_handler))
        .route("/api/stats/cpu", get(get_cpu_info).options(options_handler))
        .route("/api/connectivity", get(get_connectivity_check).options(options_handler))
        .route("/api/system/reboot", post(system_reboot).options(options_handler))
        .route("/api/health", get(health_check))
        // ========== Webhook 配置接口 ==========
        .route("/api/webhook/config", get(get_webhook_config_handler).post(set_webhook_config_handler).options(options_handler))
        .route("/api/webhook/test", post(test_webhook_handler).options(options_handler))
        // ========== OTA 更新接口 ==========
        .route("/api/ota/status", get(get_ota_status_handler).options(options_handler))
        .route("/api/ota/upload", post(upload_ota_handler).options(options_handler)
            .layer(DefaultBodyLimit::max(50 * 1024 * 1024))) // 50MB 限制
        .route("/api/ota/apply", post(apply_ota_handler).options(options_handler))
        .route("/api/ota/cancel", post(cancel_ota_handler).options(options_handler))
        // ========== 统一状态和中间件 ==========
        .with_state(app_state)
        .layer(cors)
        .fallback(spa_fallback);

    // Start server - 显示版权信息
    info!(
        version = env!("APP_VERSION"),
        branch = env!("GIT_BRANCH"),
        commit = env!("GIT_COMMIT"),
        "Project CPE - UDX710 5G/LTE Module Management"
    );
    info!("Copyright (c) 2025 1orz - https://github.com/1orz/project-cpe");

    // 绑定端口，如果被占用则轮询等待（最多 30 秒）
    let listener = bind_with_retry(&bind_addr, 30).await?;
    info!(addr = %bind_addr, "Server listening");
    // 使用优雅关闭
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

/// 绑定端口，如果被占用则轮询等待
async fn bind_with_retry(addr: &str, max_retries: u32) -> Result<tokio::net::TcpListener> {
    use std::time::Duration;
    
    for i in 0..max_retries {
        match tokio::net::TcpListener::bind(addr).await {
            Ok(listener) => return Ok(listener),
            Err(e) => {
                if i == 0 {
                    warn!(addr = %addr, "Port busy, waiting for release...");
                }
                if i + 1 < max_retries {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                } else {
                    return Err(anyhow::anyhow!("Failed to bind to {}: {}", addr, e));
                }
            }
        }
    }
    unreachable!()
}

/// 监听 Ctrl+C 和 SIGTERM 信号，用于优雅关闭
async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C signal handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
