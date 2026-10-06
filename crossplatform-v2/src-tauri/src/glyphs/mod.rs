//! Provider marks. No vendor logo is drawn here; only existing artwork, in this order:
//!   1. a user override, `<config dir>/glyphs/<id>.svg|.png` or `glyphs/` next to the executable;
//!   2. the built-in SVGs from npm `@lobehub/icons-static-svg` 1.95.0 (MIT), unmodified, see
//!      assets/glyphs/NOTICE.md.
//! v0.3 also had a third step (the installed app's icon from its .exe), but every provider has a
//! built-in mark, so it could never run and was not carried over. SVGs are inlined into the DOM
//! (`fill="currentColor"` follows the page's state), so they are sanitized first.

use crate::providers::ProviderId;
use serde::Serialize;
use specta::Type;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "lowercase")]
pub enum GlyphKind {
    /// Inline SVG, monochrome, follows currentColor
    Svg,
    /// Bitmap artwork, as a data: URL
    Png,
}

#[derive(Debug, Clone, PartialEq, Serialize, Type)]
pub struct Glyph {
    pub kind: GlyphKind,
    /// data: URL for png
    pub url: String,
    /// The sanitized SVG text for svg
    pub svg: String,
    /// Where it came from, for doctor
    pub source: String,
}

fn builtin(id: ProviderId) -> &'static str {
    match id {
        ProviderId::Claude => include_str!("../../assets/glyphs/claude.svg"),
        ProviderId::Codex => include_str!("../../assets/glyphs/codex.svg"),
        ProviderId::Cursor => include_str!("../../assets/glyphs/cursor.svg"),
        ProviderId::Gemini => include_str!("../../assets/glyphs/gemini.svg"),
        ProviderId::Opencode => include_str!("../../assets/glyphs/opencode.svg"),
    }
}

/// Minimal SVG sanitising before inlining into the DOM: drop <script> blocks and on*="…" event
/// attributes (the built-in files have none; this guards user files). Every slice position comes
/// from an ASCII pattern match and lands on a character boundary, so non-ASCII content is safe.
fn sanitize_svg(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let lower = s.to_ascii_lowercase();
    let mut i = 0;
    while let Some(rel) = lower[i..].find("<script") {
        out.push_str(&s[i..i + rel]);
        match lower[i + rel..].find("</script>") {
            Some(e) => i = i + rel + e + "</script>".len(),
            None => {
                i = s.len();
                break;
            }
        }
    }
    out.push_str(&s[i..]);
    let lo = out.to_ascii_lowercase();
    let mut res = String::with_capacity(out.len());
    let mut i = 0;
    loop {
        let Some(rel) = lo[i..].find(" on") else { break };
        let start = i + rel;
        let name_len = lo[start + 3..].bytes().take_while(|b| b.is_ascii_alphanumeric()).count();
        let eq = start + 3 + name_len;
        if name_len > 0 && lo.as_bytes().get(eq) == Some(&b'=') {
            if let Some(&q) = lo.as_bytes().get(eq + 1) {
                if q == b'"' || q == b'\'' {
                    if let Some(close) = lo[eq + 2..].find(q as char) {
                        res.push_str(&out[i..start]);
                        i = eq + 2 + close + 1;
                        continue;
                    }
                }
            }
        }
        res.push_str(&out[i..start + 3]);
        i = start + 3;
    }
    res.push_str(&out[i..]);
    res
}

pub fn user_dir() -> PathBuf {
    crate::storage::paths::config_dir().join("glyphs")
}

fn override_dirs() -> Vec<PathBuf> {
    let mut v = vec![user_dir()];
    if let Some(d) = std::env::current_exe().ok().and_then(|e| e.parent().map(|p| p.join("glyphs"))) {
        v.push(d);
    }
    v
}

fn from_file(p: &Path) -> Option<Glyph> {
    let bytes = std::fs::read(p).ok().filter(|b| !b.is_empty() && b.len() <= 512 * 1024)?;
    let source = p.display().to_string();
    match p.extension()?.to_string_lossy().to_lowercase().as_str() {
        "svg" => Some(Glyph { kind: GlyphKind::Svg, url: String::new(), svg: sanitize_svg(&String::from_utf8_lossy(&bytes)), source }),
        "png" => Some(Glyph { kind: GlyphKind::Png, url: format!("data:image/png;base64,{}", crate::support::base64::encode(&bytes)), svg: String::new(), source }),
        _ => None,
    }
}

fn glyph_for(id: ProviderId, dirs: &[PathBuf]) -> Glyph {
    dirs.iter()
        .flat_map(|d| ["svg", "png"].map(|ext| d.join(format!("{}.{ext}", id.as_str()))))
        .find_map(|p| from_file(&p))
        .unwrap_or_else(|| Glyph {
            kind: GlyphKind::Svg,
            url: String::new(),
            svg: sanitize_svg(builtin(id)),
            source: "built-in · @lobehub/icons-static-svg 1.95.0 (MIT)".into(),
        })
}

/// Every provider's mark, keyed by its id string as the page uses it.
pub fn collect() -> HashMap<String, Glyph> {
    let dirs = override_dirs();
    ProviderId::ALL.into_iter().map(|id| (id.as_str().to_string(), glyph_for(id, &dirs))).collect()
}

pub fn probe() -> String {
    let m = collect();
    let mut lines = vec![format!("glyph directory: {} (drop claude/codex/cursor/gemini/opencode .svg or .png files here)", user_dir().display())];
    for id in ProviderId::ALL {
        let g = &m[id.as_str()];
        lines.push(format!("  {}: {:?} ← {}", id.as_str(), g.kind, g.source));
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scripts_and_event_handlers_are_stripped() {
        let s = r#"<svg onload="x()"><script>alert(1)</script><path d="M0" ONCLICK='y' fill="currentColor"/>é</svg>"#;
        let clean = sanitize_svg(s);
        assert!(!clean.to_lowercase().contains("script") && !clean.to_lowercase().contains("onload") && !clean.to_lowercase().contains("onclick"));
        assert!(clean.contains(r#"fill="currentColor""#) && clean.contains('é'));
    }

    #[test]
    fn an_override_wins_over_the_builtin_mark() {
        let dir = tempfile::tempdir().unwrap();
        assert!(glyph_for(ProviderId::Cursor, &[dir.path().to_path_buf()]).source.starts_with("built-in"));
        std::fs::write(dir.path().join("cursor.png"), [0x89, b'P', b'N', b'G']).unwrap();
        let g = glyph_for(ProviderId::Cursor, &[dir.path().to_path_buf()]);
        assert_eq!(g.kind, GlyphKind::Png);
        assert!(g.url.starts_with("data:image/png;base64,"));
    }

    #[test]
    fn every_provider_has_a_builtin_mark() {
        for id in ProviderId::ALL {
            assert!(builtin(id).contains("<svg"), "{id:?}");
        }
    }
}
