//! censored 拼音占位符还原（笔趣阁类源常见 yindao、yin 等）
//!
//! 策略：
//! 1. 多音节词组优先查反向词典（长键优先，支持 yin_dao 连字符写法）
//! 2. 高歧义音节仅走上下文模板（如 yin，避免「阴符」误伤）
//! 3. 低歧义音节：上下文或「中文夹拼音」时还原
//! 4. 英文白名单保护真实英文词（gun、chaos、Chapter 等）

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use regex::Regex;

static DICT_ENTRIES: OnceLock<Vec<(&'static str, &'static str)>> = OnceLock::new();

/// 仅上下文模板才替换，禁止「中文夹拼音」盲替
static CONTEXT_ONLY: OnceLock<HashSet<&'static str>> = OnceLock::new();

/// 可上下文或中文夹拼音盲替
static SANDWICH_OK: OnceLock<HashSet<&'static str>> = OnceLock::new();

static PINYIN_TOKEN_RE: OnceLock<Regex> = OnceLock::new();
static SPACED_PINYIN_RE: OnceLock<Regex> = OnceLock::new();
static SPACED_TRIPLE_PINYIN_RE: OnceLock<Regex> = OnceLock::new();
static CJK_SPACE_RE: OnceLock<Regex> = OnceLock::new();

/// 用户自定义拼音映射（设置页「自定义拼音词典」）
#[derive(Debug, Default, Clone)]
pub struct PinyinOverrides {
    entries: HashMap<String, String>,
}

impl PinyinOverrides {
    pub fn from_text(text: &str) -> Self {
        let mut entries = HashMap::new();
        for line in text.lines() {
            if let Some((key, hanzi)) = parse_override_line(line) {
                entries.insert(key, hanzi);
            }
        }
        Self { entries }
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries.get(key).map(String::as_str)
    }

    pub fn has(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// 解析单行自定义映射：`yindao=阴道`、`yindao:阴道`、`yindao 阴道`
fn parse_override_line(line: &str) -> Option<(String, String)> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') || line.starts_with("//") {
        return None;
    }

    for sep in ['=', ':', '→', '＝', '：'] {
        if let Some((raw_key, raw_val)) = line.split_once(sep) {
            let key = normalize_key(raw_key.trim());
            let val = raw_val.trim().to_string();
            if !key.is_empty() && !val.is_empty() {
                return Some((key, val));
            }
        }
    }

    // 首个空白分隔：`yindao 阴道`
    if let Some((raw_key, raw_val)) = line.split_once(|c: char| c.is_whitespace()) {
        let key = normalize_key(raw_key.trim());
        let val = raw_val.trim().to_string();
        if !key.is_empty() && !val.is_empty() {
            return Some((key, val));
        }
    }

    None
}

/// (前文后缀, 拼音, 后文前缀, 还原字)
static CONTEXT_RULES: &[(&str, &str, &str, &str)] = &[
    ("一片", "yin", "暗", "阴"),
    ("", "yin", "道", "阴"),
    ("", "yin", "茎", "阴"),
    ("", "yin", "部", "阴"),
    ("", "yin", "毛", "阴"),
    ("", "yin", "唇", "阴"),
    ("", "yin", "户", "阴"),
    ("", "yin", "蒂", "阴"),
    ("", "yin", "精", "阴"),
    ("", "yin", "水", "阴"),
    ("", "yin", "影", "阴"),
    ("", "yin", "核", "阴"),
    ("", "yin", "囊", "阴"),
    ("", "yin", "唇", "阴"),
    ("", "yin", "户", "阴"),
    ("", "yin", "魔", "阴"),
    ("", "yin", "阳", "阴"),
    ("", "yin", "沉", "阴"),
    ("", "yin", "谋", "阴"),
    ("", "yin", "险", "阴"),
    ("", "yin", "私", "阴"),
    ("", "yin", "茎", "阴"),
    ("", "yin", "毛", "阴"),
    ("淫", "dang", "", "荡"),
    ("", "dang", "妇", "荡"),
    ("", "dang", "妇", "荡"),
    ("发", "qing", "", "情"),
    ("", "qing", "欲", "情"),
    ("性", "yu", "", "欲"),
    ("", "yu", "望", "欲"),
    ("", "yu", "火", "欲"),
    ("", "yu", "望", "欲"),
    ("天", "chao", "", "朝"),
    ("", "fu", "bai", "腐"),
    ("", "fu", "败", "腐"),
    ("腐", "bai", "", "败"),
    ("", "shen", "yin", "呻"),
    ("", "shen", "吟", "呻"),
    ("娇", "chuan", "", "喘"),
    ("", "chuan", "来", "喘"),
    ("", "chuan", "息", "喘"),
    ("", "chuan", "气", "喘"),
    ("", "cao", "你", "操"),
    ("", "cao", "他", "操"),
    ("", "cao", "她", "操"),
    ("", "cao", "妈", "操"),
    ("", "ri", "你", "日"),
    ("", "ri", "他", "日"),
    ("", "ri", "死", "日"),
    ("", "shuang", "了", "爽"),
    ("", "shuang", "快", "爽"),
    ("高", "chao", "", "潮"),
    ("", "chao", "喷", "潮"),
    ("", "chao", "吹", "潮"),
    ("", "she", "精", "射"),
    ("", "she", "了", "射"),
    ("", "she", "进", "射"),
    ("", "lu", "管", "撸"),
    ("", "lu", "了", "撸"),
    ("", "bi", "紧", "逼"),
    ("", "bi", "里", "逼"),
    ("", "bi", "穴", "逼"),
    ("小", "bi", "", "逼"),
    ("干", "ni", "妈", "你"),
    ("", "gan", "他", "干"),
    ("", "gan", "她", "干"),
    ("", "gan", "死", "干"),
    ("", "ji", "巴", "鸡"),
    ("", "ji", "八", "鸡"),
    ("", "xing", "欲", "性"),
    ("", "xing", "交", "性"),
    ("", "xing", "爱", "性"),
    ("", "xing", "感", "性"),
    ("", "jiao", "床", "叫"),
    ("", "jiao", "声", "叫"),
    ("", "se", "情", "色"),
    ("", "se", "狼", "色"),
    ("", "luo", "体", "裸"),
    ("", "luo", "露", "裸"),
    ("", "ru", "房", "乳"),
    ("", "ru", "头", "乳"),
    ("", "tun", "部", "臀"),
    ("", "pi", "股", "屁"),
    ("", "gong", "颈", "宫"),
    ("", "gong", "口", "宫"),
    ("", "xue", "道", "穴"),
    ("", "xue", "口", "穴"),
    ("", "hui", "秽", "淫"),
    ("淫", "hui", "", "秽"),
    ("", "jian", "货", "贱"),
    ("", "jian", "人", "贱"),
    ("", "sa", "货", "骚"),
    ("", "sa", "比", "骚"),
    ("", "luan", "伦", "乱"),
    ("", "lun", "奸", "乱"),
];

fn dict_entries() -> &'static Vec<(&'static str, &'static str)> {
    DICT_ENTRIES.get_or_init(|| {
        let mut entries: Vec<(&str, &str)> = vec![
            // ── 多音节 censored 词（高置信，直接替换）──
            ("yindao", "阴道"),
            ("yinjing", "阴茎"),
            ("yinchun", "阴唇"),
            ("yindi", "阴蒂"),
            ("yindang", "淫荡"),
            ("yinjiao", "淫叫"),
            ("yinhui", "淫秽"),
            ("yinshui", "淫水"),
            ("yinmao", "阴毛"),
            ("yinbu", "阴部"),
            ("yinnang", "阴囊"),
            ("yinjing", "阴茎"),
            ("jiaochuan", "娇喘"),
            ("shenyin", "呻吟"),
            ("caonima", "草泥马"),
            ("nimabi", "你妈逼"),
            ("nmlgb", "你妈了个逼"),
            ("wqnmlgb", "我去你妈了个逼"),
            ("shabi", "傻逼"),
            ("shabi", "傻逼"),
            ("tianchao", "天朝"),
            ("fubai", "腐败"),
            ("gaochao", "高潮"),
            ("jingye", "精液"),
            ("jingzi", "精子"),
            ("shejing", "射精"),
            ("koujiao", "口交"),
            ("koubao", "口爆"),
            ("zuoai", "做爱"),
            ("xingai", "性爱"),
            ("xingjiaochuan", "性娇喘"),
            ("seqing", "色情"),
            ("maiyin", "卖淫"),
            ("maibi", "卖逼"),
            ("luoti", "裸体"),
            ("luolou", "裸露"),
            ("luoguang", "裸光"),
            ("shenhou", "深喉"),
            ("zhuru", "插入"),
            ("choucha", "抽插"),
            ("neishe", "内射"),
            ("wutao", "无套"),
            ("youhuo", "诱惑"),
            ("faqing", "发情"),
            ("xingyu", "性欲"),
            ("xingfen", "性奋"),
            ("chaochui", "潮吹"),
            ("rufang", "乳房"),
            ("rutou", "乳头"),
            ("tunbu", "臀部"),
            ("piigu", "屁股"),
            ("dangfu", "荡妇"),
            ("saohuo", "骚货"),
            ("saobi", "骚逼"),
            ("jianren", "贱人"),
            ("jianhuo", "贱货"),
            ("biaozi", "婊子"),
            ("chunv", "处女"),
            ("luanlun", "乱伦"),
            ("maiyin", "卖淫"),
            ("yinluan", "淫乱"),
            ("yinhui", "淫秽"),
            ("shenhou", "深喉"),
            ("wenxiong", "吻胸"),
            ("moixiong", "摸胸"),
            ("moxiong", "摸胸"),
            ("taotao", "套套"),
            ("biyun", "避孕"),
            ("qingmi", "亲密"),
            ("qinwen", "亲吻"),
            ("fajiao", "发娇"),
            // ── 短音节（需歧义分级）──
            ("shuang", "爽"),
            ("chao", "潮"),
            ("she", "射"),
            ("lu", "撸"),
            ("gan", "干"),
            ("ri", "日"),
            ("cao", "操"),
            ("bi", "逼"),
            ("ji", "鸡"),
            ("yin", "阴"),
            ("dang", "荡"),
            ("qing", "情"),
            ("yu", "欲"),
            ("xing", "性"),
            ("jiao", "叫"),
            ("chuan", "喘"),
            ("shen", "呻"),
            ("fu", "腐"),
            ("bai", "败"),
            ("an", "暗"),
            ("luo", "裸"),
            ("ru", "乳"),
            ("tun", "臀"),
            ("pi", "屁"),
            ("gong", "宫"),
            ("jing", "精"),
            ("ye", "液"),
            ("zi", "子"),
            ("xue", "穴"),
            ("chun", "唇"),
            ("hu", "户"),
            ("mao", "毛"),
            ("shui", "水"),
            ("fen", "奋"),
            ("mi", "迷"),
            ("luan", "乱"),
            ("lun", "伦"),
            ("jian", "贱"),
            ("wu", "污"),
            ("hui", "秽"),
            ("se", "色"),
            ("sa", "骚"),
            ("biao", "婊"),
        ];
        entries.sort_by_key(|(k, _)| std::cmp::Reverse(k.len()));
        entries.dedup_by_key(|(k, _)| *k);
        entries
    })
}

fn context_only() -> &'static HashSet<&'static str> {
    CONTEXT_ONLY.get_or_init(|| {
        [
            "yin", "qing", "yu", "xing", "shen", "dang", "an", "fu", "bai", "chao", "se", "jiao",
            "chuan", "hui", "wu", "luan", "lun", "fen", "mi", "shui", "mao", "hu", "chun", "ye",
            "zi", "jing", "gong", "tun", "ru", "luo", "pi", "biao", "sa", "jian",
        ]
        .into_iter()
        .collect()
    })
}

fn sandwich_ok() -> &'static HashSet<&'static str> {
    SANDWICH_OK.get_or_init(|| {
        ["bi", "cao", "ri", "lu", "gan", "ji", "mo", "she", "shuang"]
            .into_iter()
            .collect()
    })
}

fn token_re() -> &'static Regex {
    PINYIN_TOKEN_RE.get_or_init(|| {
        // 支持 yin_dao、yin-dao 连写
        Regex::new(r"[a-zA-Z]+(?:[-_][a-zA-Z]+)*").expect("pinyin token regex")
    })
}

fn collapse_spaced_cjk(text: &str) -> String {
    let re = CJK_SPACE_RE.get_or_init(|| {
        Regex::new(r"([\u{4e00}-\u{9fff}])\s+([\u{4e00}-\u{9fff}])").expect("cjk space regex")
    });
    let mut out = text.to_string();
    loop {
        let next = re.replace_all(&out, "$1$2").to_string();
        if next == out {
            break;
        }
        out = next;
    }
    out
}

/// 合并「shen yin」「yin dao」「cao ni ma」等空格分写的多音节拼音
fn restore_spaced_compounds(text: &str, overrides: &PinyinOverrides) -> String {
    let re2 = SPACED_PINYIN_RE.get_or_init(|| {
        Regex::new(r"(?i)([a-z]+(?:[-_][a-z]+)*)\s+([a-z]+(?:[-_][a-z]+)*)")
            .expect("spaced pinyin regex")
    });
    let re3 = SPACED_TRIPLE_PINYIN_RE.get_or_init(|| {
        Regex::new(r"(?i)([a-z]+(?:[-_][a-z]+)*)\s+([a-z]+(?:[-_][a-z]+)*)\s+([a-z]+(?:[-_][a-z]+)*)")
            .expect("spaced triple pinyin regex")
    });

    let mut out = text.to_string();
    loop {
        let mut changed = false;

        let triple = re3
            .replace_all(&out, |caps: &regex::Captures| {
                let combined = format!(
                    "{}{}{}",
                    normalize_key(&caps[1]),
                    normalize_key(&caps[2]),
                    normalize_key(&caps[3])
                );
                if combined.len() >= 4 {
                    if let Some(hanzi) = lookup_exact(&combined, overrides) {
                        return hanzi.to_string();
                    }
                }
                caps[0].to_string()
            })
            .into_owned();
        if triple != out {
            out = triple;
            changed = true;
        }

        let pair = re2
            .replace_all(&out, |caps: &regex::Captures| {
                let combined = format!(
                    "{}{}",
                    normalize_key(&caps[1]),
                    normalize_key(&caps[2])
                );
                if combined.len() >= 4 {
                    if let Some(hanzi) = lookup_exact(&combined, overrides) {
                        return hanzi.to_string();
                    }
                }
                caps[0].to_string()
            })
            .into_owned();
        if pair != out {
            out = pair;
            changed = true;
        }

        if !changed {
            break;
        }
    }
    out
}

/// 还原正文中的 censored 拼音占位符
pub fn restore_pinyin(text: &str) -> String {
    restore_pinyin_with_overrides(text, &PinyinOverrides::default())
}

/// 带用户自定义映射的拼音还原
pub fn restore_pinyin_with_overrides(text: &str, overrides: &PinyinOverrides) -> String {
    let merged = restore_spaced_compounds(text, overrides);
    let replaced = token_re()
        .replace_all(&merged, |caps: &regex::Captures| {
            let matched = &caps[0];
            let start = caps.get(0).map(|m| m.start()).unwrap_or(0);
            let end = caps.get(0).map(|m| m.end()).unwrap_or(start);
            resolve_token(matched, &merged, start, end, overrides)
                .unwrap_or_else(|| matched.to_string())
        })
        .into_owned();
    collapse_spaced_cjk(&replaced)
}

/// 查词典用的规范键：去连字符、转小写
fn normalize_key(raw: &str) -> String {
    raw.to_lowercase().replace(['-', '_'], "")
}

fn resolve_token(
    matched: &str,
    full: &str,
    start: usize,
    end: usize,
    overrides: &PinyinOverrides,
) -> Option<String> {
    if matched.is_empty() {
        return None;
    }

    // CamelCase 品牌名保留（iPhone）；全大写短 token 仍尝试还原（YIN→阴 的误伤较少）
    if matched.chars().any(|c| c.is_ascii_uppercase())
        && matched.chars().any(|c| c.is_ascii_lowercase())
    {
        return None;
    }

    let lower = normalize_key(matched);

    // 用户自定义优先：强制直接替换，跳过英文白名单与歧义分级
    if let Some(hanzi) = overrides.get(&lower) {
        return Some(hanzi.to_string());
    }

    if lower.len() > 14 && !has_exact(&lower, overrides) {
        return None;
    }

    if is_likely_english_word(&lower, full, start, end, overrides) {
        return None;
    }

    if let Some(hanzi) = lookup_exact(&lower, overrides) {
        return resolve_with_tier(&lower, hanzi, full, start, end);
    }

    context_resolve(&lower, full, start, end).map(String::from)
}

fn resolve_with_tier(
    key: &str,
    hanzi: &str,
    full: &str,
    start: usize,
    end: usize,
) -> Option<String> {
    // 上下文模板优先（含 chao 等 4 字母歧义音节）
    if let Some(ctx) = context_resolve(key, full, start, end) {
        return Some(ctx.to_string());
    }

    // 多音节词组（≥4 字母）且非歧义表 → 直接替换
    if key.len() >= 4 && !context_only().contains(key) && !sandwich_ok().contains(key) {
        return Some(hanzi.to_string());
    }

    if context_only().contains(key) {
        return None;
    }

    if sandwich_ok().contains(key) && is_sandwiched_by_cjk(full, start, end) {
        return Some(hanzi.to_string());
    }

    // 3 字母非歧义表项
    if key.len() == 3 && !context_only().contains(key) && !sandwich_ok().contains(key) {
        if is_sandwiched_by_cjk(full, start, end) || has_cjk_neighbor(full, start, end) {
            return Some(hanzi.to_string());
        }
    }

    None
}

fn has_cjk_neighbor(full: &str, start: usize, end: usize) -> bool {
    nearest_significant_char_before(full, start).is_some_and(is_cjk)
        || nearest_significant_char_after(full, end).is_some_and(is_cjk)
}

fn nearest_significant_char_before(full: &str, start: usize) -> Option<char> {
    full[..start]
        .chars()
        .rev()
        .find(|c| !c.is_whitespace())
}

fn nearest_significant_char_after(full: &str, end: usize) -> Option<char> {
    full[end..].chars().find(|c| !c.is_whitespace())
}

fn is_sandwiched_by_cjk(full: &str, start: usize, end: usize) -> bool {
    nearest_significant_char_before(full, start).is_some_and(is_cjk)
        && nearest_significant_char_after(full, end).is_some_and(is_cjk)
}

fn dict_lookup_exact(key: &str) -> Option<&'static str> {
    dict_entries()
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, v)| *v)
}

fn dict_has_exact(key: &str) -> bool {
    dict_lookup_exact(key).is_some()
}

/// 先查用户自定义，再查内置词典
fn lookup_exact<'a>(key: &str, overrides: &'a PinyinOverrides) -> Option<&'a str> {
    if let Some(custom) = overrides.get(key) {
        return Some(custom);
    }
    dict_lookup_exact(key)
}

fn has_exact(key: &str, overrides: &PinyinOverrides) -> bool {
    overrides.has(key) || dict_has_exact(key)
}

fn context_resolve(pinyin: &str, full: &str, start: usize, end: usize) -> Option<&'static str> {
    let left = compact_context(&full[..start]);
    let right = compact_context(&full[end..]);

    for (left_suffix, rule_py, right_prefix, hanzi) in CONTEXT_RULES {
        if *rule_py != pinyin {
            continue;
        }
        let left_ok = left_suffix.is_empty() || left.ends_with(left_suffix);
        let right_ok = right_prefix.is_empty() || right.starts_with(right_prefix);
        if left_ok && right_ok {
            return Some(hanzi);
        }
    }
    None
}

fn compact_context(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

fn is_between_cjk(full: &str, start: usize, end: usize) -> bool {
    is_sandwiched_by_cjk(full, start, end)
}

fn is_cjk(c: char) -> bool {
    matches!(
        c,
        '\u{4e00}'..='\u{9fff}' | '\u{3400}'..='\u{4dbf}' | '\u{3000}'..='\u{303f}'
    )
}

fn is_likely_english_word(
    lower: &str,
    full: &str,
    start: usize,
    end: usize,
    overrides: &PinyinOverrides,
) -> bool {
    // 紧邻中文的 token 按拼音处理，不按英文白名单放行
    if has_cjk_neighbor(full, start, end) {
        return false;
    }

    const ENGLISH_WHITELIST: &[&str] = &[
        "the", "and", "for", "that", "with", "from", "this", "chapter", "http", "https", "www",
        "com", "html", "org", "net", "true", "false", "null", "class", "style", "div", "span",
        "alice", "pride", "love", "god", "lord", "king", "queen", "sir", "said", "she", "her",
        "him", "his", "was", "were", "have", "has", "had", "not", "but", "you", "your", "they",
        "them", "what", "when", "where", "which", "who", "will", "would", "could", "should",
        "public", "domain", "copyright", "gutenberg", "part", "book", "page", "gun", "guns",
        "chaos", "machine", "python", "github", "google", "english", "symbol", "important",
        "unit", "union", "unique", "video", "audio", "image", "file", "data", "info", "mail",
        "email", "login", "admin", "user", "test", "demo", "api", "url", "link", "menu",
        "home", "index", "next", "prev", "back", "read", "more", "view", "list", "item",
        "name", "title", "author", "word", "words", "text", "line", "type", "code", "error",
        "yin", // 纯英文 yoga 语境 "yin symbol"（无中文邻居时才生效）
    ];

    if ENGLISH_WHITELIST.contains(&lower) {
        return true;
    }

    let prev_is_space =
        start == 0 || full[..start].chars().last().is_some_and(|c| c.is_whitespace());
    let next_is_space =
        end >= full.len() || full[end..].chars().next().is_some_and(|c| c.is_whitespace());

    // 纯英文句子中的 ASCII 词
    if lower.len() >= 4
        && prev_is_space
        && next_is_space
        && !has_exact(lower, overrides)
        && !has_cjk_neighbor(full, start, end)
    {
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 断言：还原后包含期望、不包含原始拼音 token
    fn assert_restore(input: &str, expect_contains: &str, must_not_contain: Option<&str>) {
        let out = restore_pinyin(input);
        assert!(
            out.contains(expect_contains),
            "输入: {input}\n输出: {out}\n期望包含: {expect_contains}"
        );
        if let Some(bad) = must_not_contain {
            assert!(
                !out.to_lowercase().contains(&bad.to_lowercase()),
                "输入: {input}\n输出: {out}\n不应包含: {bad}"
            );
        }
    }

    fn assert_preserve(input: &str, keep: &str) {
        let out = restore_pinyin(input);
        assert!(
            out.contains(keep),
            "输入: {input}\n输出: {out}\n应保留: {keep}"
        );
    }

    // ── 多音节词典 ──

    #[test]
    fn multi_yindao() {
        assert_restore("他感到一阵 yindao 收缩。", "阴道", Some("yindao"));
    }

    #[test]
    fn multi_yinjing() {
        assert_restore("yinjing 顶在入口。", "阴茎", Some("yinjing"));
    }

    #[test]
    fn multi_yinchun() {
        assert_restore("yinchun 微颤。", "阴唇", Some("yinchun"));
    }

    #[test]
    fn multi_jiaochuan() {
        assert_restore("她发出 jiaochuan 的声音。", "娇喘", Some("jiaochuan"));
    }

    #[test]
    fn multi_shenyin() {
        assert_restore("耳边传来 shenyin 声。", "呻吟", Some("shenyin"));
    }

    #[test]
    fn multi_caonima() {
        assert_restore("他骂了一句 caonima 就走了。", "草泥马", Some("caonima"));
    }

    #[test]
    fn multi_nimabi() {
        assert_restore("怒骂 nimabi 离去。", "你妈逼", Some("nimabi"));
    }

    #[test]
    fn multi_shabi() {
        assert_restore("这个 shabi 真烦。", "傻逼", Some("shabi"));
    }

    #[test]
    fn multi_gaochao() {
        assert_restore("身体 gaochao 了。", "高潮", Some("gaochao"));
    }

    #[test]
    fn multi_zuoai() {
        assert_restore("两人 zuoai 许久。", "做爱", Some("zuoai"));
    }

    #[test]
    fn multi_koujiao() {
        assert_restore("koujiao 细节省略。", "口交", Some("koujiao"));
    }

    #[test]
    fn multi_neishe() {
        assert_restore("最后 neishe 结束。", "内射", Some("neishe"));
    }

    #[test]
    fn multi_choucha() {
        assert_restore("疯狂 choucha 着。", "抽插", Some("choucha"));
    }

    #[test]
    fn multi_biaozi() {
        assert_restore("骂她 biaozi 不要脸。", "婊子", Some("biaozi"));
    }

    #[test]
    fn multi_saohuo() {
        assert_restore("真是个 saohuo 。", "骚货", Some("saohuo"));
    }

    #[test]
    fn multi_luoti() {
        assert_restore("luoti 走向浴室。", "裸体", Some("luoti"));
    }

    #[test]
    fn multi_tianchao() {
        assert_restore("tianchao 盛世。", "天朝", Some("tianchao"));
    }

    #[test]
    fn multi_fubai() {
        assert_restore("fubai 官员。", "腐败", Some("fubai"));
    }

    #[test]
    fn multi_yin_dao_hyphen() {
        assert_restore("进入 yin_dao 深处。", "阴道", Some("yin_dao"));
    }

    #[test]
    fn multi_yin_dao_dash() {
        assert_restore("yin-dao 收紧。", "阴道", Some("yin-dao"));
    }

    // ── 上下文歧义 yin ──

    #[test]
    fn ctx_yin_an() {
        assert_restore("树林里一片 yin 暗，气氛诡异。", "阴暗", Some("yin"));
    }

    #[test]
    fn ctx_yin_dao_spaced() {
        assert_restore("慢慢进入 yin 道。", "阴道", Some("yin"));
    }

    #[test]
    fn ctx_yin_mo() {
        assert_restore("修炼 yin 魔 功法。", "阴魔", Some("yin"));
    }

    #[test]
    fn reject_yin_fu_wrong_context() {
        let out = restore_pinyin("这个 yin 符 不对。");
        assert!(out.contains("yin"), "不应误替: {out}");
    }

    #[test]
    fn reject_yin_symbol_english() {
        assert_preserve("the yin symbol is important", "yin");
    }

    // ── 上下文脏话/动作 ──

    #[test]
    fn ctx_cao_ni() {
        assert_restore("开口就 cao 你 全家。", "操你", Some("cao"));
    }

    #[test]
    fn ctx_ri_ni() {
        assert_restore("再闹 ri 你 试试。", "日你", Some("ri"));
    }

    #[test]
    fn ctx_xiao_bi() {
        assert_restore("摸她小 bi 一下。", "小逼", Some("bi"));
    }

    #[test]
    fn ctx_gan_si() {
        assert_restore("gan 死 你。", "干死", Some("gan"));
    }

    #[test]
    fn ctx_ji_ba() {
        assert_restore("ji 巴 很大。", "鸡巴", Some("ji"));
    }

    #[test]
    fn ctx_xing_yu() {
        assert_restore("xing 欲 高涨。", "性欲", Some("xing"));
    }

    #[test]
    fn ctx_se_qing() {
        assert_restore("se 情 画面。", "色情", Some("se"));
    }

    #[test]
    fn ctx_luo_ti() {
        assert_restore("luo 体 展示。", "裸体", Some("luo"));
    }

    #[test]
    fn ctx_chao_pen() {
        assert_restore("chao 喷 而出。", "潮喷", Some("chao"));
    }

    #[test]
    fn ctx_shen_yin_split() {
        assert_restore("shen yin 不断。", "呻吟", None); // shen+yin 分写需 shenyin 或规则
    }

    // ── 英文保护 ──

    #[test]
    fn preserve_chapter() {
        assert_preserve("Chapter 1 开始了。", "Chapter");
    }

    #[test]
    fn preserve_gun_english() {
        assert_preserve("He picked up the gun.", "gun");
    }

    #[test]
    fn preserve_chaos_english() {
        assert_preserve("Order and chaos balance.", "chaos");
    }

    #[test]
    fn preserve_gutenberg() {
        assert_preserve("Project Gutenberg license.", "Gutenberg");
    }

    #[test]
    fn preserve_python() {
        assert_preserve("Learn python today.", "python");
    }

    #[test]
    fn preserve_github() {
        assert_preserve("See github for source.", "github");
    }

    #[test]
    fn preserve_she_said() {
        assert_preserve("she said nothing.", "she");
    }

    // ── 边界/混合 ──

    #[test]
    fn preserve_iphone() {
        assert_preserve("使用 iPhone 阅读。", "iPhone");
    }

    #[test]
    fn sandwich_bi_between_cjk() {
        assert_restore("好 bi 啊", "逼", Some("bi"));
    }

    #[test]
    fn multi_jingye() {
        assert_restore("jingye 流出。", "精液", Some("jingye"));
    }

    #[test]
    fn multi_yinhui() {
        assert_restore("yinhui 内容下架。", "淫秽", Some("yinhui"));
    }

    #[test]
    fn multi_luanlun() {
        assert_restore("luanlun 题材。", "乱伦", Some("luanlun"));
    }

    #[test]
    fn ctx_tian_chao() {
        assert_restore("天 chao 子民。", "天朝", Some("chao"));
    }

    #[test]
    fn ctx_fu_bai() {
        assert_restore("fu 败 分子。", "腐败", None);
    }

    #[test]
    fn no_false_on_url_like() {
        let out = restore_pinyin("访问 httpxxx 已过滤");
        assert!(!out.contains("阴道"));
    }

    #[test]
    fn collapse_spaces_after_restore() {
        let out = restore_pinyin("一片 yin 暗");
        assert_eq!(out, "一片阴暗");
    }

    // ── 空格分写多音节（restore_spaced_compounds）──

    #[test]
    fn spaced_yin_jing() {
        assert_restore("yin jing 顶在入口。", "阴茎", Some("yin"));
    }

    #[test]
    fn spaced_yin_dao() {
        assert_restore("进入 yin dao 深处。", "阴道", Some("yin"));
    }

    #[test]
    fn spaced_gao_chao() {
        assert_restore("身体 gao chao 了。", "高潮", Some("gao"));
    }

    #[test]
    fn spaced_zuo_ai() {
        assert_restore("两人 zuo ai 许久。", "做爱", Some("zuo"));
    }

    #[test]
    fn spaced_jiao_chuan() {
        assert_restore("发出 jiao chuan 声。", "娇喘", Some("jiao"));
    }

    #[test]
    fn spaced_fa_qing() {
        assert_restore("她 fa qing 了。", "发情", Some("fa"));
    }

    #[test]
    fn spaced_cao_ni_ma() {
        assert_restore("怒骂 cao ni ma。", "草泥马", Some("cao"));
    }

    // ── 更多上下文 yin / 其他歧义音节 ──

    #[test]
    fn ctx_yin_yang() {
        assert_restore("平衡 yin 阳 二气。", "阴阳", Some("yin"));
    }

    #[test]
    fn ctx_yin_si() {
        assert_restore("不可 yin 私 外传。", "阴私", Some("yin"));
    }

    #[test]
    fn ctx_yin_mou() {
        assert_restore("暗中 yin 谋 算计。", "阴谋", Some("yin"));
    }

    #[test]
    fn ctx_yin_jing_spaced() {
        assert_restore("yin 茎 顶在入口。", "阴茎", Some("yin"));
    }

    #[test]
    fn ctx_qing_yu() {
        assert_restore("qing 欲 高涨。", "情欲", Some("qing"));
    }

    #[test]
    fn ctx_xing_jiao() {
        assert_restore("xing 交 细节省略。", "性交", Some("xing"));
    }

    #[test]
    fn ctx_gao_chao_spaced() {
        assert_restore("高 chao 了。", "高潮", Some("chao"));
    }

    #[test]
    fn ctx_chao_chui() {
        assert_restore("chao 吹 而出。", "潮吹", Some("chao"));
    }

    #[test]
    fn ctx_luan_lun_spaced() {
        assert_restore("luan 伦 题材。", "乱伦", Some("luan"));
    }

    #[test]
    fn ctx_ru_fang() {
        assert_restore("ru 房 起伏。", "乳房", Some("ru"));
    }

    #[test]
    fn ctx_she_jing_with_cjk() {
        assert_restore("最后 she 精 结束。", "射精", Some("she"));
    }

    #[test]
    fn ctx_shuang_le() {
        assert_restore("shuang 了 一下。", "爽", Some("shuang"));
    }

    // ── 更多多音节词典 ──

    #[test]
    fn multi_chaochui() {
        assert_restore("chaochui 喷涌。", "潮吹", Some("chaochui"));
    }

    #[test]
    fn multi_rufang() {
        assert_restore("rufang 起伏。", "乳房", Some("rufang"));
    }

    #[test]
    fn multi_dangfu() {
        assert_restore("骂她 dangfu 不要脸。", "荡妇", Some("dangfu"));
    }

    #[test]
    fn multi_xingyu() {
        assert_restore("xingyu 高涨。", "性欲", Some("xingyu"));
    }

    #[test]
    fn multi_seqing() {
        assert_restore("seqing 画面。", "色情", Some("seqing"));
    }

    #[test]
    fn multi_zhuru() {
        assert_restore("zhuru 深处。", "插入", Some("zhuru"));
    }

    #[test]
    fn multi_wenxiong() {
        assert_restore("wenxiong 情节。", "吻胸", Some("wenxiong"));
    }

    // ── 英文/误伤保护 ──

    #[test]
    fn preserve_machine_english() {
        assert_preserve("machine learning is fun.", "machine");
    }

    #[test]
    fn preserve_order_and() {
        assert_preserve("Order and chaos balance.", "and");
    }

    #[test]
    fn preserve_chapter_with_pinyin() {
        let out = restore_pinyin("Chapter 1 里出现 yindao 描写。");
        assert!(out.contains("Chapter"));
        assert!(out.contains("阴道"));
        assert!(!out.contains("yindao"));
    }

    #[test]
    fn preserve_ascii_in_code() {
        assert_preserve("error code null pointer.", "null");
    }

    #[test]
    fn reject_yin_alone_no_context() {
        let out = restore_pinyin("这个 yin 不对。");
        assert!(out.contains("yin"), "无上下文时不应替换: {out}");
    }

    #[test]
    fn reject_qing_without_context() {
        let out = restore_pinyin("下载 qing 文件。");
        assert!(out.contains("qing"), "无上下文时不应替换: {out}");
    }

    // ── 混合/边界 ──

    #[test]
    fn mixed_cjk_pinyin_cjk_chain() {
        assert_restore("摸她小 bi 紧 了。", "小逼", Some("bi"));
    }

    #[test]
    fn multi_yinshui() {
        assert_restore("yinshui 横流。", "淫水", Some("yinshui"));
    }

    #[test]
    fn multi_yinmao() {
        assert_restore("yinmao 茂密。", "阴毛", Some("yinmao"));
    }

    #[test]
    fn ctx_cao_ta() {
        assert_restore("开口 cao 他 全家。", "操他", Some("cao"));
    }

    #[test]
    fn ctx_ri_si() {
        assert_restore("再闹 ri 死 你。", "日死", Some("ri"));
    }

    #[test]
    fn spaced_shen_yin_with_noise() {
        assert_restore("耳边 shen yin 不断。", "呻吟", Some("shen"));
    }

    #[test]
    fn hyphen_triple_syllable() {
        assert_restore("gao-chao 来了。", "高潮", Some("gao-chao"));
    }

    // ── 用户自定义映射（第三期）──

    fn assert_restore_custom(input: &str, mappings: &str, expect_contains: &str, bad: Option<&str>) {
        let overrides = PinyinOverrides::from_text(mappings);
        let out = restore_pinyin_with_overrides(input, &overrides);
        assert!(
            out.contains(expect_contains),
            "输入: {input}\n映射: {mappings}\n输出: {out}\n期望包含: {expect_contains}"
        );
        if let Some(token) = bad {
            assert!(
                !out.to_lowercase().contains(&token.to_lowercase()),
                "输入: {input}\n输出: {out}\n不应包含: {token}"
            );
        }
    }

    #[test]
    fn custom_new_site_word() {
        assert_restore_custom(
            "修炼 zhuanqi 法门。",
            "zhuanqi=转气",
            "转气",
            Some("zhuanqi"),
        );
    }

    #[test]
    fn custom_override_builtin_dict() {
        assert_restore_custom(
            "进入 yindao 深处。",
            "yindao=自定义词",
            "自定义词",
            Some("yindao"),
        );
    }

    #[test]
    fn custom_hyphen_and_colon_syntax() {
        assert_restore_custom(
            "yin-dao 收紧。",
            "yindao:自定义",
            "自定义",
            Some("yin-dao"),
        );
    }

    #[test]
    fn custom_spaced_compound() {
        assert_restore_custom(
            "site only 出现。",
            "siteonly=站点专有",
            "站点专有",
            Some("site"),
        );
    }

    #[test]
    fn custom_short_syllable_force() {
        assert_restore_custom(
            "这个 yin 符 不对。",
            "yin=阴",
            "阴符",
            Some("yin"),
        );
    }

    #[test]
    fn custom_ignore_comment_lines() {
        let overrides = PinyinOverrides::from_text(
            "# 站点词典\n// 注释\n\nfoo=福\n",
        );
        assert_eq!(overrides.get("foo"), Some("福"));
        assert!(!overrides.has("comment"));
    }

    #[test]
    fn parse_override_various_separators() {
        assert_eq!(
            parse_override_line("yindao=阴道").map(|(k, v)| (k, v)),
            Some(("yindao".into(), "阴道".into()))
        );
        assert_eq!(
            parse_override_line("yin_dao:阴道").map(|(k, v)| (k, v)),
            Some(("yindao".into(), "阴道".into()))
        );
        assert_eq!(
            parse_override_line("yindao 阴道").map(|(k, v)| (k, v)),
            Some(("yindao".into(), "阴道".into()))
        );
        assert!(parse_override_line("# only comment").is_none());
    }
}
