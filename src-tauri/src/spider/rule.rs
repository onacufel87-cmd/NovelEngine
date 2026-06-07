use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// 书源规则：描述如何从某个网站抓取小说
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookSource {
    /// 书源名称，如"示例书源"
    pub name: String,
    /// 搜索接口模板，如 "https://example.com/search?q={keyword}"
    #[serde(default)]
    pub search_url: String,
    /// 搜索结果页中每条结果的容器选择器（可选，启用搜索时需要）
    #[serde(default)]
    pub search_result_selector: Option<String>,
    /// 搜索结果中的书名选择器（相对 result 容器）
    /// 兼容别名 `result_title_selector`
    #[serde(default, alias = "result_title_selector")]
    pub search_title_selector: Option<String>,
    /// 搜索结果中的作者选择器（可选）
    #[serde(default)]
    pub search_author_selector: Option<String>,
    /// 搜索结果中的目录/详情链接选择器；设为 `self` 表示条目本身即为链接
    #[serde(default)]
    pub search_link_selector: Option<String>,
    /// 从链接元素读取 URL 的属性名，默认 `href`（兼容 `result_url_attr`）
    #[serde(default, alias = "result_url_attr")]
    pub search_link_attr: Option<String>,
    /// 榜单页书本条目选择器（可选，默认可复用 chapter_list_selector）
    #[serde(default)]
    pub book_list_selector: Option<String>,
    /// 榜单条目标题选择器（可选，默认可复用 chapter_title_selector）
    #[serde(default)]
    pub book_title_selector: Option<String>,
    /// 榜单分类 URL，键为榜单名称如「热门」「新书」
    #[serde(default)]
    pub rank_urls: Option<HashMap<String, String>>,
    /// 从目录页提取章节链接的 CSS 选择器
    pub chapter_list_selector: String,
    /// 提取章节标题：`text` 表示取链接自身文本，否则为子元素 CSS 选择器
    #[serde(default = "default_title_selector")]
    pub chapter_title_selector: String,
    /// 提取正文内容的选择器
    pub content_selector: String,
    /// 正文分页时"下一页"按钮的选择器（可选）
    pub next_page_selector: Option<String>,
    /// 网页编码，如 gbk、utf-8
    pub encoding: Option<String>,
    /// 广告关键词列表，字面量替换（可选）
    pub ad_keywords: Option<Vec<String>>,
    /// 自定义正文清洗正则列表（可选）
    pub clean_patterns: Option<Vec<String>>,
    /// 站点 Cookie（可选，从浏览器复制；覆盖全局 Cookie）
    #[serde(default)]
    pub cookies: Option<String>,
    /// 请求最小间隔毫秒（可选，覆盖全局限速）
    #[serde(default)]
    pub request_interval_ms: Option<u64>,
}

fn default_title_selector() -> String {
    "text".to_string()
}

/// 章节信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub title: String,
    pub url: String,
}

/// 从 JSON 字符串解析书源规则
pub fn parse_book_source(json: &str) -> crate::utils::AppResult<BookSource> {
    let source: BookSource = serde_json::from_str(json)
        .map_err(|e| crate::utils::AppError::InvalidRule(format!("JSON 格式错误: {e}")))?;
    validate_book_source(&source)?;
    super::rule_schema::validate_book_source_extended(&source)?;
    Ok(source)
}

/// 校验书源规则必填字段
pub fn validate_book_source(source: &BookSource) -> crate::utils::AppResult<()> {
    use crate::utils::AppError;

    if source.name.trim().is_empty() {
        return Err(AppError::InvalidRule("书源 name 不能为空".into()));
    }
    if source.chapter_list_selector.trim().is_empty() {
        return Err(AppError::InvalidRule("chapter_list_selector 不能为空".into()));
    }
    if source.content_selector.trim().is_empty() {
        return Err(AppError::InvalidRule("content_selector 不能为空".into()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_rule_json() {
        // 使用 r##"..."## 避免 JSON 内的 "# 与 raw string 定界符冲突
        let json = r##"{
            "name": "测试书源",
            "chapter_list_selector": "#list a",
            "content_selector": "#content"
        }"##;
        let source = parse_book_source(json).unwrap();
        assert_eq!(source.name, "测试书源");
        assert_eq!(source.chapter_title_selector, "text");
    }

    #[test]
    fn reject_missing_content_selector() {
        let json = r##"{"name":"x","chapter_list_selector":"#list a"}"##;
        assert!(parse_book_source(json).is_err());
    }
}
