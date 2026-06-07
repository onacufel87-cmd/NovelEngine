//! 应用层：编排 storage / spider / crawler，供 Tauri commands 调用

pub mod discovery;
pub mod notes;
pub mod read;
pub mod settings;
pub mod source;

use crate::utils::AppError;

/// 将 AppError 转为 JSON 字符串（含 code + message）
pub fn to_cmd_err(err: AppError) -> String {
    err.to_cmd_json()
}