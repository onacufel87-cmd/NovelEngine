mod commands;
mod crawler;
mod services;
mod spider;
mod storage;
mod utils;



use tauri::Manager;



#[cfg_attr(mobile, tauri::mobile_entry_point)]

pub fn run() {

    tauri::Builder::default()

        .plugin(tauri_plugin_dialog::init())

        .setup(|app| {

            // 便携版 release：缺少 WebView2 时弹窗（dev 模式跳过，避免误报阻断开发）
            #[cfg(windows)]
            if utils::webview2::should_check_at_startup()
                && !utils::webview2::is_runtime_installed()
            {
                use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

                app.dialog()
                    .message(format!(
                        "未检测到 Microsoft Edge WebView2 运行时，无法显示界面。\n\n\
                         请安装 WebView2 后重新启动本程序。\n\
                         下载地址：{}",
                        utils::webview2::WEBVIEW2_DOWNLOAD_URL
                    ))
                    .title("缺少运行依赖")
                    .kind(MessageDialogKind::Warning)
                    .buttons(MessageDialogButtons::Ok)
                    .blocking_show();

                std::process::exit(1);
            }

            // 窗口标题栏 / 任务栏图标：显式应用 bundle 嵌入的自定义图标
            if let Some(icon) = app.default_window_icon().cloned() {
                for (_, window) in app.webview_windows() {
                    let _ = window.set_icon(icon.clone());
                }
            }

            // 数据库放在应用数据目录，避免污染项目根目录

            let app_data = app
                .path()
                .app_data_dir()
                .map_err(|e| format!("无法获取数据目录: {e}"))?;

            storage::library::init(&app_data)
                .map_err(|e| format!("书库初始化失败: {e}"))?;

            storage::init_builtin_sources().map_err(|e| format!("内置书源初始化失败: {e}"))?;

            // 启动时加载网络抓取全局配置

            if let Ok(Some(json)) = storage::get_setting("reader_settings") {

                spider::apply_reader_settings_json(&json);

            }

            Ok(())

        })

        .invoke_handler(commands::handler())

        .run(tauri::generate_context!())

        .expect("error while running tauri application");

}

