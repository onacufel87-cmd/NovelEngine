//! Windows WebView2 运行时检测（便携版 release exe 启动前提示）

use std::path::Path;

/// Edge WebView2 固定 GUID（微软官方）
const WEBVIEW2_CLIENT_ID: &str = "{F3017226-FE2A-5C84-BE2A-EFB54386E44A}";

/// WebView2 官方离线安装包下载页
pub const WEBVIEW2_DOWNLOAD_URL: &str =
    "https://go.microsoft.com/fwlink/p/?LinkId=2124703";

/// 是否应做启动前检测（仅 release 便携 exe；dev 由 Tauri 自行报错）
pub fn should_check_at_startup() -> bool {
    !cfg!(debug_assertions)
}

/// 检测 WebView2 是否可用（注册表 + 常见安装目录，避免误杀）
#[cfg(windows)]
pub fn is_runtime_installed() -> bool {
    registry_has_webview2() || filesystem_has_webview2()
}

#[cfg(not(windows))]
pub fn is_runtime_installed() -> bool {
    true
}

/// Edge Update 注册表（Evergreen 安装器会写入，但部分机器没有）
#[cfg(windows)]
fn registry_has_webview2() -> bool {
    use std::process::Command;

    let keys = [
        format!(r"HKLM\SOFTWARE\Microsoft\EdgeUpdate\Clients\{WEBVIEW2_CLIENT_ID}"),
        format!(r"HKLM\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{WEBVIEW2_CLIENT_ID}"),
        format!(r"HKCU\Software\Microsoft\EdgeUpdate\Clients\{WEBVIEW2_CLIENT_ID}"),
    ];

    keys.iter().any(|key| {
        Command::new("reg")
            .args(["query", key, "/v", "pv"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    })
}

/// 固定版本 / 系统内置 WebView2 常见目录（本机即在此路径）
#[cfg(windows)]
fn filesystem_has_webview2() -> bool {
    const ROOTS: [&str; 2] = [
        r"C:\Program Files (x86)\Microsoft\EdgeWebView\Application",
        r"C:\Program Files\Microsoft\EdgeWebView\Application",
    ];

    ROOTS.iter().any(|root| dir_contains_webview2_exe(Path::new(root)))
}

#[cfg(windows)]
fn dir_contains_webview2_exe(root: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(root) else {
        return false;
    };

    entries.filter_map(Result::ok).any(|entry| {
        entry
            .path()
            .join("msedgewebview2.exe")
            .is_file()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn webview2_check_does_not_panic() {
        let _ = is_runtime_installed();
    }
}
