//! 书源管理、规则校验、目录/正文解析、零配置接入

use crate::services::{source, to_cmd_err};
use crate::spider::{AutoConnectResult, DetectLogger, DetectResponse};
use crate::storage::{SourceHealth, SourceRecord, SourceSubscriptionRecord, SubscribeBatchResult};

#[tauri::command]
pub fn validate_source_rule(rule_json: String) -> Result<String, String> {
    source::validate_source_rule(&rule_json).map_err(to_cmd_err)
}

#[tauri::command]
pub fn fetch_chapters(url: String, rule_json: String) -> Result<Vec<crate::spider::Chapter>, String> {
    source::fetch_chapters(&url, &rule_json).map_err(to_cmd_err)
}

#[tauri::command]
pub fn get_book_content(chapter_url: String, rule_json: String) -> Result<String, String> {
    source::fetch_chapter_content(&chapter_url, &rule_json).map_err(to_cmd_err)
}

#[tauri::command]
pub fn list_book_sources() -> Result<Vec<SourceRecord>, String> {
    source::list_book_sources().map_err(to_cmd_err)
}

#[tauri::command]
pub fn toggle_book_source(source_id: String, enabled: bool) -> Result<SourceRecord, String> {
    source::toggle_book_source(&source_id, enabled).map_err(to_cmd_err)
}

#[tauri::command]
pub fn subscribe_remote_source(url: String) -> Result<SubscribeBatchResult, String> {
    source::subscribe_remote_source(&url).map_err(to_cmd_err)
}

#[tauri::command]
pub fn import_book_sources_batch(rule_json: String) -> Result<SubscribeBatchResult, String> {
    source::import_book_sources_batch(&rule_json).map_err(to_cmd_err)
}

#[tauri::command]
pub fn import_book_source_json(rule_json: String) -> Result<SourceRecord, String> {
    source::import_book_source_json(&rule_json).map_err(to_cmd_err)
}

#[tauri::command]
pub fn list_source_subscriptions() -> Result<Vec<SourceSubscriptionRecord>, String> {
    source::list_source_subscriptions().map_err(to_cmd_err)
}

#[tauri::command]
pub fn sync_source_subscription(sub_id: String) -> Result<SubscribeBatchResult, String> {
    source::sync_source_subscription(&sub_id).map_err(to_cmd_err)
}

#[tauri::command]
pub fn ping_book_source(source_id: String) -> Result<SourceHealth, String> {
    source::ping_source(&source_id).map_err(to_cmd_err)
}

#[tauri::command]
pub fn ping_book_sources(source_ids: Vec<String>) -> Result<Vec<SourceHealth>, String> {
    source::ping_sources_batch(source_ids).map_err(to_cmd_err)
}

#[tauri::command]
pub fn delete_book_sources(source_ids: Vec<String>) -> Result<usize, String> {
    source::delete_sources_batch(source_ids).map_err(to_cmd_err)
}

#[tauri::command]
pub fn set_book_sources_enabled(source_ids: Vec<String>, enabled: bool) -> Result<usize, String> {
    source::set_sources_enabled_batch(source_ids, enabled).map_err(to_cmd_err)
}

#[tauri::command]
pub fn auto_detect_selectors_cmd(
    toc_url: String,
    content_url: String,
    search_url: Option<String>,
) -> Result<crate::spider::DetectedSelectors, String> {
    source::auto_detect_selectors(&toc_url, &content_url, search_url.as_deref())
        .map_err(to_cmd_err)
}

#[tauri::command]
pub fn auto_detect_source_rule(
    name: String,
    toc_url: String,
    content_url: String,
    search_url: Option<String>,
) -> Result<DetectResponse<String>, String> {
    let mut log = DetectLogger::new();
    match source::auto_detect_source_rule_json_with_logs(
        &name,
        &toc_url,
        &content_url,
        search_url.as_deref(),
        &mut log,
    ) {
        Ok(rule_json) => Ok(DetectResponse::ok(rule_json, log)),
        Err(e) => Ok(DetectResponse::fail(e.to_string(), log)),
    }
}

#[tauri::command]
pub fn auto_detect_from_url(url: String) -> Result<DetectResponse<AutoConnectResult>, String> {
    Ok(source::auto_detect_from_url_with_logs(&url))
}

#[tauri::command]
pub fn receive_captured_html(html: String) {
    source::receive_captured_html(html);
}

#[tauri::command]
pub fn auto_detect_from_url_rendered(
    app: tauri::AppHandle,
    url: String,
) -> Result<DetectResponse<AutoConnectResult>, String> {
    Ok(source::auto_detect_from_url_rendered_with_logs(&app, &url))
}
