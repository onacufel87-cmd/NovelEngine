use std::collections::HashSet;
use std::thread;

use serde::{Deserialize, Serialize};

use crate::spider::{
    fetch_for_source, parse_book_list, parse_search_results, providers,
    rule::parse_book_source,
};
use crate::storage::sources::{get_source_by_id, is_demo_source, list_enabled_sources, SourceRecord};
use crate::utils::{AppError, AppResult};

/// 搜索结果条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub author: Option<String>,
    pub catalog_url: String,
    pub source_id: String,
    pub source_name: String,
}

/// 榜单/列表中的书本信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookListItem {
    pub title: String,
    pub author: Option<String>,
    pub catalog_url: String,
}

const SEARCH_CACHE_TTL_SECS: i64 = 3600;
const RANK_CACHE_TTL_SECS: i64 = 6 * 3600;

/// 替换 URL 模板中的 {origin} 与 {keyword}
pub fn resolve_url_template(url: &str, origin: &str, keyword: Option<&str>) -> String {
    let mut resolved = url.replace("{origin}", origin.trim_end_matches('/'));
    if let Some(kw) = keyword {
        let encoded: String = url::form_urlencoded::byte_serialize(kw.as_bytes()).collect();
        resolved = resolved.replace("{keyword}", &encoded);
    }
    resolved
}

/// 在所有启用书源中并行搜索并聚合结果
pub fn search_books(keyword: &str, origin: &str) -> AppResult<Vec<SearchResult>> {
    let keyword = keyword.trim();
    if keyword.is_empty() {
        return Err(AppError::InvalidRule("搜索关键词不能为空".into()));
    }

    // 优先读缓存（1 小时有效）
    if let Some(cached) = get_search_cache(keyword, origin)? {
        return Ok(cached);
    }

    let sources = list_enabled_sources()?;
    if sources.is_empty() {
        return Err(AppError::InvalidRule("没有已启用的书源，请先在书源管理中添加".into()));
    }

    let keyword_owned = keyword.to_string();
    let origin_owned = origin.to_string();

    // 使用 std::thread 并行（项目使用 reqwest blocking）
    let handles: Vec<_> = sources
        .into_iter()
        .filter(|source| !is_demo_source(source))
        .map(|source| {
            let kw = keyword_owned.clone();
            let org = origin_owned.clone();
            thread::spawn(move || search_one_source(&source, &kw, &org))
        })
        .collect();

    let mut merged = Vec::new();
    for handle in handles {
        if let Ok(Ok(mut batch)) = handle.join() {
            merged.append(&mut batch);
        }
    }

    dedupe_search_results(&mut merged);
    set_search_cache(keyword, origin, &merged)?;
    Ok(merged)
}

/// 单个书源内搜索
fn search_one_source(
    source: &SourceRecord,
    keyword: &str,
    origin: &str,
) -> AppResult<Vec<SearchResult>> {
    // 内置公版源：优先走 Provider 专用搜索
    let items = if let Some(result) = providers::search_by_source_id(&source.id, keyword) {
        result?
    } else {
        search_by_rule(&source.rule_json, keyword, origin)?
    };

    Ok(items
        .into_iter()
        .map(|item| SearchResult {
            title: item.title,
            author: item.author,
            catalog_url: item.catalog_url,
            source_id: source.id.clone(),
            source_name: source.name.clone(),
        })
        .collect())
}

/// 通用 CSS 规则搜索（Gutenberg 等）
fn search_by_rule(rule_json: &str, keyword: &str, origin: &str) -> AppResult<Vec<crate::spider::parser::SearchResultItem>> {
    let rule = parse_book_source(rule_json)?;
    if rule.search_url.trim().is_empty() {
        return Ok(Vec::new());
    }

    let search_url = resolve_url_template(&rule.search_url, origin, Some(keyword));
    let html = fetch_for_source(&search_url, &rule)?;
    parse_search_results(&html, &rule, &search_url)
}

/// 抓取指定书源的榜单
pub fn fetch_rankings(source_id: &str, rank_type: &str, origin: &str) -> AppResult<Vec<BookListItem>> {
    if let Some(cached) = get_rank_cache(source_id, rank_type)? {
        return Ok(cached);
    }

    let source = get_source_by_id(source_id)?;
    let rule = parse_book_source(&source.rule_json)?;

    let rank_urls = rule
        .rank_urls
        .as_ref()
        .ok_or_else(|| AppError::InvalidRule("该书源未配置 rank_urls".into()))?;

    let rank_url = rank_urls
        .get(rank_type)
        .ok_or_else(|| AppError::InvalidRule(format!("无此榜单类型: {rank_type}")))?;

    let url = resolve_url_template(rank_url, origin, None);
    let html = fetch_for_source(&url, &rule)?;
    let parsed = parse_book_list(&html, &rule, &url)?;
    let books: Vec<BookListItem> = parsed
        .into_iter()
        .map(|item| BookListItem {
            title: item.title,
            author: item.author,
            catalog_url: item.catalog_url,
        })
        .collect();

    set_rank_cache(source_id, rank_type, &books)?;
    Ok(books)
}

/// 获取书源支持的榜单类型名称列表
pub fn list_rank_types(source_id: &str) -> AppResult<Vec<String>> {
    let source = get_source_by_id(source_id)?;
    let rule = parse_book_source(&source.rule_json)?;

    Ok(rule
        .rank_urls
        .as_ref()
        .map(|m| {
            let mut keys: Vec<String> = m.keys().cloned().collect();
            keys.sort();
            keys
        })
        .unwrap_or_default())
}

fn dedupe_search_results(results: &mut Vec<SearchResult>) {
    let mut seen = HashSet::new();
    results.retain(|item| {
        // 按书名 + 作者去重（多书源可能返回同一本书）
        let author = item.author.as_deref().unwrap_or("").trim().to_lowercase();
        let key = format!("{}|{}", item.title.trim().to_lowercase(), author);
        seen.insert(key)
    });
}

fn get_search_cache(keyword: &str, origin: &str) -> AppResult<Option<Vec<SearchResult>>> {
    let conn = crate::storage::db::get_connection()?;
    let row: Result<(String, i64), _> = conn.query_row(
        "SELECT results_json, cached_at FROM search_cache WHERE keyword = ?1 AND origin = ?2",
        rusqlite::params![keyword, origin],
        |row| Ok((row.get(0)?, row.get(1)?)),
    );

    match row {
        Ok((json, cached_at)) => {
            if unix_now() - cached_at > SEARCH_CACHE_TTL_SECS {
                return Ok(None);
            }
            let items: Vec<SearchResult> = serde_json::from_str(&json)
                .map_err(|e| AppError::Database(format!("搜索缓存解析失败: {e}")))?;
            Ok(Some(items))
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(AppError::Database(e.to_string())),
    }
}

fn set_search_cache(keyword: &str, origin: &str, results: &[SearchResult]) -> AppResult<()> {
    let json = serde_json::to_string(results).map_err(|e| AppError::Database(e.to_string()))?;
    let conn = crate::storage::db::get_connection()?;
    conn.execute(
        "INSERT INTO search_cache (keyword, origin, results_json, cached_at)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(keyword, origin) DO UPDATE SET
           results_json = excluded.results_json,
           cached_at = excluded.cached_at",
        rusqlite::params![keyword, origin, json, unix_now()],
    )
    .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(())
}

fn get_rank_cache(source_id: &str, rank_type: &str) -> AppResult<Option<Vec<BookListItem>>> {
    let conn = crate::storage::db::get_connection()?;
    let row: Result<(String, i64), _> = conn.query_row(
        "SELECT books_json, cached_at FROM rank_list WHERE source_id = ?1 AND rank_type = ?2",
        rusqlite::params![source_id, rank_type],
        |row| Ok((row.get(0)?, row.get(1)?)),
    );

    match row {
        Ok((json, cached_at)) => {
            if unix_now() - cached_at > RANK_CACHE_TTL_SECS {
                return Ok(None);
            }
            let items: Vec<BookListItem> = serde_json::from_str(&json)
                .map_err(|e| AppError::Database(format!("榜单缓存解析失败: {e}")))?;
            Ok(Some(items))
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(AppError::Database(e.to_string())),
    }
}

fn set_rank_cache(source_id: &str, rank_type: &str, books: &[BookListItem]) -> AppResult<()> {
    let json = serde_json::to_string(books).map_err(|e| AppError::Database(e.to_string()))?;
    let conn = crate::storage::db::get_connection()?;
    conn.execute(
        "INSERT INTO rank_list (source_id, rank_type, books_json, cached_at)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(source_id, rank_type) DO UPDATE SET
           books_json = excluded.books_json,
           cached_at = excluded.cached_at",
        rusqlite::params![source_id, rank_type, json, unix_now()],
    )
    .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(())
}

fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
