//! 书库位置配置：保存在 app_data 根目录，不随书库迁移而丢失

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::utils::{AppError, AppResult};

const CONFIG_NAME: &str = "library_config.json";

#[derive(Debug, Default, Serialize, Deserialize)]
struct LibraryConfig {
    /// 用户自定义书库根目录；为空则使用系统默认
    custom_root: Option<String>,
}

fn config_path(app_data: &Path) -> PathBuf {
    app_data.join(CONFIG_NAME)
}

fn load_config(app_data: &Path) -> LibraryConfig {
    let path = config_path(app_data);
    if !path.is_file() {
        return LibraryConfig::default();
    }
    fs::read_to_string(&path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

fn save_config(app_data: &Path, config: &LibraryConfig) -> AppResult<()> {
    fs::create_dir_all(app_data).map_err(|e| {
        AppError::Database(format!("创建配置目录失败: {e}"))
    })?;
    let json = serde_json::to_string_pretty(config)
        .map_err(|e| AppError::Database(format!("序列化书库配置失败: {e}")))?;
    fs::write(config_path(app_data), json).map_err(|e| {
        AppError::Database(format!("写入书库配置失败: {e}"))
    })?;
    Ok(())
}

/// 系统默认书库：{当前用户 AppData}/com.novel.reader.core/library/
pub fn default_library_root(app_data: &Path) -> PathBuf {
    app_data.join("library")
}

/// 实际使用的书库根目录（优先读用户自定义）
pub fn resolve_library_root(app_data: &Path) -> PathBuf {
    let config = load_config(app_data);
    if let Some(custom) = config.custom_root {
        let path = PathBuf::from(&custom);
        if path.is_absolute() {
            return path;
        }
    }
    default_library_root(app_data)
}

pub fn is_custom_path(app_data: &Path) -> bool {
    load_config(app_data).custom_root.is_some()
}

/// 设置自定义书库目录（重启应用后生效）
pub fn set_custom_root(app_data: &Path, path: PathBuf) -> AppResult<()> {
    if !path.is_absolute() {
        return Err(AppError::Database("书库路径必须是绝对路径".into()));
    }
    let mut config = load_config(app_data);
    config.custom_root = Some(path.to_string_lossy().into_owned());
    save_config(app_data, &config)
}

/// 恢复系统默认书库位置
pub fn clear_custom_root(app_data: &Path) -> AppResult<()> {
    let mut config = load_config(app_data);
    config.custom_root = None;
    save_config(app_data, &config)
}
