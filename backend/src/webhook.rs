/*
 * @Author: 1orz cloudorzi@gmail.com
 * @Date: 2025-12-09 17:34:01
 * @LastEditors: 1orz cloudorzi@gmail.com
 * @LastEditTime: 2025-12-13 12:46:25
 * @FilePath: /udx710-backend/backend/src/webhook.rs
 * @Description: 
 * 
 * Copyright (c) 2025 by 1orz, All Rights Reserved. 
 */
//! Webhook 转发模块
//!
//! 用于将来电和短信转发到外部 Webhook
//! 支持自定义 payload 模板，使用 {{变量名}} 格式替换

use crate::config::{ConfigManager, WebhookConfig};
use crate::db::{CallRecord, SmsMessage};
use chrono::Utc;
use reqwest::Client;
use std::sync::Arc;

/// Webhook 发送器
pub struct WebhookSender {
    client: Client,
    config_manager: Arc<ConfigManager>,
}

impl WebhookSender {
    /// 创建新的 Webhook 发送器
    pub fn new(config_manager: Arc<ConfigManager>) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("Failed to create HTTP client"),
            config_manager,
        }
    }
    
    /// 获取当前 Webhook 配置
    fn get_config(&self) -> WebhookConfig {
        self.config_manager.get_webhook()
    }
    
    /// 转发短信
    pub async fn forward_sms(&self, message: &SmsMessage) -> Result<(), String> {
        let config = self.get_config();
        
        if !config.enabled || !config.forward_sms || config.url.is_empty() {
            return Ok(());
        }
        
        // 使用模板替换变量
        let payload = render_sms_template(&config.sms_template, message);
        
        self.send_webhook_raw(&config, &payload).await
    }
    
    /// 转发通话记录
    pub async fn forward_call(&self, call: &CallRecord) -> Result<(), String> {
        let config = self.get_config();
        
        if !config.enabled || !config.forward_calls || config.url.is_empty() {
            return Ok(());
        }
        
        // 使用模板替换变量
        let payload = render_call_template(&config.call_template, call);
        
        self.send_webhook_raw(&config, &payload).await
    }
    
    /// 发送原始 JSON 字符串的 Webhook 请求
    async fn send_webhook_raw(&self, config: &WebhookConfig, payload: &str) -> Result<(), String> {
        let mut request = self.client.post(&config.url);
        
        // 添加自定义请求头
        for (key, value) in &config.headers {
            request = request.header(key, value);
        }
        
        // 添加 Content-Type
        request = request.header("Content-Type", "application/json");
        
        // 如果有密钥，添加签名头
        if !config.secret.is_empty() {
            let signature = compute_hmac(&config.secret, payload);
            request = request.header("X-Webhook-Signature", signature);
        }
        
        // 发送请求
        let response = request
            .body(payload.to_string())
            .send()
            .await
            .map_err(|e| format!("Failed to send webhook: {}", e))?;
        
        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(format!("Webhook returned error status {}: {}", status, body))
        }
    }
    
    /// 测试 Webhook 连接（使用短信模板发送测试数据）
    pub async fn test_webhook(&self) -> Result<String, String> {
        let config = self.get_config();
        
        if config.url.is_empty() {
            return Err("Webhook URL is not configured".to_string());
        }
        
        // 使用模拟数据渲染短信模板进行测试
        let test_message = SmsMessage {
            id: 0,
            direction: "incoming".to_string(),
            phone_number: "+8613800138000".to_string(),
            content: "这是一条测试短信 (Webhook Test)".to_string(),
            timestamp: Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            status: "received".to_string(),
            pdu: None,
        };
        
        let payload = render_sms_template(&config.sms_template, &test_message);
        
        let mut request = self.client.post(&config.url);
        
        for (key, value) in &config.headers {
            request = request.header(key, value);
        }
        
        request = request.header("Content-Type", "application/json");
        
        if !config.secret.is_empty() {
            let signature = compute_hmac(&config.secret, &payload);
            request = request.header("X-Webhook-Signature", signature);
        }
        
        let response = request
            .body(payload)
            .send()
            .await
            .map_err(|e| format!("Failed to send test webhook: {}", e))?;
        
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        
        if status.is_success() {
            Ok(format!("Webhook test successful (status: {})", status))
        } else {
            Err(format!("Webhook test failed (status: {}): {}", status, body))
        }
    }
}

/// 渲染短信模板，替换变量
/// 支持的变量：{{id}}, {{phone_number}}, {{content}}, {{direction}}, {{timestamp}}, {{status}}
fn render_sms_template(template: &str, message: &SmsMessage) -> String {
    template
        .replace("{{id}}", &message.id.to_string())
        .replace("{{phone_number}}", &message.phone_number)
        .replace("{{content}}", &escape_json_string(&message.content))
        .replace("{{direction}}", &message.direction)
        .replace("{{timestamp}}", &message.timestamp)
        .replace("{{status}}", &message.status)
        // 别名支持
        .replace("{{sender}}", &message.phone_number)
        .replace("{{message}}", &escape_json_string(&message.content))
        .replace("{{time}}", &message.timestamp)
}

/// 渲染通话模板，替换变量
/// 支持的变量：{{id}}, {{phone_number}}, {{direction}}, {{duration}}, {{start_time}}, {{end_time}}, {{answered}}
fn render_call_template(template: &str, call: &CallRecord) -> String {
    let end_time = call.end_time.clone().unwrap_or_default();
    let answered_str = if call.answered { "是" } else { "否" };
    let direction_cn = if call.direction == "incoming" { "来电" } else { "去电" };
    
    template
        .replace("{{id}}", &call.id.to_string())
        .replace("{{phone_number}}", &call.phone_number)
        .replace("{{direction}}", &call.direction)
        .replace("{{direction_cn}}", direction_cn)
        .replace("{{duration}}", &call.duration.to_string())
        .replace("{{start_time}}", &call.start_time)
        .replace("{{end_time}}", &end_time)
        .replace("{{answered}}", answered_str)
        .replace("{{answered_bool}}", &call.answered.to_string())
        // 别名支持
        .replace("{{caller}}", &call.phone_number)
        .replace("{{time}}", &call.start_time)
}

/// 转义 JSON 字符串中的特殊字符
fn escape_json_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// 计算简单签名
fn compute_hmac(secret: &str, data: &str) -> String {
    use std::hash::{Hash, Hasher};
    use std::collections::hash_map::DefaultHasher;
    
    let mut hasher = DefaultHasher::new();
    format!("{}{}", secret, data).hash(&mut hasher);
    let hash = hasher.finish();
    
    format!("{:016x}", hash)
}

