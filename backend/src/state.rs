/*
 * @Author: 1orz cloudorzi@gmail.com
 * @Date: 2025-12-10 10:09:22
 * @LastEditors: 1orz cloudorzi@gmail.com
 * @LastEditTime: 2025-12-13 12:46:18
 * @FilePath: /udx710-backend/backend/src/state.rs
 * @Description: 
 * 
 * Copyright (c) 2025 by 1orz, All Rights Reserved. 
 */
//! 应用状态模块
//!
//! 统一管理应用的共享状态

use std::sync::Arc;
use axum::extract::FromRef;
use zbus::Connection;

use crate::config::ConfigManager;
use crate::db::Database;
use crate::webhook::WebhookSender;

/// 应用全局状态
///
/// 统一管理所有共享资源，避免在路由中多次调用 `.with_state()`
#[derive(Clone)]
pub struct AppState {
    /// D-Bus 连接（用于与 ofono 通信）
    pub dbus_conn: Arc<Connection>,
    /// 数据库连接（用于存储 SMS 和通话记录）
    pub database: Arc<Database>,
    /// 配置管理器（用于管理 Webhook 等配置）
    pub config_manager: Arc<ConfigManager>,
    /// Webhook 发送器（用于转发 SMS 和通话通知）
    pub webhook_sender: Arc<WebhookSender>,
}

impl AppState {
    /// 创建新的应用状态
    pub fn new(
        dbus_conn: Arc<Connection>,
        database: Arc<Database>,
        config_manager: Arc<ConfigManager>,
        webhook_sender: Arc<WebhookSender>,
    ) -> Self {
        Self {
            dbus_conn,
            database,
            config_manager,
            webhook_sender,
        }
    }
}

// 实现 FromRef trait，允许从 AppState 中提取子状态
// 这样现有的 handler 可以继续使用 State<Arc<Connection>> 等类型

impl FromRef<AppState> for Arc<Connection> {
    fn from_ref(state: &AppState) -> Self {
        state.dbus_conn.clone()
    }
}

impl FromRef<AppState> for Arc<Database> {
    fn from_ref(state: &AppState) -> Self {
        state.database.clone()
    }
}

impl FromRef<AppState> for Arc<ConfigManager> {
    fn from_ref(state: &AppState) -> Self {
        state.config_manager.clone()
    }
}

impl FromRef<AppState> for Arc<WebhookSender> {
    fn from_ref(state: &AppState) -> Self {
        state.webhook_sender.clone()
    }
}

// 支持 (Arc<Connection>, Arc<Database>) 元组类型
impl FromRef<AppState> for (Arc<Connection>, Arc<Database>) {
    fn from_ref(state: &AppState) -> Self {
        (state.dbus_conn.clone(), state.database.clone())
    }
}
