//! 阅读器与全局设置

use crate::spider;
use crate::storage::{get_setting, set_setting};
use crate::utils::AppResult;

/// 读取设置 JSON
pub fn get_reader_settings_json() -> AppResult<String> {
    match get_setting("reader_settings")? {
        Some(json) => Ok(json),
        None => Ok("{}".to_string()),
    }
}

/// 保存设置并同步网络抓取配置
pub fn save_reader_settings_json(settings_json: &str) -> AppResult<()> {
    spider::apply_reader_settings_json(settings_json);
    set_setting("reader_settings", settings_json)
}
