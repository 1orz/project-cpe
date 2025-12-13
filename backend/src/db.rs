/*
 * @Author: 1orz cloudorzi@gmail.com
 * @Date: 2025-12-08 13:03:48
 * @LastEditors: 1orz cloudorzi@gmail.com
 * @LastEditTime: 2025-12-13 12:46:00
 * @FilePath: /udx710-backend/backend/src/db.rs
 * @Description: 
 * 
 * Copyright (c) 2025 by 1orz, All Rights Reserved. 
 */
//! 数据库模块
//!
//! 使用 SQLite 存储短信历史记录和通话记录

use chrono::Utc;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// 短信记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmsMessage {
    pub id: i64,
    pub direction: String,      // "incoming" 或 "outgoing"
    pub phone_number: String,   // 发件人或收件人
    pub content: String,        // 短信内容
    pub timestamp: String,      // ISO 8601 格式时间
    pub status: String,         // "pending", "sent", "failed", "received"
    pub pdu: Option<String>,    // 原始 PDU（如果有）
}

/// 通话记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallRecord {
    pub id: i64,
    pub direction: String,      // "incoming" / "outgoing" / "missed"
    pub phone_number: String,   // 电话号码
    pub duration: i64,          // 通话时长（秒）
    pub start_time: String,     // 开始时间 ISO 8601
    pub end_time: Option<String>, // 结束时间 ISO 8601
    pub answered: bool,         // 是否接通
}

/// 短信统计
#[derive(Debug, Serialize, Deserialize)]
pub struct SmsStats {
    pub total: i64,
    pub incoming: i64,
    pub outgoing: i64,
}

/// 通话统计
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CallStats {
    pub total: i64,
    pub incoming: i64,
    pub outgoing: i64,
    pub missed: i64,
    pub total_duration: i64,  // 总通话时长（秒）
}

/// 数据库管理器
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    /// 创建或打开数据库
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        
        // 创建短信表（如果不存在）
        conn.execute(
            "CREATE TABLE IF NOT EXISTS sms_messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                direction TEXT NOT NULL,
                phone_number TEXT NOT NULL,
                content TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                status TEXT NOT NULL,
                pdu TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        
        // 创建短信索引
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_sms_timestamp ON sms_messages(timestamp DESC)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_sms_phone ON sms_messages(phone_number)",
            [],
        )?;
        
        // 创建通话记录表（如果不存在）
        conn.execute(
            "CREATE TABLE IF NOT EXISTS call_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                direction TEXT NOT NULL,
                phone_number TEXT NOT NULL,
                duration INTEGER DEFAULT 0,
                start_time TEXT NOT NULL,
                end_time TEXT,
                answered INTEGER DEFAULT 0,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        
        // 创建通话记录索引
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_call_start_time ON call_history(start_time DESC)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_call_phone ON call_history(phone_number)",
            [],
        )?;
        
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }
    
    // ==================== 短信相关方法 ====================
    
    /// 插入新短信
    pub fn insert_sms(
        &self,
        direction: &str,
        phone_number: &str,
        content: &str,
        status: &str,
        pdu: Option<&str>,
    ) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let timestamp = Utc::now().to_rfc3339();
        
        conn.execute(
            "INSERT INTO sms_messages (direction, phone_number, content, timestamp, status, pdu)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![direction, phone_number, content, timestamp, status, pdu],
        )?;
        
        Ok(conn.last_insert_rowid())
    }
    
    /// 更新短信状态
    #[allow(dead_code)]
    pub fn update_sms_status(&self, id: i64, status: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE sms_messages SET status = ?1 WHERE id = ?2",
            params![status, id],
        )?;
        Ok(())
    }
    
    /// 获取所有短信（分页）
    pub fn get_sms_messages(&self, limit: i64, offset: i64) -> Result<Vec<SmsMessage>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, direction, phone_number, content, timestamp, status, pdu
             FROM sms_messages
             ORDER BY timestamp DESC
             LIMIT ?1 OFFSET ?2"
        )?;
        
        let messages = stmt.query_map(params![limit, offset], |row| {
            Ok(SmsMessage {
                id: row.get(0)?,
                direction: row.get(1)?,
                phone_number: row.get(2)?,
                content: row.get(3)?,
                timestamp: row.get(4)?,
                status: row.get(5)?,
                pdu: row.get(6)?,
            })
        })?;
        
        let mut result = Vec::new();
        for message in messages {
            result.push(message?);
        }
        
        Ok(result)
    }
    
    /// 获取与特定号码的对话历史
    pub fn get_sms_conversation(&self, phone_number: &str, limit: i64) -> Result<Vec<SmsMessage>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, direction, phone_number, content, timestamp, status, pdu
             FROM sms_messages
             WHERE phone_number = ?1
             ORDER BY timestamp DESC
             LIMIT ?2"
        )?;
        
        let messages = stmt.query_map(params![phone_number, limit], |row| {
            Ok(SmsMessage {
                id: row.get(0)?,
                direction: row.get(1)?,
                phone_number: row.get(2)?,
                content: row.get(3)?,
                timestamp: row.get(4)?,
                status: row.get(5)?,
                pdu: row.get(6)?,
            })
        })?;
        
        let mut result = Vec::new();
        for message in messages {
            result.push(message?);
        }
        
        Ok(result)
    }
    
    /// 获取短信统计
    pub fn get_sms_stats(&self) -> Result<SmsStats> {
        let conn = self.conn.lock().unwrap();
        
        let total: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sms_messages",
            [],
            |row| row.get(0),
        )?;
        
        let incoming: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sms_messages WHERE direction = 'incoming'",
            [],
            |row| row.get(0),
        )?;
        
        let outgoing: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sms_messages WHERE direction = 'outgoing'",
            [],
            |row| row.get(0),
        )?;
        
        Ok(SmsStats {
            total,
            incoming,
            outgoing,
        })
    }
    
    /// 删除旧短信（保留最近 N 条）
    #[allow(dead_code)]
    pub fn cleanup_old_sms(&self, keep_count: i64) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let deleted = conn.execute(
            "DELETE FROM sms_messages WHERE id NOT IN (
                SELECT id FROM sms_messages ORDER BY timestamp DESC LIMIT ?1
            )",
            params![keep_count],
        )?;
        Ok(deleted)
    }
    
    /// 删除所有短信
    pub fn clear_all_sms(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM sms_messages", [])?;
        Ok(())
    }
    
    // ==================== 通话记录相关方法 ====================
    
    /// 插入新通话记录
    pub fn insert_call(
        &self,
        direction: &str,
        phone_number: &str,
        answered: bool,
    ) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let start_time = Utc::now().to_rfc3339();
        
        conn.execute(
            "INSERT INTO call_history (direction, phone_number, duration, start_time, answered)
             VALUES (?1, ?2, 0, ?3, ?4)",
            params![direction, phone_number, start_time, answered as i32],
        )?;
        
        Ok(conn.last_insert_rowid())
    }
    
    /// 更新通话记录（通话结束时调用）
    pub fn update_call_end(&self, id: i64, duration: i64, answered: bool) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let end_time = Utc::now().to_rfc3339();
        
        conn.execute(
            "UPDATE call_history SET duration = ?1, end_time = ?2, answered = ?3 WHERE id = ?4",
            params![duration, end_time, answered as i32, id],
        )?;
        Ok(())
    }
    
    /// 标记通话为未接来电
    pub fn mark_call_missed(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let end_time = Utc::now().to_rfc3339();
        
        conn.execute(
            "UPDATE call_history SET direction = 'missed', end_time = ?1, answered = 0 WHERE id = ?2",
            params![end_time, id],
        )?;
        Ok(())
    }
    
    /// 获取通话记录（分页）
    pub fn get_call_history(&self, limit: i64, offset: i64) -> Result<Vec<CallRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, direction, phone_number, duration, start_time, end_time, answered
             FROM call_history
             ORDER BY start_time DESC
             LIMIT ?1 OFFSET ?2"
        )?;
        
        let records = stmt.query_map(params![limit, offset], |row| {
            Ok(CallRecord {
                id: row.get(0)?,
                direction: row.get(1)?,
                phone_number: row.get(2)?,
                duration: row.get(3)?,
                start_time: row.get(4)?,
                end_time: row.get(5)?,
                answered: row.get::<_, i32>(6)? != 0,
            })
        })?;
        
        let mut result = Vec::new();
        for record in records {
            result.push(record?);
        }
        
        Ok(result)
    }
    
    /// 获取与特定号码的通话记录
    #[allow(dead_code)]
    pub fn get_call_history_by_number(&self, phone_number: &str, limit: i64) -> Result<Vec<CallRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, direction, phone_number, duration, start_time, end_time, answered
             FROM call_history
             WHERE phone_number = ?1
             ORDER BY start_time DESC
             LIMIT ?2"
        )?;
        
        let records = stmt.query_map(params![phone_number, limit], |row| {
            Ok(CallRecord {
                id: row.get(0)?,
                direction: row.get(1)?,
                phone_number: row.get(2)?,
                duration: row.get(3)?,
                start_time: row.get(4)?,
                end_time: row.get(5)?,
                answered: row.get::<_, i32>(6)? != 0,
            })
        })?;
        
        let mut result = Vec::new();
        for record in records {
            result.push(record?);
        }
        
        Ok(result)
    }
    
    /// 获取通话统计
    pub fn get_call_stats(&self) -> Result<CallStats> {
        let conn = self.conn.lock().unwrap();
        
        let total: i64 = conn.query_row(
            "SELECT COUNT(*) FROM call_history",
            [],
            |row| row.get(0),
        )?;
        
        let incoming: i64 = conn.query_row(
            "SELECT COUNT(*) FROM call_history WHERE direction = 'incoming'",
            [],
            |row| row.get(0),
        )?;
        
        let outgoing: i64 = conn.query_row(
            "SELECT COUNT(*) FROM call_history WHERE direction = 'outgoing'",
            [],
            |row| row.get(0),
        )?;
        
        let missed: i64 = conn.query_row(
            "SELECT COUNT(*) FROM call_history WHERE direction = 'missed'",
            [],
            |row| row.get(0),
        )?;
        
        let total_duration: i64 = conn.query_row(
            "SELECT COALESCE(SUM(duration), 0) FROM call_history WHERE answered = 1",
            [],
            |row| row.get(0),
        )?;
        
        Ok(CallStats {
            total,
            incoming,
            outgoing,
            missed,
            total_duration,
        })
    }
    
    /// 删除单条通话记录
    pub fn delete_call(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM call_history WHERE id = ?1", params![id])?;
        Ok(())
    }
    
    /// 删除所有通话记录
    pub fn clear_all_calls(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM call_history", [])?;
        Ok(())
    }
}


