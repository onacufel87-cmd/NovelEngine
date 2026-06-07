//! 统一错误类型，所有模块的 Result 均使用 AppError

use std::fmt;

use serde::Serialize;

/// 应用级错误枚举
#[derive(Debug, Clone)]
pub enum AppError {
    Network(String),
    Parse(String),
    Database(String),
    NotFound(String),
    InvalidRule(String),
}

/// 结构化错误（供 Tauri 命令返回 JSON，前端按 code 展示友好提示）
#[derive(Debug, Clone, Serialize)]
pub struct AppErrorPayload {
    pub code: &'static str,
    pub message: String,
}

impl AppError {
    /// 机器可读错误码
    pub fn code(&self) -> &'static str {
        match self {
            AppError::Network(_) => "network",
            AppError::Parse(_) => "parse",
            AppError::Database(_) => "database",
            AppError::NotFound(_) => "not_found",
            AppError::InvalidRule(_) => "invalid_rule",
        }
    }

    /// 用户可读消息（不含类型前缀）
    pub fn user_message(&self) -> String {
        match self {
            AppError::Network(msg) => msg.clone(),
            AppError::Parse(msg) => msg.clone(),
            AppError::Database(msg) => msg.clone(),
            AppError::NotFound(msg) => msg.clone(),
            AppError::InvalidRule(msg) => msg.clone(),
        }
    }

    pub fn to_payload(&self) -> AppErrorPayload {
        AppErrorPayload {
            code: self.code(),
            message: self.user_message(),
        }
    }

    /// 序列化为 JSON 字符串，供 invoke 错误回传
    pub fn to_cmd_json(&self) -> String {
        serde_json::to_string(&self.to_payload()).unwrap_or_else(|_| self.to_string())
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Network(msg) => write!(f, "网络错误: {msg}"),
            AppError::Parse(msg) => write!(f, "解析错误: {msg}"),
            AppError::Database(msg) => write!(f, "数据库错误: {msg}"),
            AppError::NotFound(msg) => write!(f, "未找到: {msg}"),
            AppError::InvalidRule(msg) => write!(f, "书源规则无效: {msg}"),
        }
    }
}

impl std::error::Error for AppError {}

impl From<AppError> for String {
    fn from(err: AppError) -> String {
        err.to_cmd_json()
    }
}

pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_payload_serializes_code() {
        let err = AppError::Network("连接超时".into());
        let json = err.to_cmd_json();
        assert!(json.contains("\"code\":\"network\""));
        assert!(json.contains("连接超时"));
    }
}
