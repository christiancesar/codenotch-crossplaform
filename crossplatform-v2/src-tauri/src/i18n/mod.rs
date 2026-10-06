//! Strings drawn by Rust (the tray menu). The page has its own dictionary; keys stay identical.

use crate::platform::{Locale, Platform};

pub fn resolve(lang: &str) -> &'static str {
    match lang {
        "auto" => Platform.system_lang(),
        "zh" => "zh",
        "ja" => "ja",
        "ko" => "ko",
        "pt" => "pt",
        _ => "en",
    }
}

pub fn tr(lang: &str, key: &str) -> &'static str {
    match (resolve(lang), key) {
        ("zh", "settings") => "设置…",
        ("ja", "settings") => "設定…",
        ("ko", "settings") => "설정…",
        ("pt", "settings") => "Configurações…",
        (_, "settings") => "Settings…",
        ("zh", "refresh") => "立即刷新用量",
        ("ja", "refresh") => "使用量を今すぐ更新",
        ("ko", "refresh") => "사용량 지금 새로고침",
        ("pt", "refresh") => "Atualizar uso agora",
        (_, "refresh") => "Refresh usage now",
        ("zh", "quit") => "退出",
        ("ja", "quit") => "終了",
        ("ko", "quit") => "종료",
        ("pt", "quit") => "Sair",
        (_, "quit") => "Quit",
        _ => "?",
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn falls_back_to_english() {
        assert_eq!(super::tr("pt", "quit"), "Sair");
        assert_eq!(super::tr("de", "quit"), "Quit");
        assert_eq!(super::tr("en", "nope"), "?");
    }
}
