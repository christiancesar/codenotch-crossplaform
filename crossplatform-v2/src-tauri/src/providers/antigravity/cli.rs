//! The official CLI: `agy --sandbox --print-timeout 30s --print /usage`, which prints rows like
//! `Gemini Models Weekly Limit Remaining 94% 2026-09-12T01:47:23Z`.

use crate::platform::{Executables, Platform, Pty};
use crate::providers::LimitWindow;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// `<data dir>/agy/bin/agy[.exe]` (where the installer puts it), `/usr/local/bin/agy`, then PATH.
pub fn find_agy() -> Option<PathBuf> {
    let names = Platform.exe_names("agy");
    let mut dirs: Vec<PathBuf> = dirs::data_local_dir().map(|d| d.join("agy").join("bin")).into_iter().collect();
    dirs.push(PathBuf::from("/usr/local/bin"));
    dirs.iter()
        .flat_map(|d| names.iter().map(move |n| d.join(n)))
        .find(|p| p.is_file())
        .or_else(|| Platform.find_on_path("agy"))
}

pub fn read_quota(agy: &Path, workdir: &Path) -> Result<Vec<LimitWindow>, String> {
    let out = Platform.run_captured(agy, &["--sandbox", "--print-timeout", "30s", "--print", "/usage"], Some(workdir), Duration::from_secs(70))?;
    parse_quota(&sanitize(&out))
}

/// Strips ANSI escapes (CSI, OSC, two-character) and normalizes line endings: through a pseudo
/// console the CLI paints its output like a terminal.
pub fn sanitize(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\x1b' => match chars.peek() {
                Some('[') => {
                    chars.next();
                    // CSI: parameter and intermediate bytes, ended by 0x40..=0x7E
                    while let Some(next) = chars.next() {
                        if ('\x40'..='\x7e').contains(&next) {
                            break;
                        }
                    }
                }
                Some(']') => {
                    chars.next();
                    // OSC: until BEL or ST (ESC \)
                    while let Some(next) = chars.next() {
                        if next == '\x07' {
                            break;
                        }
                        if next == '\x1b' && chars.peek() == Some(&'\\') {
                            chars.next();
                            break;
                        }
                    }
                }
                Some(&next) if ('\x40'..='\x5f').contains(&next) => {
                    chars.next();
                }
                _ => {}
            },
            '\r' if chars.peek() == Some(&'\n') => {}
            '\r' => out.push('\n'),
            c => out.push(c),
        }
    }
    out
}

/// Remaining percentage to fraction used. Any malformed row fails the whole read: a wrong table
/// must never become a dummy 0 % ring. Through a pseudo console the table has a "Quota:" header,
/// over a pipe it does not; the header carries nothing the rows don't.
pub fn parse_quota(clean: &str) -> Result<Vec<LimitWindow>, String> {
    let mut out = Vec::new();
    for line in clean.lines().filter(|l| l.contains("Limit Remaining")) {
        let (left, reset) = line.rsplit_once('%').ok_or("Invalid CLI quota row")?;
        let (label, remaining) = left.trim().rsplit_once(char::is_whitespace).ok_or("Missing quota percentage")?;
        let remaining: f64 = remaining.parse().map_err(|_| "Invalid quota percentage")?;
        if !remaining.is_finite() || !(0.0..=100.0).contains(&remaining) {
            return Err("Quota percentage is out of range".into());
        }
        let reset = chrono::DateTime::parse_from_rfc3339(reset.trim()).map_err(|_| "Invalid quota reset time")?.timestamp_millis();
        if reset <= 0 {
            return Err("Invalid quota reset time".into());
        }
        let label = label.split_whitespace().collect::<Vec<_>>().join(" ");
        let id = label.strip_suffix(" Remaining").ok_or("Unknown quota label")?.to_string();
        let short = id
            .replace(" Models", "")
            .replace(" models", "")
            .replace(" and ", "/")
            .replace(" Weekly Limit", " · Weekly")
            .replace(" Five Hour Limit", " · 5h");
        out.push(LimitWindow { id, label: short, used: ((100.0 - remaining) / 100.0).clamp(0.0, 1.0), resets_at: Some(reset as u64), ..Default::default() });
    }
    if out.is_empty() {
        return Err("CLI returned no recognised quota windows".into());
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "Quota:\nGemini Models Weekly Limit Remaining 94% 2026-09-12T01:47:23Z\nGemini Models Five Hour Limit Remaining 78% 2026-09-10T06:12:31Z\nClaude and GPT models Weekly Limit Remaining 100% 2026-09-17T01:47:23Z\nClaude and GPT models Five Hour Limit Remaining 100% 2026-09-10T08:13:44Z";

    #[test]
    fn official_sample_parses_and_bad_tables_fail_whole() {
        let w = parse_quota(SAMPLE).expect("valid sample");
        let got: Vec<_> = w.iter().map(|x| (x.id.as_str(), x.label.as_str())).collect();
        assert_eq!(
            got,
            vec![
                ("Gemini Models Weekly Limit", "Gemini · Weekly"),
                ("Gemini Models Five Hour Limit", "Gemini · 5h"),
                ("Claude and GPT models Weekly Limit", "Claude/GPT · Weekly"),
                ("Claude and GPT models Five Hour Limit", "Claude/GPT · 5h"),
            ]
        );
        assert!((w[0].used - 0.06).abs() < 1e-5 && (w[1].used - 0.22).abs() < 1e-5 && w[2].used == 0.0);
        assert!(w.iter().all(|x| x.resets_at.is_some()));
        for bad in [
            "".to_string(),
            "authentication required".into(),
            "Quota:\nunknown".into(),
            SAMPLE.replace("94%", "101%"),
            SAMPLE.replace("94%", "-5%"),
            SAMPLE.replace("94%", "NaN%"),
            SAMPLE.replace("2026-09-12T01:47:23Z", "not-a-date"),
        ] {
            assert!(parse_quota(&bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn terminal_paint_is_stripped() {
        let raw = "\x1b[?25h\x1b[32mQuota:\x1b[0m\r\nGemini Models Weekly Limit Remaining 94% 2026-09-12T01:47:23Z\r\n\x1b]0;title\x07";
        let clean = sanitize(raw);
        assert_eq!(clean, "Quota:\nGemini Models Weekly Limit Remaining 94% 2026-09-12T01:47:23Z\n");
        assert_eq!(parse_quota(&clean).unwrap()[0].label, "Gemini · Weekly");
    }

    #[test]
    #[ignore = "runs the signed-in official CLI; opt in for integration checks"]
    fn live_quota() {
        let agy = find_agy().expect("agy installed");
        let dir = tempfile::tempdir().unwrap();
        assert!(!read_quota(&agy, dir.path()).expect("quota").is_empty());
    }
}
