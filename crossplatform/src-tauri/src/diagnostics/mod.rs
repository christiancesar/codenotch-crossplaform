//! Diagnostics: run.log, watch.log, and `codenotch doctor [deep]`, which looks instead of
//! guessing. Every check prints structure, times and short scalars only: never a token, never
//! conversation content.

mod checks;
mod deep;

/// `codenotch doctor`
pub fn run() -> String {
    let dir = crate::storage::paths::config_dir();
    let mut r = Report::new(&format!("Codenotch doctor v{}", env!("CARGO_PKG_VERSION")));
    r.section("config", checks::config(&dir));
    r.section("hook server port", checks::port(&dir));
    r.section("session transcripts", checks::sessions());
    r.section("usage sources", checks::providers(&crate::providers::all()));
    r.section("provider glyphs", crate::glyphs::probe());
    r.section("watch.log (most recent watcher lines)", checks::watch_log(&dir));
    r.render()
}

/// `codenotch doctor deep`: the raw material behind the working-state signals.
pub fn run_deep() -> String {
    deep::run()
}

/// Sections rendered to text at the end, so each check stays a plain function returning a string.
pub struct Report {
    title: String,
    sections: Vec<(String, String)>,
}

impl Report {
    pub fn new(title: &str) -> Report {
        Report { title: title.into(), sections: Vec::new() }
    }
    pub fn section(&mut self, name: &str, body: String) {
        self.sections.push((name.into(), body));
    }
    pub fn render(&self) -> String {
        let mut o = format!("== {} ==\n", self.title);
        for (name, body) in &self.sections {
            o += &format!("\n{name}:\n");
            for line in body.lines() {
                o += &format!("  {line}\n");
            }
        }
        o
    }
}

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

/// watch.log: what the transcript watcher saw, for `doctor`. Cleared at each start.
pub fn watch_log(line: &str) {
    if cfg!(test) {
        return;
    }
    use std::io::Write;
    let path = crate::storage::paths::config_dir().join("watch.log");
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(f, "[{}] {line}", crate::support::time::now_ms());
    }
}

pub fn clear_watch_log() {
    if !cfg!(test) {
        let _ = std::fs::write(crate::storage::paths::config_dir().join("watch.log"), "");
    }
}
