//! 书库路径信息（返回给前端展示）

use serde::Serialize;

use super::config;

/// 书库路径说明（供设置页展示）
#[derive(Debug, Serialize)]
pub struct LibraryPathInfo {
    /// 当前生效的书库绝对路径
    pub path: String,
    /// default = 系统分配；custom = 用户自选文件夹
    pub mode: String,
    /// 默认路径（未自定义时与 path 相同）
    pub default_path: String,
    /// 面向用户的说明
    pub hint: String,
}

pub fn build_path_info(app_data: &std::path::Path, current_root: &std::path::Path) -> LibraryPathInfo {
    let default_path = config::default_library_root(app_data);
    let custom = config::is_custom_path(app_data);
    let mode = if custom { "custom" } else { "default" };

    let hint = if custom {
        "已使用自定义书库文件夹；恢复默认后需重启应用。".to_string()
    } else {
        "路径由 Windows 按当前登录用户自动分配，不同电脑/用户名会不同，并非写死某一地址。".to_string()
    };

    LibraryPathInfo {
        path: current_root.to_string_lossy().into_owned(),
        mode: mode.to_string(),
        default_path: default_path.to_string_lossy().into_owned(),
        hint,
    }
}
