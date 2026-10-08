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
        ("zh", "notchPosition") => "刘海位置",
        ("ja", "notchPosition") => "ノッチの位置",
        ("ko", "notchPosition") => "노치 위치",
        ("pt", "notchPosition") => "Posição do notch",
        (_, "notchPosition") => "Notch position",
        ("zh", "top") => "顶部",
        ("ja", "top") => "上",
        ("ko", "top") => "위",
        ("pt", "top") => "Topo",
        (_, "top") => "Top",
        ("zh", "left") => "左侧",
        ("ja", "left") => "左",
        ("ko", "left") => "왼쪽",
        ("pt", "left") => "Esquerda",
        (_, "left") => "Left",
        ("zh", "right") => "右侧",
        ("ja", "right") => "右",
        ("ko", "right") => "오른쪽",
        ("pt", "right") => "Direita",
        (_, "right") => "Right",
        ("zh", "bottom") => "底部",
        ("ja", "bottom") => "下",
        ("ko", "bottom") => "아래",
        ("pt", "bottom") => "Embaixo",
        (_, "bottom") => "Bottom",
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
