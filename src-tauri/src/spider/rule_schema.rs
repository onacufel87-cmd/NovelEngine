//! 书源规则扩展校验（配合 JSON Schema 文档 resources/book_source.schema.json）

use super::rule::BookSource;
use crate::utils::{AppError, AppResult};

const ALLOWED_ENCODINGS: &[&str] = &["utf-8", "gbk", "gb2312", "big5"];

/// 在基础字段校验之上做语义检查
pub fn validate_book_source_extended(source: &BookSource) -> AppResult<()> {
    validate_selector("chapter_list_selector", &source.chapter_list_selector)?;
    validate_selector("content_selector", &source.content_selector)?;

    if let Some(ref enc) = source.encoding {
        let lower = enc.trim().to_lowercase();
        if !lower.is_empty() && !ALLOWED_ENCODINGS.contains(&lower.as_str()) {
            return Err(AppError::InvalidRule(format!(
                "encoding 不支持「{enc}」，可选: utf-8 / gbk / gb2312 / big5"
            )));
        }
    }

    if !source.search_url.trim().is_empty() {
        let has_search = source
            .search_result_selector
            .as_ref()
            .is_some_and(|s| !s.trim().is_empty());
        if !has_search {
            return Err(AppError::InvalidRule(
                "配置了 search_url 时必须提供 search_result_selector".into(),
            ));
        }
    }

    if let Some(ms) = source.request_interval_ms {
        if ms > 60_000 {
            return Err(AppError::InvalidRule(
                "request_interval_ms 不应超过 60000（60 秒）".into(),
            ));
        }
    }

    if let Some(ref patterns) = source.clean_patterns {
        for (idx, pat) in patterns.iter().enumerate() {
            if regex::Regex::new(pat).is_err() {
                return Err(AppError::InvalidRule(format!(
                    "clean_patterns[{idx}] 不是合法正则: {pat}"
                )));
            }
        }
    }

    Ok(())
}

fn validate_selector(field: &str, selector: &str) -> AppResult<()> {
    let s = selector.trim();
    if s.contains('\n') || s.contains('\r') {
        return Err(AppError::InvalidRule(format!("{field} 不能包含换行")));
    }
    if s.len() > 512 {
        return Err(AppError::InvalidRule(format!("{field} 过长（>512 字符）")));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spider::rule::BookSource;

    fn minimal_source() -> BookSource {
        BookSource {
            name: "测试".into(),
            search_url: String::new(),
            search_result_selector: None,
            search_title_selector: None,
            search_author_selector: None,
            search_link_selector: None,
            search_link_attr: None,
            book_list_selector: None,
            book_title_selector: None,
            rank_urls: None,
            chapter_list_selector: "#list a".into(),
            chapter_title_selector: "text".into(),
            content_selector: "#content".into(),
            next_page_selector: None,
            encoding: None,
            ad_keywords: None,
            clean_patterns: None,
            cookies: None,
            request_interval_ms: None,
        }
    }

    #[test]
    fn rejects_search_without_result_selector() {
        let mut s = minimal_source();
        s.search_url = "https://example.com/s?q={keyword}".into();
        assert!(validate_book_source_extended(&s).is_err());
    }

    #[test]
    fn rejects_invalid_clean_pattern() {
        let mut s = minimal_source();
        s.clean_patterns = Some(vec!["[invalid".into()]);
        assert!(validate_book_source_extended(&s).is_err());
    }
}
