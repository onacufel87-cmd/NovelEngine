//! 书源管理、规则校验、解析、零配置接入

use crate::spider::{
    self, auto_detect_selectors_with_log, detect_from_single_url_rendered_with_log,
    detect_from_single_url_with_log, detected_to_book_source_json, store_captured_html,
    AutoConnectResult, Chapter, DetectedSelectors, DetectLogger, DetectResponse,
};
use crate::storage::{
    add_remote_source, delete_book_sources, import_source_json, import_sources_batch,
    list_sources, list_subscriptions, ping_book_source, ping_book_sources, set_source_enabled,
    set_sources_enabled, sync_subscription, SourceHealth, SourceRecord, SourceSubscriptionRecord,
    SubscribeBatchResult,
};
use crate::utils::AppResult;

/// 校验书源 JSON
pub fn validate_source_rule(rule_json: &str) -> AppResult<String> {
    let source = spider::rule::parse_book_source(rule_json)?;
    Ok(format!("书源「{}」规则有效", source.name))
}

/// 解析目录页
pub fn fetch_chapters(url: &str, rule_json: &str) -> AppResult<Vec<Chapter>> {
    spider::fetch_and_parse_chapters(url, rule_json)
}

/// 抓取并清洗章节正文
pub fn fetch_chapter_content(chapter_url: &str, rule_json: &str) -> AppResult<String> {
    spider::fetch_and_parse_content(chapter_url, rule_json)
        .map(|text| spider::cleaner::apply_global_clean(&text))
}

/// 书源 CRUD
pub fn list_book_sources() -> AppResult<Vec<SourceRecord>> {
    list_sources()
}

pub fn toggle_book_source(source_id: &str, enabled: bool) -> AppResult<SourceRecord> {
    set_source_enabled(source_id, enabled)
}

pub fn subscribe_remote_source(url: &str) -> AppResult<SubscribeBatchResult> {
    add_remote_source(url)
}

pub fn import_book_sources_batch(rule_json: &str) -> AppResult<SubscribeBatchResult> {
    import_sources_batch(rule_json)
}

pub fn import_book_source_json(rule_json: &str) -> AppResult<SourceRecord> {
    import_source_json(rule_json)
}

pub fn list_source_subscriptions() -> AppResult<Vec<SourceSubscriptionRecord>> {
    list_subscriptions()
}

pub fn sync_source_subscription(sub_id: &str) -> AppResult<SubscribeBatchResult> {
    sync_subscription(sub_id)
}

pub fn ping_source(source_id: &str) -> AppResult<SourceHealth> {
    ping_book_source(source_id)
}

pub fn ping_sources_batch(source_ids: Vec<String>) -> AppResult<Vec<SourceHealth>> {
    ping_book_sources(&source_ids)
}

pub fn delete_sources_batch(source_ids: Vec<String>) -> AppResult<usize> {
    delete_book_sources(&source_ids)
}

pub fn set_sources_enabled_batch(source_ids: Vec<String>, enabled: bool) -> AppResult<usize> {
    set_sources_enabled(&source_ids, enabled)
}

/// 自动检测 CSS 选择器
pub fn auto_detect_selectors(
    toc_url: &str,
    content_url: &str,
    search_url: Option<&str>,
) -> AppResult<DetectedSelectors> {
    spider::auto_detect_selectors(toc_url, content_url, search_url)
}

/// 带日志的手动双 URL 检测，返回规则 JSON
pub fn auto_detect_source_rule_json_with_logs(
    name: &str,
    toc_url: &str,
    content_url: &str,
    search_url: Option<&str>,
    log: &mut DetectLogger,
) -> AppResult<String> {
    log.info(format!("Manual detect: catalog={toc_url}"));
    log.info(format!("Manual detect: content={content_url}"));
    let detected = auto_detect_selectors_with_log(toc_url, content_url, search_url, log)?;
    let mut json = detected_to_book_source_json(name, search_url, &detected);
    if let Some(obj) = json.as_object_mut() {
        obj.insert("_confidence".to_string(), serde_json::json!(detected.confidence));
    }
    serde_json::to_string_pretty(&json).map_err(|e| crate::utils::AppError::Parse(e.to_string()))
}

/// 检测并生成书源规则 JSON
pub fn auto_detect_source_rule_json(
    name: &str,
    toc_url: &str,
    content_url: &str,
    search_url: Option<&str>,
) -> AppResult<String> {
    auto_detect_source_rule_json_with_logs(name, toc_url, content_url, search_url, &mut DetectLogger::new())
}

/// 一键全自动接入（带日志响应）
pub fn auto_detect_from_url_with_logs(url: &str) -> DetectResponse<AutoConnectResult> {
    let mut log = DetectLogger::new();
    match detect_from_single_url_with_log(url, &mut log) {
        Ok(result) => DetectResponse::ok(result, log),
        Err(e) => DetectResponse::fail(e.to_string(), log),
    }
}

/// 一键全自动接入
pub fn auto_detect_from_url(url: &str) -> AppResult<AutoConnectResult> {
    detect_from_single_url_with_log(url, &mut DetectLogger::new())
}

/// WebView 渲染后一键接入（带日志响应）
pub fn auto_detect_from_url_rendered_with_logs(
    app: &tauri::AppHandle,
    url: &str,
) -> DetectResponse<AutoConnectResult> {
    let mut log = DetectLogger::new();
    match detect_from_single_url_rendered_with_log(app, url, &mut log) {
        Ok(result) => DetectResponse::ok(result, log),
        Err(e) => DetectResponse::fail(e.to_string(), log),
    }
}

/// WebView 渲染后一键接入
pub fn auto_detect_from_url_rendered(
    app: &tauri::AppHandle,
    url: &str,
) -> AppResult<AutoConnectResult> {
    detect_from_single_url_rendered_with_log(app, url, &mut DetectLogger::new())
}

/// 接收 WebView 回传的 HTML
pub fn receive_captured_html(html: String) {
    store_captured_html(html);
}
