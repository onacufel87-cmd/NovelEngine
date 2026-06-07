//! 书源自动检测结构化日志（供前端 Terminal 展示）

use serde::{Deserialize, Serialize};

/// 单条检测日志
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectLogEntry {
    /// INFO | WARN | SUCCESS | ERROR
    pub level: String,
    pub message: String,
    /// Unix 秒时间戳
    pub ts: i64,
}

/// 检测 API 统一响应：成功/失败均携带日志
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectResponse<T> {
    pub result: Option<T>,
    pub logs: Vec<DetectLogEntry>,
    pub error: Option<String>,
}

impl<T> DetectResponse<T> {
    pub fn ok(result: T, logger: DetectLogger) -> Self {
        Self {
            result: Some(result),
            logs: logger.entries,
            error: None,
        }
    }

    pub fn fail(error: String, logger: DetectLogger) -> Self {
        Self {
            result: None,
            logs: logger.entries,
            error: Some(error),
        }
    }
}

/// 检测过程日志收集器
#[derive(Debug, Default)]
pub struct DetectLogger {
    pub entries: Vec<DetectLogEntry>,
}

impl DetectLogger {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn info(&mut self, message: impl Into<String>) {
        self.push("INFO", message);
    }

    pub fn warn(&mut self, message: impl Into<String>) {
        self.push("WARN", message);
    }

    pub fn success(&mut self, message: impl Into<String>) {
        self.push("SUCCESS", message);
    }

    pub fn error(&mut self, message: impl Into<String>) {
        self.push("ERROR", message);
    }

    fn push(&mut self, level: &str, message: impl Into<String>) {
        self.entries.push(DetectLogEntry {
            level: level.to_string(),
            message: message.into(),
            ts: unix_now(),
        });
    }
}

fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
