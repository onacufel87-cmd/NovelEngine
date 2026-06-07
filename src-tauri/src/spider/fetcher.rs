//! HTTP 抓取：浏览器头、Cookie、全局限速

use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use encoding_rs::{GBK, UTF_8};
use reqwest::blocking::{Client, RequestBuilder, Response};

use super::rule::BookSource;
use crate::utils::{AppError, AppResult};

/// 默认 User-Agent
const DEFAULT_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

/// 默认请求最小间隔（毫秒）
const DEFAULT_MIN_INTERVAL_MS: u64 = 1000;

/// 单次请求可选项
#[derive(Debug, Clone, Default)]
pub struct FetchOptions {
    pub encoding: Option<String>,
    /// 浏览器 Cookie 字符串，如 `a=1; b=2`
    pub cookies: Option<String>,
    /// 与上一请求的最小间隔（毫秒），0 表示不限速
    pub min_interval_ms: Option<u64>,
}

/// 全局抓取配置（来自设置页）
#[derive(Debug, Clone)]
pub struct GlobalFetchConfig {
    pub cookies: String,
    pub min_interval_ms: u64,
}

impl Default for GlobalFetchConfig {
    fn default() -> Self {
        Self {
            cookies: String::new(),
            min_interval_ms: DEFAULT_MIN_INTERVAL_MS,
        }
    }
}

static GLOBAL_CONFIG: OnceLock<Mutex<GlobalFetchConfig>> = OnceLock::new();
static LAST_REQUEST_AT: OnceLock<Mutex<Option<Instant>>> = OnceLock::new();

fn global_config() -> &'static Mutex<GlobalFetchConfig> {
    GLOBAL_CONFIG.get_or_init(|| Mutex::new(GlobalFetchConfig::default()))
}

fn last_request_at() -> &'static Mutex<Option<Instant>> {
    LAST_REQUEST_AT.get_or_init(|| Mutex::new(None))
}

/// 更新全局抓取配置（设置页保存 / 应用启动时调用）
pub fn set_global_fetch_config(config: GlobalFetchConfig) {
    if let Ok(mut g) = global_config().lock() {
        *g = config;
    }
}

/// 从 reader_settings JSON 同步全局配置
pub fn apply_reader_settings_json(json: &str) {
    let cookies = serde_json::from_str::<serde_json::Value>(json)
        .ok()
        .and_then(|v| v.get("fetchCookie").and_then(|x| x.as_str()).map(String::from))
        .unwrap_or_default();

    let min_interval_ms = serde_json::from_str::<serde_json::Value>(json)
        .ok()
        .and_then(|v| v.get("fetchMinIntervalMs").and_then(|x| x.as_u64()))
        .unwrap_or(DEFAULT_MIN_INTERVAL_MS);

    set_global_fetch_config(GlobalFetchConfig {
        cookies,
        min_interval_ms,
    });
}

impl FetchOptions {
    /// 合并全局配置与书源级配置（书源优先）
    pub fn from_source(source: &BookSource) -> Self {
        let global = global_config()
            .lock()
            .map(|g| g.clone())
            .unwrap_or_default();

        let cookies = source
            .cookies
            .as_ref()
            .filter(|c| !c.trim().is_empty())
            .cloned()
            .or_else(|| {
                if global.cookies.trim().is_empty() {
                    None
                } else {
                    Some(global.cookies.clone())
                }
            });

        let min_interval_ms = source
            .request_interval_ms
            .or(Some(global.min_interval_ms));

        Self {
            encoding: source.encoding.clone(),
            cookies,
            min_interval_ms,
        }
    }

    fn from_global_with_encoding(encoding: Option<&str>) -> Self {
        let global = global_config()
            .lock()
            .map(|g| g.clone())
            .unwrap_or_default();

        Self {
            encoding: encoding.map(String::from),
            cookies: if global.cookies.trim().is_empty() {
                None
            } else {
                Some(global.cookies.clone())
            },
            min_interval_ms: Some(global.min_interval_ms),
        }
    }
}

/// 下载 HTML（使用全局 Cookie / 限速）
pub fn fetch_html(url: &str) -> AppResult<String> {
    fetch_with_options(url, &FetchOptions::from_global_with_encoding(None))
}

/// 下载 HTML 并按编码解码（合并全局 Cookie / 限速）
pub fn fetch_html_with_encoding(url: &str, encoding: Option<&str>) -> AppResult<String> {
    fetch_with_options(url, &FetchOptions::from_global_with_encoding(encoding))
}

/// 按书源规则抓取（编码 + Cookie + 限速均来自书源/全局）
pub fn fetch_for_source(url: &str, source: &BookSource) -> AppResult<String> {
    fetch_with_options(url, &FetchOptions::from_source(source))
}

/// 自定义选项抓取
pub fn fetch_with_options(url: &str, options: &FetchOptions) -> AppResult<String> {
    let min_ms = options.min_interval_ms.unwrap_or(DEFAULT_MIN_INTERVAL_MS);
    apply_rate_limit(min_ms);

    let client = build_client()?;
    let response = send_get(&client, url, options)?;
    decode_response(response, options.encoding.as_deref())
}

fn apply_rate_limit(min_ms: u64) {
    if min_ms == 0 {
        return;
    }
    let wait_ms = {
        let last_guard = last_request_at().lock().unwrap();
        last_guard
            .map(|prev| {
                let elapsed = prev.elapsed().as_millis() as u64;
                min_ms.saturating_sub(elapsed)
            })
            .unwrap_or(0)
    };
    if wait_ms > 0 {
        std::thread::sleep(Duration::from_millis(wait_ms));
    }
    *last_request_at().lock().unwrap() = Some(Instant::now());
}

fn build_client() -> AppResult<Client> {
    Client::builder()
        .user_agent(DEFAULT_USER_AGENT)
        .redirect(reqwest::redirect::Policy::limited(10))
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| AppError::Network(e.to_string()))
}

/// 附加浏览器常见请求头
fn apply_browser_headers(mut req: RequestBuilder, url: &str) -> RequestBuilder {
    req = req
        .header(
            reqwest::header::ACCEPT,
            "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8",
        )
        .header(reqwest::header::ACCEPT_LANGUAGE, "zh-CN,zh;q=0.9,en;q=0.8")
        .header(reqwest::header::CACHE_CONTROL, "no-cache")
        .header("Sec-Fetch-Dest", "document")
        .header("Sec-Fetch-Mode", "navigate")
        .header("Sec-Fetch-User", "?1")
        .header("Upgrade-Insecure-Requests", "1");

    if let Ok(parsed) = url::Url::parse(url) {
        let origin = parsed.origin().ascii_serialization();
        req = req
            .header(reqwest::header::REFERER, format!("{origin}/"))
            .header("Sec-Fetch-Site", "same-origin");
    } else {
        req = req.header("Sec-Fetch-Site", "none");
    }

    req
}

fn send_get(client: &Client, url: &str, options: &FetchOptions) -> AppResult<Response> {
    let mut req = client.get(url);
    req = apply_browser_headers(req, url);

    if let Some(ref cookie) = options.cookies {
        let trimmed = cookie.trim();
        if !trimmed.is_empty() {
            req = req.header(reqwest::header::COOKIE, trimmed);
        }
    }

    let response = req
        .send()
        .map_err(|e| AppError::Network(format!("请求失败 ({url}): {e}")))?;

    if !response.status().is_success() {
        let status = response.status();
        let hint = if status.as_u16() == 403 {
            "：目标网站启用了反爬虫。可尝试在设置中粘贴浏览器 Cookie，或改用手动检测。"
        } else if status.as_u16() == 404 {
            "：页面不存在，请检查 URL。"
        } else {
            ""
        };
        return Err(AppError::Network(format!("HTTP {status}{hint} ({url})")));
    }

    Ok(response)
}

fn decode_response(response: Response, encoding: Option<&str>) -> AppResult<String> {
    let bytes = response
        .bytes()
        .map_err(|e| AppError::Network(e.to_string()))?;

    let enc = encoding.unwrap_or("utf-8").to_lowercase();
    let (cow, _, had_errors) = match enc.as_str() {
        "gbk" | "gb2312" | "gb18030" => GBK.decode(&bytes),
        _ => UTF_8.decode(&bytes),
    };

    if had_errors {
        return Err(AppError::Network(format!("网页编码 {enc} 解码失败")));
    }

    Ok(cow.into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_source_cookie_over_global() {
        set_global_fetch_config(GlobalFetchConfig {
            cookies: "global=1".into(),
            min_interval_ms: 1000,
        });

        let source = BookSource {
            name: "t".into(),
            search_url: String::new(),
            search_result_selector: None,
            search_title_selector: None,
            search_author_selector: None,
            search_link_selector: None,
            search_link_attr: None,
            book_list_selector: None,
            book_title_selector: None,
            rank_urls: None,
            chapter_list_selector: "#a".into(),
            chapter_title_selector: "text".into(),
            content_selector: "#c".into(),
            next_page_selector: None,
            encoding: Some("gbk".into()),
            ad_keywords: None,
            clean_patterns: None,
            cookies: Some("site=abc".into()),
            request_interval_ms: Some(2000),
        };

        let opts = FetchOptions::from_source(&source);
        assert_eq!(opts.cookies.as_deref(), Some("site=abc"));
        assert_eq!(opts.encoding.as_deref(), Some("gbk"));
        assert_eq!(opts.min_interval_ms, Some(2000));
    }

    #[test]
    fn utf8_decode_works() {
        let bytes = b"<html><body>hello</body></html>";
        let (text, _, err) = UTF_8.decode(bytes);
        assert!(!err);
        assert!(text.contains("hello"));
    }
}
