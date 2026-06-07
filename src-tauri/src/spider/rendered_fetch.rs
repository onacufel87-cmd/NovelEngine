//! 通过隐藏 WebView 获取 JS 渲染后的 HTML（轻量绕过部分动态站）
//!
//! 注入脚本在页面 load 后调用 `receive_captured_html` 回传 HTML。
//! 需在 `tauri.conf.json` 启用 `withGlobalTauri: true`。

use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use tauri::{AppHandle, WebviewUrl, WebviewWindowBuilder};

use crate::utils::{AppError, AppResult};

static RENDER_CAPTURE: OnceLock<Mutex<Option<String>>> = OnceLock::new();

fn capture_slot() -> &'static Mutex<Option<String>> {
    RENDER_CAPTURE.get_or_init(|| Mutex::new(None))
}

/// 由 WebView 内注入脚本调用，回传渲染后 HTML
pub fn store_captured_html(html: String) {
    if let Ok(mut slot) = capture_slot().lock() {
        *slot = Some(html);
    }
}

/// 使用隐藏 WebView 加载 URL 并提取渲染后 HTML
pub fn fetch_html_rendered(app: &AppHandle, url: &str) -> AppResult<String> {
    let parsed = url
        .parse()
        .map_err(|e| AppError::Network(format!("URL 无效: {e}")))?;

    {
        let mut slot = capture_slot()
            .lock()
            .map_err(|e| AppError::Network(format!("锁失败: {e}")))?;
        *slot = None;
    }

    let label = format!(
        "render-fetch-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0)
    );

    // 页面 load 后延迟回传 HTML（等待 JS 渲染）
    let init_script = r#"
        window.addEventListener('load', function() {
            setTimeout(function() {
                try {
                    var html = document.documentElement.outerHTML;
                    if (window.__TAURI__ && window.__TAURI__.core) {
                        window.__TAURI__.core.invoke('receive_captured_html', { html: html });
                    }
                } catch (e) { console.error(e); }
            }, 2500);
        });
    "#;

    let webview = WebviewWindowBuilder::new(app, &label, WebviewUrl::External(parsed))
        .visible(false)
        .inner_size(1280.0, 800.0)
        .initialization_script(init_script)
        .build()
        .map_err(|e| AppError::Network(format!("创建 WebView 失败: {e}")))?;

    // 等待注入脚本 invoke 回传
    std::thread::sleep(Duration::from_secs(6));

    let html = capture_slot()
        .lock()
        .map_err(|e| AppError::Network(format!("锁失败: {e}")))?
        .take();

    let _ = webview.close();

    html.ok_or_else(|| {
        AppError::Network(
            "WebView 未能回传 HTML（可能站点禁止脚本或需人机验证）".into(),
        )
    })
}
