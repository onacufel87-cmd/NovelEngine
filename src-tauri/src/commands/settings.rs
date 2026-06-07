//! 阅读器与全局设置持久化

use tauri::Manager;

use crate::services::{settings, to_cmd_err};
use crate::storage::{clear_custom_root, path_info, set_custom_root, LibraryPathInfo};

#[tauri::command]
pub fn get_reader_settings() -> Result<String, String> {
    settings::get_reader_settings_json().map_err(to_cmd_err)
}

#[tauri::command]
pub fn save_reader_settings(settings_json: String) -> Result<(), String> {
    settings::save_reader_settings_json(&settings_json).map_err(to_cmd_err)
}

#[tauri::command]
pub fn get_library_path() -> Result<LibraryPathInfo, String> {
    path_info().ok_or_else(|| "书库尚未初始化".to_string())
}

/// 设置自定义书库文件夹（保存后需重启应用）
#[tauri::command]
pub fn set_library_path(app: tauri::AppHandle, path: String) -> Result<String, String> {
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("无法获取应用数据目录: {e}"))?;
    set_custom_root(&app_data, std::path::PathBuf::from(path)).map_err(to_cmd_err)?;
    Ok("已保存新书库位置，请完全退出并重新打开应用后生效。".into())
}

/// 恢复系统默认书库位置
#[tauri::command]
pub fn reset_library_path(app: tauri::AppHandle) -> Result<String, String> {
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("无法获取应用数据目录: {e}"))?;
    clear_custom_root(&app_data).map_err(to_cmd_err)?;
    Ok("已恢复默认书库位置，请完全退出并重新打开应用后生效。".into())
}
