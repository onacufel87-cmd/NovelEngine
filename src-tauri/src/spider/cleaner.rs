use std::sync::OnceLock;

use regex::{Regex, RegexBuilder};
use serde::Deserialize;

use super::rule::BookSource;
use crate::storage::settings::get_setting;

/// 预编译的默认正则（只初始化一次，避免重复编译开销）
static DEFAULT_REGEXES: OnceLock<Vec<Regex>> = OnceLock::new();

/// 内置默认清洗规则
fn default_pattern_strs() -> &'static [&'static str] {
    &[
        r"【[^】]*】",                              // 【分页标记】等
        r"（本章未完[^）]*）|（请记住[^）]*）|（请收藏[^）]*）", // 常见广告括号句（比全量括号更保守）
        r"本章未完，请点击下一页",                    // 精确匹配
        r"请记住本书首发域名：\S+",
        r"请记住本站网址[^\n]*",
        r"https?://[a-zA-Z0-9./?=_-]+",             // 完整 URL
        r"\.?[a-zA-Z0-9-]+\.(com|cn|net|org|cc|vip)", // 常见域名
        r"更新最快|最快更新|百度搜索|手机用户请浏览",
        r"第\d+页",
        r"^\s*\d+\s*$",                             // 单独一行的页码（多行模式）
        // 维基文库 / 公版站常见噪音
        r"Public\s*domain(?:Public\s*domain)?(?:true|false)*",
        r"上一回[\s　]*回目录[\s　]*下一回",
        r"^上一回$|^下一回$|^回目录$",
        r"此(?:清|明|民)?朝?作品在[^。]*公有领域[^。]*。",
        r"检索自「https?://[^」]+」",
        r"姊妹计划[^：]*：[^。]*。",
    ]
}

/// 获取预编译的默认正则列表
fn cached_default_regexes() -> &'static Vec<Regex> {
    DEFAULT_REGEXES.get_or_init(|| {
        default_pattern_strs()
            .iter()
            .filter_map(|pattern| compile_pattern(pattern).ok())
            .collect()
    })
}

/// 编译单条正则（页码规则需要 multiline）
fn compile_pattern(pattern: &str) -> Result<Regex, regex::Error> {
    if pattern.starts_with("^") {
        RegexBuilder::new(pattern)
            .multi_line(true)
            .build()
    } else {
        Regex::new(pattern)
    }
}

/// 清洗正文：移除广告与分页噪音，压缩多余空行
///
/// # 参数
/// - `raw`：原始正文文本
/// - `extra_patterns`：书源或调用方传入的额外正则规则
pub fn clean_content(raw: &str, extra_patterns: Option<Vec<&str>>) -> String {
    let mut text = raw.to_string();

    // 1. 应用预编译的默认规则
    apply_regexes(&mut text, cached_default_regexes());

    // 2. 应用额外正则（无效规则跳过并打印日志）
    if let Some(extra) = extra_patterns {
        for pattern in extra {
            match compile_pattern(pattern) {
                Ok(re) => {
                    text = re.replace_all(&text, "").to_string();
                }
                Err(e) => {
                    eprintln!("无效清洗正则，已跳过: {pattern} ({e})");
                }
            }
        }
    }

    // 3. 按行剔除典型页脚/导航噪音
    text = strip_noise_lines(&text);

    // 4. 截断末尾连续噪音段落
    text = trim_trailing_noise(&text);

    // 5. 智能段落重排（合并恶意断行）
    text = reflow_paragraphs(&text);

    // 6. 压缩连续三个及以上换行
    compress_blank_lines(&mut text);

    text.trim().to_string()
}

/// 结合书源规则清洗正文（`clean_patterns` + `ad_keywords`）
pub fn clean_content_for_source(raw: &str, source: Option<&BookSource>) -> String {
    let extra: Option<Vec<&str>> = source.and_then(|s| {
        s.clean_patterns
            .as_ref()
            .map(|patterns| patterns.iter().map(String::as_str).collect())
    });

    let mut text = clean_content(raw, extra);

    // 字面量关键词替换（简单广告短语）
    if let Some(src) = source {
        if let Some(keywords) = &src.ad_keywords {
            for kw in keywords {
                text = text.replace(kw, "");
            }
            compress_blank_lines(&mut text);
            text = text.trim().to_string();
        }
    }

    text
}

/// 读取用户全局正文清洗配置（来自 reader_settings JSON）
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GlobalCleanSettings {
    #[serde(default)]
    global_ad_keywords: String,
    #[serde(default)]
    global_clean_patterns: String,
    #[serde(default)]
    chinese_variant: String,
    /// 是否还原 censored 拼音占位符（yindao → 阴道）
    #[serde(default = "default_restore_pinyin")]
    restore_censored_pinyin: bool,
    /// 自定义拼音映射，每行一条（yindao=阴道）
    #[serde(default)]
    custom_pinyin_mappings: String,
}

impl Default for GlobalCleanSettings {
    fn default() -> Self {
        Self {
            global_ad_keywords: String::new(),
            global_clean_patterns: String::new(),
            chinese_variant: String::new(),
            restore_censored_pinyin: true,
            custom_pinyin_mappings: String::new(),
        }
    }
}

fn default_restore_pinyin() -> bool {
    true
}

fn load_global_clean_settings() -> GlobalCleanSettings {
    let Ok(Some(json)) = get_setting("reader_settings") else {
        return GlobalCleanSettings::default();
    };
    serde_json::from_str(&json).unwrap_or_default()
}

fn split_setting_lines(text: &str) -> Vec<String> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(String::from)
        .collect()
}

/// 应用用户全局正文清洗（对所有书源、含已缓存章节生效）
pub fn apply_global_clean(text: &str) -> String {
    let settings = load_global_clean_settings();
    let patterns = split_setting_lines(&settings.global_clean_patterns);
    let pattern_refs: Vec<&str> = patterns.iter().map(String::as_str).collect();
    let extra = if pattern_refs.is_empty() {
        None
    } else {
        Some(pattern_refs)
    };

    // 始终走默认清洗 + 用户规则 + 行级过滤
    let mut out = clean_content(text, extra);

    for kw in split_setting_lines(&settings.global_ad_keywords) {
        out = out.replace(&kw, "");
    }

    compress_blank_lines(&mut out);

    // censored 拼音占位符还原（在简繁转换前，避免干扰 OpenCC）
    if settings.restore_censored_pinyin {
        let overrides =
            super::pinyin_restore::PinyinOverrides::from_text(&settings.custom_pinyin_mappings);
        out = super::pinyin_restore::restore_pinyin_with_overrides(&out, &overrides);
    }

    out = super::text_conv::apply_chinese_variant(&out, &settings.chinese_variant);
    out.trim().to_string()
}

/// 批量应用正则替换
fn apply_regexes(text: &mut String, regexes: &[Regex]) {
    for re in regexes {
        *text = re.replace_all(text.as_str(), "").to_string();
    }
}

/// 将连续三个及以上换行压缩为两个
fn compress_blank_lines(text: &mut String) {
    if let Ok(re) = Regex::new(r"\n\s*\n\s*\n+") {
        *text = re.replace_all(text.as_str(), "\n\n").to_string();
    }
    // 去掉仅含空白的行
    *text = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n");
}

/// 判断是否为应剔除的噪音行（维基导航、版权模板等）
fn is_noise_line(line: &str) -> bool {
    let t = line.trim();
    if t.is_empty() {
        return true;
    }
    if t.contains("Public domain") || t.contains("falsefalse") {
        return true;
    }
    if t.contains("上一回") && t.contains("下一回") {
        return true;
    }
    if t == "回目录" || t == "上一回" || t == "下一回" {
        return true;
    }
    if t.contains("公有领域") && (t.contains("作者逝世") || t.contains("1931年") || t.contains("在全世界")) {
        return true;
    }
    if t.starts_with("检索自") || t.starts_with("分类：") || t.starts_with("姊妹计划") {
        return true;
    }
    if t.contains("维基文库") && t.len() < 80 {
        return true;
    }
    false
}

/// 过滤正文中的噪音行
fn strip_noise_lines(text: &str) -> String {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !is_noise_line(line))
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// 从末尾向前剔除连续噪音行（章节导航、PD 声明等）
fn trim_trailing_noise(text: &str) -> String {
    let mut lines: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();

    while lines.last().is_some_and(|l| is_noise_line(l)) {
        lines.pop();
    }

    lines.join("\n\n")
}

/// 智能段落重排：合并被恶意切碎的断行，接近出版级排版
pub fn reflow_paragraphs(text: &str) -> String {
    let blocks: Vec<&str> = text.split("\n\n").collect();
    let mut out_blocks = Vec::new();

    for block in blocks {
        let trimmed = block.trim();
        if trimmed.is_empty() {
            continue;
        }
        let lines: Vec<&str> = trimmed.lines().map(str::trim).filter(|l| !l.is_empty()).collect();
        if lines.is_empty() {
            continue;
        }
        if lines.len() == 1 {
            out_blocks.push(lines[0].to_string());
            continue;
        }

        let mut paragraph = String::new();
        for line in lines {
            if paragraph.is_empty() {
                paragraph.push_str(line);
                continue;
            }
            if should_merge_lines(&paragraph, line) {
                // 中文直接拼接，英文加空格
                if paragraph.chars().last().is_some_and(|c| c.is_ascii_alphabetic())
                    && line.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
                {
                    paragraph.push(' ');
                }
                paragraph.push_str(line);
            } else {
                out_blocks.push(paragraph);
                paragraph = line.to_string();
            }
        }
        if !paragraph.is_empty() {
            out_blocks.push(paragraph);
        }
    }

    out_blocks.join("\n\n")
}

/// 判断两行是否应合并为同一段
fn should_merge_lines(prev: &str, next: &str) -> bool {
    if is_chapter_heading(next) || is_chapter_heading(prev) {
        return false;
    }
    let prev_end = prev.chars().last().unwrap_or(' ');
    let sentence_end = ['。', '！', '？', '…', '」', '』', '"', '"', '.', '!', '?'];
    if sentence_end.contains(&prev_end) {
        return false;
    }
    // 上一行较短且未句号结束 → 疑似被截断
    let prev_len = prev.chars().count();
    let next_len = next.chars().count();
    if prev_len < 80 || next_len < 120 {
        return true;
    }
    false
}

fn is_chapter_heading(line: &str) -> bool {
    let t = line.trim();
    if t.len() > 40 {
        return false;
    }
    t.starts_with("第") && (t.contains('章') || t.contains('节') || t.contains('回'))
        || t.starts_with("Chapter ")
        || t.starts_with("CHAPTER ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reflow_merges_broken_lines() {
        let raw = "他说着话，突然\n停下了脚步。\n\n天空下起了小雨。";
        let out = reflow_paragraphs(raw);
        assert!(out.contains("他说着话，突然停下了脚步。"));
        assert!(out.contains("天空下起了小雨。"));
    }

    #[test]
    fn removes_bracket_page_marker() {
        let raw = "第一段。\n\n【第二页的正文内容。】\n\n第二段。";
        let cleaned = clean_content(raw, None);
        assert!(!cleaned.contains("第二页的正文内容"));
        assert!(cleaned.contains("第一段"));
        assert!(cleaned.contains("第二段"));
    }

    #[test]
    fn removes_chapter_continue_paren() {
        let raw = "段落一\n\n（本章未完，请点击下一页）\n\n段落二";
        let cleaned = clean_content(raw, None);
        assert!(!cleaned.contains("本章未完"));
        assert!(cleaned.contains("段落一"));
        assert!(cleaned.contains("段落二"));
    }

    #[test]
    fn removes_domain() {
        let raw = "正文开始。www.biquge.com 提供阅读。正文结束。";
        let cleaned = clean_content(raw, None);
        assert!(!cleaned.contains("biquge.com"));
        assert!(cleaned.contains("正文开始"));
        assert!(cleaned.contains("正文结束"));
    }

    #[test]
    fn preserves_normal_punctuation_and_structure() {
        let raw = "他说：「你好。」\n\n她点了点头，没有说话。\n\n天空下起了雨。";
        let cleaned = clean_content(raw, None);
        assert!(cleaned.contains("。"));
        assert!(cleaned.contains("你好"));
        assert!(cleaned.contains("天空下起了雨"));
        assert!(cleaned.contains("\n\n"));
    }

    #[test]
    fn applies_extra_patterns() {
        let raw = "正文XYZABC结尾";
        let cleaned = clean_content(raw, Some(vec![r"XYZABC"]));
        assert!(!cleaned.contains("XYZABC"));
        assert!(cleaned.contains("正文"));
        assert!(cleaned.contains("结尾"));
    }

    #[test]
    fn removes_wikisource_footer_noise() {
        let raw = "…要知是谁，且听下回分解。\n\n上一回 回目录 下一回\n\n此清朝作品在全世界都属于公有领域，因为作者逝世已经超过100年，且作品于1931年1月1日之前出版。\n\nPublic domainPublic domainfalsefalse";
        let cleaned = clean_content(raw, None);
        assert!(cleaned.contains("要知是谁"));
        assert!(!cleaned.contains("上一回"));
        assert!(!cleaned.contains("公有领域"));
        assert!(!cleaned.contains("Public domain"));
        assert!(!cleaned.contains("falsefalse"));
    }

    #[test]
    fn clean_patterns_from_source() {
        let source = BookSource {
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
            chapter_list_selector: String::new(),
            chapter_title_selector: String::new(),
            content_selector: "#c".into(),
            next_page_selector: None,
            encoding: None,
            ad_keywords: Some(vec!["广告词".into()]),
            clean_patterns: Some(vec![r"CUSTOM\d+".into()]),
            cookies: None,
            request_interval_ms: None,
        };
        let raw = "正文CUSTOM123广告词结束";
        let cleaned = clean_content_for_source(raw, Some(&source));
        assert!(!cleaned.contains("CUSTOM123"));
        assert!(!cleaned.contains("广告词"));
        assert!(cleaned.contains("正文"));
        assert!(cleaned.contains("结束"));
    }

    #[test]
    fn global_clean_restores_censored_pinyin() {
        let raw = "正文 yindao 测试。";
        let cleaned = clean_content(raw, None);
        let out = apply_global_clean(&cleaned);
        assert!(out.contains("阴道"));
        assert!(!out.contains("yindao"));
    }
}
