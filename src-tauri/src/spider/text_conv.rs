//! 简繁转换（OpenCC / MediaWiki 词表，后端高性能处理）

use zhconv::{zhconv, Variant};

/// 按用户设置转换中文变体：original | simplified | traditional
pub fn apply_chinese_variant(text: &str, variant: &str) -> String {
    match variant {
        "simplified" => to_simplified(text),
        "traditional" => to_traditional(text),
        _ => text.to_string(),
    }
}

/// 转简体
pub fn to_simplified(text: &str) -> String {
    zhconv(text, Variant::ZhHans)
}

/// 转繁体
pub fn to_traditional(text: &str) -> String {
    zhconv(text, Variant::ZhHant)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_to_simplified() {
        let out = to_simplified("軟體");
        assert!(out.contains("软") || out.contains("体"));
    }
}
