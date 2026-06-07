use rusqlite::params;
use serde::{Deserialize, Serialize};

use super::db::get_connection;
use crate::spider::rule::{parse_book_source, BookSource};
use crate::utils::{AppError, AppResult};

/// 编译期嵌入的内置公版书源
const BUILTIN_SOURCES_JSON: &str = include_str!("../../resources/builtin_sources.json");

/// 已废弃的演示书源 ID（启动时从数据库清除）
const DEPRECATED_SOURCE_IDS: &[&str] = &["demo-local"];

/// 书源记录（数据库 + API 返回）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceRecord {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub rule_json: String,
    pub enabled: bool,
    pub is_builtin: bool,
    pub last_verified: Option<i64>,
    /// 来源：builtin | subscription | custom
    #[serde(default = "default_origin")]
    pub origin: String,
    /// 若来自仓库订阅，记录仓库 URL
    pub subscription_url: Option<String>,
    /// JSON 标签数组，如 ["ZH","公版"]
    pub tags: Option<String>,
    pub ping_ms: Option<i64>,
    /// online | slow | offline | unknown
    #[serde(default = "default_health")]
    pub health_status: String,
}

fn default_origin() -> String {
    "custom".into()
}

fn default_health() -> String {
    "unknown".into()
}

/// 单书源健康检测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceHealth {
    pub source_id: String,
    pub ping_ms: Option<u64>,
    pub health_status: String,
    pub last_verified: i64,
    pub error: Option<String>,
}

/// 已订阅的书源仓库
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceSubscriptionRecord {
    pub id: String,
    pub url: String,
    pub label: Option<String>,
    pub last_synced_at: Option<i64>,
    /// 该仓库导入的书源数量
    pub source_count: usize,
}

/// 批量订阅书源仓库的结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribeBatchResult {
    pub imported: usize,
    pub updated: usize,
    pub failed: usize,
    pub names: Vec<String>,
    pub errors: Vec<String>,
    /// 兼容单条导入 API，不序列化到前端
    #[serde(skip)]
    pub last_record: Option<SourceRecord>,
}

#[derive(Debug, Deserialize)]
struct BuiltinEntry {
    #[serde(default)]
    id: String,
    name: String,
    description: Option<String>,
    /// 是否默认启用（内置公版源可设为 false）
    #[serde(default = "default_enabled")]
    enabled: bool,
    rule: BookSource,
}

fn default_enabled() -> bool {
    true
}

/// 启动时同步内置书源（新用户默认启用 Gutenberg；不覆盖用户手动开关）
pub fn init_builtin_sources() -> AppResult<()> {
    remove_deprecated_sources()?;

    let entries: Vec<BuiltinEntry> = serde_json::from_str(BUILTIN_SOURCES_JSON)
        .map_err(|e| AppError::InvalidRule(format!("内置书源 JSON 无效: {e}")))?;

    for entry in entries {
        let rule_json = serde_json::to_string(&entry.rule)
            .map_err(|e| AppError::InvalidRule(e.to_string()))?;

        if get_source_by_id(&entry.id).is_ok() {
            // 已存在：仅同步名称/描述/规则，保留用户设置的 enabled
            sync_builtin_entry(&entry.id, &entry.name, entry.description.as_deref(), &rule_json)?;
        } else {
            upsert_source(
                &entry.id,
                &entry.name,
                entry.description.as_deref(),
                &rule_json,
                entry.enabled,
                true,
                "builtin",
                None,
                infer_builtin_tags(&entry.id, &entry.name),
            )?;
        }
    }

    Ok(())
}

/// 清除已下线的演示书源，并清空可能含假数据的搜索缓存
fn remove_deprecated_sources() -> AppResult<()> {
    let conn = get_connection()?;
    for id in DEPRECATED_SOURCE_IDS {
        // 不限 is_builtin，彻底移除旧版 demo-local
        conn.execute("DELETE FROM book_sources WHERE id = ?1", params![id])
            .map_err(|e| AppError::Database(e.to_string()))?;
    }
    // 移除指向 localhost 演示 HTML 的书源（含用户误导入的 test-rule.json）
    conn.execute(
        "DELETE FROM book_sources WHERE rule_json LIKE '%test-search.html%'
         OR rule_json LIKE '%test-catalog.html%'
         OR rule_json LIKE '%test-rank-%'",
        [],
    )
    .map_err(|e| AppError::Database(e.to_string()))?;
    // 旧缓存可能仍返回「引擎测试小说」等演示结果
    conn.execute("DELETE FROM search_cache", [])
        .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(())
}

/// 是否为本地演示书源（不应参与真实搜索）
pub fn is_demo_source(source: &SourceRecord) -> bool {
    DEPRECATED_SOURCE_IDS.contains(&source.id.as_str())
        || source.rule_json.contains("test-search.html")
        || source.rule_json.contains("test-catalog.html")
        || source.name.contains("演示")
}

/// 更新已有内置书源的元数据（不改动 enabled，尊重用户选择）
fn sync_builtin_entry(
    id: &str,
    name: &str,
    description: Option<&str>,
    rule_json: &str,
) -> AppResult<()> {
    let conn = get_connection()?;
    conn.execute(
        "UPDATE book_sources SET name = ?1, description = ?2, rule_json = ?3, updated_at = ?4
         WHERE id = ?5 AND is_builtin = 1",
        params![name, description, rule_json, unix_now(), id],
    )
    .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(())
}

/// 列出所有书源
pub fn list_sources() -> AppResult<Vec<SourceRecord>> {
    let conn = get_connection()?;
    let mut stmt = conn
        .prepare(
            "SELECT id, name, description, rule_json, enabled, is_builtin, last_verified,
                    origin, subscription_url, tags, ping_ms, health_status
             FROM book_sources ORDER BY is_builtin DESC, name ASC",
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

    let rows = stmt
        .query_map([], map_source_row)
        .map_err(|e| AppError::Database(e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(rows)
}

/// 列出已启用的书源
pub fn list_enabled_sources() -> AppResult<Vec<SourceRecord>> {
    Ok(list_sources()?
        .into_iter()
        .filter(|s| s.enabled)
        .collect())
}

/// 按 ID 获取书源
pub fn get_source_by_id(id: &str) -> AppResult<SourceRecord> {
    let conn = get_connection()?;
    conn.query_row(
        "SELECT id, name, description, rule_json, enabled, is_builtin, last_verified,
                origin, subscription_url, tags, ping_ms, health_status
         FROM book_sources WHERE id = ?1",
        params![id],
        map_source_row,
    )
    .map_err(|e| AppError::Database(format!("书源 id={id} 不存在: {e}")))
}

/// 设置书源启用状态
pub fn set_source_enabled(id: &str, enabled: bool) -> AppResult<SourceRecord> {
    let conn = get_connection()?;
    let changed = conn
        .execute(
            "UPDATE book_sources SET enabled = ?1, updated_at = ?2 WHERE id = ?3",
            params![enabled as i32, unix_now(), id],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

    if changed == 0 {
        return Err(AppError::Database(format!("书源 id={id} 不存在")));
    }

    get_source_by_id(id)
}

/// 从 JSON 文本导入书源（用户粘贴单条规则或数组）
pub fn import_source_json(body: &str) -> AppResult<SourceRecord> {
    let result = import_sources_batch(body)?;
    result
        .last_record
        .ok_or_else(|| AppError::InvalidRule("书源列表为空".into()))
}

/// 从远程 URL 拉取书源仓库 JSON 并批量入库
pub fn add_remote_source(url: &str) -> AppResult<SubscribeBatchResult> {
    validate_remote_repo_url(url)?;
    let body = crate::spider::fetch_html(url)?;
    let result = import_sources_batch_with_origin(&body, "subscription", Some(url))?;
    upsert_subscription(url, None)?;
    Ok(result)
}

/// 列出已订阅的书源仓库
pub fn list_subscriptions() -> AppResult<Vec<SourceSubscriptionRecord>> {
    let conn = get_connection()?;
    let mut stmt = conn
        .prepare(
            "SELECT id, url, label, last_synced_at FROM source_subscriptions ORDER BY last_synced_at DESC",
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<i64>>(3)?,
            ))
        })
        .map_err(|e| AppError::Database(e.to_string()))?
        .filter_map(|r| r.ok())
        .map(|(id, url, label, last_synced_at)| {
            let source_count = count_sources_by_subscription(&url).unwrap_or(0);
            SourceSubscriptionRecord {
                id,
                url,
                label,
                last_synced_at,
                source_count,
            }
        })
        .collect();

    Ok(rows)
}

/// 重新同步指定订阅仓库
pub fn sync_subscription(sub_id: &str) -> AppResult<SubscribeBatchResult> {
    let conn = get_connection()?;
    let url: String = conn
        .query_row(
            "SELECT url FROM source_subscriptions WHERE id = ?1",
            params![sub_id],
            |row| row.get(0),
        )
        .map_err(|e| AppError::Database(format!("订阅 id={sub_id} 不存在: {e}")))?;

    validate_remote_repo_url(&url)?;
    let body = crate::spider::fetch_html(&url)?;
    let result = import_sources_batch_with_origin(&body, "subscription", Some(&url))?;
    touch_subscription_sync(&sub_id)?;
    Ok(result)
}

/// 探测单书源网络可达性并更新健康字段
pub fn ping_book_source(source_id: &str) -> AppResult<SourceHealth> {
    let source = get_source_by_id(source_id)?;
    let probe_url = resolve_probe_url(&source.rule_json)?;
    let started = std::time::Instant::now();

    match crate::spider::fetch_html(&probe_url) {
        Ok(_) => {
            let ping_ms = started.elapsed().as_millis() as u64;
            let status = if ping_ms > 2000 { "slow" } else { "online" };
            update_source_health(source_id, Some(ping_ms as i64), status)?;
            Ok(SourceHealth {
                source_id: source_id.to_string(),
                ping_ms: Some(ping_ms),
                health_status: status.to_string(),
                last_verified: unix_now(),
                error: None,
            })
        }
        Err(e) => {
            update_source_health(source_id, None, "offline")?;
            Ok(SourceHealth {
                source_id: source_id.to_string(),
                ping_ms: None,
                health_status: "offline".to_string(),
                last_verified: unix_now(),
                error: Some(e.to_string()),
            })
        }
    }
}

/// 批量探测书源
pub fn ping_book_sources(source_ids: &[String]) -> AppResult<Vec<SourceHealth>> {
    let mut results = Vec::with_capacity(source_ids.len());
    for id in source_ids {
        results.push(ping_book_source(id)?);
    }
    Ok(results)
}

/// 批量删除非内置书源
pub fn delete_book_sources(source_ids: &[String]) -> AppResult<usize> {
    let conn = get_connection()?;
    let mut deleted = 0usize;
    for id in source_ids {
        let changed = conn
            .execute(
                "DELETE FROM book_sources WHERE id = ?1 AND is_builtin = 0",
                params![id],
            )
            .map_err(|e| AppError::Database(e.to_string()))?;
        deleted += changed;
    }
    Ok(deleted)
}

/// 批量设置启用状态
pub fn set_sources_enabled(source_ids: &[String], enabled: bool) -> AppResult<usize> {
    let conn = get_connection()?;
    let mut changed = 0usize;
    for id in source_ids {
        let n = conn
            .execute(
                "UPDATE book_sources SET enabled = ?1, updated_at = ?2 WHERE id = ?3",
                params![enabled as i32, unix_now(), id],
            )
            .map_err(|e| AppError::Database(e.to_string()))?;
        changed += n;
    }
    Ok(changed)
}

/// 校验远程书源仓库 URL（仅允许 http(s)，生产环境建议 https）
pub fn validate_remote_repo_url(url: &str) -> AppResult<()> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return Err(AppError::InvalidRule("书源仓库 URL 不能为空".into()));
    }

    let parsed = url::Url::parse(trimmed)
        .map_err(|e| AppError::InvalidRule(format!("书源仓库 URL 无效: {e}")))?;

    match parsed.scheme() {
        "https" => Ok(()),
        // 开发环境允许本地 http 测试
        "http" if cfg!(debug_assertions) => Ok(()),
        "http" => Err(AppError::InvalidRule(
            "请使用 https 书源仓库链接（http 仅开发模式可用）".into(),
        )),
        other => Err(AppError::InvalidRule(format!(
            "不支持的 URL 协议「{other}」，仅允许 http(s)"
        ))),
    }
}

/// 批量导入书源（支持社区仓库常见格式）
pub fn import_sources_batch(body: &str) -> AppResult<SubscribeBatchResult> {
    import_sources_batch_with_origin(body, "custom", None)
}

/// 带来源标记的批量导入
pub fn import_sources_batch_with_origin(
    body: &str,
    origin: &str,
    subscription_url: Option<&str>,
) -> AppResult<SubscribeBatchResult> {
    let items = normalize_source_list_json(body)?;
    let mut result = SubscribeBatchResult {
        imported: 0,
        updated: 0,
        failed: 0,
        names: Vec::new(),
        errors: Vec::new(),
        last_record: None,
    };

    for (idx, item) in items.into_iter().enumerate() {
        match import_single_source_value(&item, origin, subscription_url) {
            Ok((record, is_new)) => {
                if is_new {
                    result.imported += 1;
                } else {
                    result.updated += 1;
                }
                result.names.push(record.name.clone());
                result.last_record = Some(record);
            }
            Err(e) => {
                result.failed += 1;
                result.errors.push(format!("第 {} 条: {e}", idx + 1));
            }
        }
    }

    if result.imported + result.updated == 0 && result.failed > 0 {
        return Err(AppError::InvalidRule(format!(
            "未能导入任何书源: {}",
            result.errors.join("; ")
        )));
    }

    Ok(result)
}

/// 将社区仓库 JSON 规范化为书源对象数组
fn normalize_source_list_json(body: &str) -> AppResult<Vec<serde_json::Value>> {
    let trimmed = body.trim();
    let value: serde_json::Value = serde_json::from_str(trimmed)
        .map_err(|e| AppError::InvalidRule(format!("书源 JSON 无效: {e}")))?;

    match value {
        serde_json::Value::Array(arr) => Ok(arr),
        serde_json::Value::Object(map) => {
            for key in ["sources", "data", "list", "bookSources", "book_sources", "items"] {
                if let Some(arr) = map.get(key).and_then(|v| v.as_array()) {
                    return Ok(arr.clone());
                }
            }
            // 单条书源对象
            if map.contains_key("chapter_list_selector") || map.contains_key("content_selector") {
                return Ok(vec![serde_json::Value::Object(map)]);
            }
            Err(AppError::InvalidRule(
                "无法识别书源仓库格式，期望数组或 { sources: [...] }".into(),
            ))
        }
        _ => Err(AppError::InvalidRule("书源 JSON 必须是对象或数组".into())),
    }
}

/// 导入单条书源 JSON 值，返回 (记录, 是否新建)
fn import_single_source_value(
    item: &serde_json::Value,
    origin: &str,
    subscription_url: Option<&str>,
) -> AppResult<(SourceRecord, bool)> {
    // 格式 A: { id, name, description, enabled, rule: {...} }
    if let Ok(entry) = serde_json::from_value::<BuiltinEntry>(item.clone()) {
        let rule_json = serde_json::to_string(&entry.rule)
            .map_err(|e| AppError::InvalidRule(e.to_string()))?;
        parse_book_source(&rule_json)?;
        let id = if entry.id.is_empty() {
            slug_id(&entry.name)
        } else {
            entry.id
        };
        let is_new = get_source_by_id(&id).is_err();
        upsert_source(
            &id,
            &entry.name,
            entry.description.as_deref(),
            &rule_json,
            entry.enabled,
            false,
            origin,
            subscription_url,
            None,
        )?;
        return Ok((get_source_by_id(&id)?, is_new));
    }

    // 格式 B: 直接 BookSource 字段
    let source: BookSource = serde_json::from_value(item.clone())
        .map_err(|e| AppError::InvalidRule(format!("书源条目无效: {e}")))?;
    parse_book_source(&serde_json::to_string(&source).map_err(|e| AppError::InvalidRule(e.to_string()))?)?;
    let rule_json = serde_json::to_string(&source).map_err(|e| AppError::InvalidRule(e.to_string()))?;
    let id = slug_id(&source.name);
    let is_new = get_source_by_id(&id).is_err();
    upsert_source(
        &id,
        &source.name,
        Some("书源仓库导入"),
        &rule_json,
        true,
        false,
        origin,
        subscription_url,
        None,
    )?;
    Ok((get_source_by_id(&id)?, is_new))
}

fn upsert_source(
    id: &str,
    name: &str,
    description: Option<&str>,
    rule_json: &str,
    enabled: bool,
    is_builtin: bool,
    origin: &str,
    subscription_url: Option<&str>,
    tags: Option<&str>,
) -> AppResult<()> {
    let conn = get_connection()?;
    let now = unix_now();

    conn.execute(
        "INSERT INTO book_sources (
            id, name, description, rule_json, enabled, is_builtin,
            origin, subscription_url, tags, created_at, updated_at
         )
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
         ON CONFLICT(id) DO UPDATE SET
           name = excluded.name,
           description = excluded.description,
           rule_json = excluded.rule_json,
           origin = CASE WHEN book_sources.is_builtin = 1 THEN book_sources.origin ELSE excluded.origin END,
           subscription_url = COALESCE(excluded.subscription_url, book_sources.subscription_url),
           tags = COALESCE(excluded.tags, book_sources.tags),
           updated_at = excluded.updated_at",
        params![
            id,
            name,
            description,
            rule_json,
            enabled as i32,
            is_builtin as i32,
            origin,
            subscription_url,
            tags,
            now,
            now
        ],
    )
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(())
}

fn map_source_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<SourceRecord> {
    Ok(SourceRecord {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        rule_json: row.get(3)?,
        enabled: row.get::<_, i32>(4)? != 0,
        is_builtin: row.get::<_, i32>(5)? != 0,
        last_verified: row.get(6)?,
        origin: row.get::<_, Option<String>>(7)?.unwrap_or_else(|| "custom".into()),
        subscription_url: row.get(8)?,
        tags: row.get(9)?,
        ping_ms: row.get(10)?,
        health_status: row
            .get::<_, Option<String>>(11)?
            .unwrap_or_else(|| "unknown".into()),
    })
}

/// 从规则 JSON 解析可用于探测的首个 URL
fn resolve_probe_url(rule_json: &str) -> AppResult<String> {
    let source = parse_book_source(rule_json)?;

    if !source.search_url.is_empty() {
        return Ok(normalize_probe_url(&source.search_url));
    }

    if let Some(rank_urls) = &source.rank_urls {
        if let Some(first) = rank_urls.values().next() {
            return Ok(normalize_probe_url(first));
        }
    }

    Err(AppError::InvalidRule(
        "该书源无 search_url 或 rank_urls，无法探测".into(),
    ))
}

/// 将模板 URL 中的占位符替换为探测用固定值
fn normalize_probe_url(template: &str) -> String {
    let mut url = template.to_string();
    url = url.replace("{keyword}", "test");
    url = url.replace("{KEY}", "test");
    if url.contains("{origin}") {
        let origin = extract_base_origin(template).unwrap_or_else(|| "https://example.com".into());
        url = url.replace("{origin}", &origin);
    }
    url
}

/// 从含 {origin} 的模板里推断站点根 URL
fn extract_base_origin(template: &str) -> Option<String> {
    let sample = template
        .replace("{origin}", "https://probe.local")
        .replace("{keyword}", "x")
        .replace("{KEY}", "x");
    url::Url::parse(&sample).ok().and_then(|u| {
        u.host_str()
            .map(|host| format!("{}://{}", u.scheme(), host))
    })
}

fn update_source_health(source_id: &str, ping_ms: Option<i64>, status: &str) -> AppResult<()> {
    let conn = get_connection()?;
    let now = unix_now();
    conn.execute(
        "UPDATE book_sources SET ping_ms = ?1, health_status = ?2, last_verified = ?3, updated_at = ?4
         WHERE id = ?5",
        params![ping_ms, status, now, now, source_id],
    )
    .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(())
}

fn subscription_id_from_url(url: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    url.trim().hash(&mut h);
    format!("sub-{:x}", h.finish())
}

fn upsert_subscription(url: &str, label: Option<&str>) -> AppResult<()> {
    let conn = get_connection()?;
    let id = subscription_id_from_url(url);
    let now = unix_now();
    conn.execute(
        "INSERT INTO source_subscriptions (id, url, label, last_synced_at, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(url) DO UPDATE SET
           label = COALESCE(excluded.label, source_subscriptions.label),
           last_synced_at = excluded.last_synced_at",
        params![id, url.trim(), label, now, now],
    )
    .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(())
}

fn touch_subscription_sync(sub_id: &str) -> AppResult<()> {
    let conn = get_connection()?;
    conn.execute(
        "UPDATE source_subscriptions SET last_synced_at = ?1 WHERE id = ?2",
        params![unix_now(), sub_id],
    )
    .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(())
}

fn count_sources_by_subscription(url: &str) -> AppResult<usize> {
    let conn = get_connection()?;
    let n: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM book_sources WHERE subscription_url = ?1",
            params![url],
            |row| row.get(0),
        )
        .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(n as usize)
}

/// 内置书源展示标签
fn infer_builtin_tags(id: &str, name: &str) -> Option<&'static str> {
    if id.contains("gutenberg") || name.contains("Gutenberg") {
        return Some(r#"["内置","EN","公版"]"#);
    }
    if id.contains("wikisource") || name.contains("维基") {
        return Some(r#"["内置","ZH","公版"]"#);
    }
    if id.contains("openlibrary") || name.contains("Open Library") {
        return Some(r#"["内置","公版"]"#);
    }
    Some(r#"["内置","公版"]"#)
}

fn slug_id(name: &str) -> String {
    let slug: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else if ('\u{4e00}'..='\u{9fff}').contains(&c) {
                c
            } else {
                '-'
            }
        })
        .collect();

    format!("src-{}", slug.chars().take(24).collect::<String>())
}

fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod url_tests {
    use super::validate_remote_repo_url;

    #[test]
    fn allows_https_repo() {
        assert!(validate_remote_repo_url("https://example.com/sources.json").is_ok());
    }

    #[test]
    fn rejects_file_scheme() {
        assert!(validate_remote_repo_url("file:///etc/passwd").is_err());
    }
}
