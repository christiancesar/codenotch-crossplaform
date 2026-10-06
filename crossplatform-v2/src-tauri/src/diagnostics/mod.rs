//! Diagnostics: run.log now; `doctor` and `doctor deep` arrive in phase 4.

/// Appends one line to run.log. Tests print instead, so `cargo test` never touches the real
/// config directory.
pub fn log(line: &str) {
    if cfg!(test) {
        eprintln!("{line}");
        return;
    }
    use std::io::Write;
    let path = crate::storage::paths::config_dir().join(crate::storage::paths::RUN_LOG);
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(f, "{line}");
    }
}
