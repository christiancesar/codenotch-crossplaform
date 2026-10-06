//! Session state inferred from transcript appends, for the Claude desktop app (its settings.json
//! hooks do not fire). Rules:
//!   - the file is being appended                          -> running
//!   - last entry is plain assistant text, quiet > 2.5 s   -> done
//!   - last entry is an assistant tool_use, quiet > 20 s   -> attention (waiting for approval;
//!     inferred, so a slow tool can be misread)
//! The store ignores this while a session has fresh hook data.

use super::{HookEvent, Source};
use notify::{RecursiveMode, Watcher};
use std::collections::{HashMap, HashSet};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, RecvTimeoutError};
use std::sync::Arc;
use std::time::{Duration, Instant};

const QUIET_DONE_MS: u64 = 2_500;
const QUIET_ATTN_MS: u64 = 20_000;
/// A user entry and long silence: a stopped or abandoned turn. Long enough to tolerate hard
/// thinking, which otherwise reads as done.
const QUIET_USER_DONE_MS: u64 = 75_000;
/// Anything else silent this long must not keep the light green forever
const QUIET_OTHER_DONE_MS: u64 = 5 * 60_000;
/// notify does not deliver every event; a periodic rescan heals what it misses
const RESCAN: Duration = Duration::from_secs(45);
const FRESH_WINDOW_MS: u64 = 10 * 60 * 1000;
/// One message (a whole-file write) often exceeds 16 KB; a cut last line would never parse
const TAIL_BYTES: u64 = 256 * 1024;
/// While the desktop app streams, a transcript fires dozens of events per second. Each file is
/// read at most this often; v0.3 reading it on every event slowed the whole system.
const INGEST_MIN_GAP: Duration = Duration::from_millis(800);

/// The CLI's ~/.claude/projects and the desktop app's per-session mirrors (also inside an MSIX
/// package's virtualised AppData on Windows).
pub fn roots() -> Vec<PathBuf> {
    let mut v = Vec::new();
    if let Some(h) = dirs::home_dir() {
        v.push(h.join(".claude").join("projects"));
    }
    if let Some(c) = dirs::config_dir() {
        v.push(c.join("Claude").join("local-agent-mode-sessions"));
    }
    if let Some(local) = dirs::data_local_dir() {
        for e in std::fs::read_dir(local.join("Packages")).into_iter().flatten().flatten() {
            let name = e.file_name().to_string_lossy().to_lowercase();
            if name.contains("claude") || name.contains("anthropic") {
                v.push(e.path().join("LocalCache").join("Roaming").join("Claude").join("local-agent-mode-sessions"));
            }
        }
    }
    v
}

/// Real session transcripts only: inside a .claude tree, no audit log, no sub-agent.
pub fn is_session_jsonl(p: &Path) -> bool {
    if p.extension().is_none_or(|e| e != "jsonl") || p.file_name().is_some_and(|n| n == "audit.jsonl") {
        return false;
    }
    let parts: Vec<_> = p.components().map(|c| c.as_os_str().to_string_lossy().into_owned()).collect();
    !parts.iter().any(|s| s == "subagents") && parts.iter().any(|s| s == ".claude")
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Kind {
    User,
    AsstText,
    AsstTool,
    Other,
}

pub struct TailInfo {
    /// Latest user/assistant entry, bookkeeping lines skipped
    pub entry: serde_json::Value,
    /// Latest real user input (tool_result entries are not input)
    pub prompt: String,
    pub model: String,
}

fn user_text(v: &serde_json::Value) -> Option<String> {
    let c = v.pointer("/message/content")?;
    let take = |s: &str| {
        let s = s.trim();
        (!s.is_empty()).then(|| s.chars().take(120).collect())
    };
    if let Some(s) = c.as_str() {
        return take(s);
    }
    let arr = c.as_array()?;
    if arr.iter().any(|b| b.get("type").and_then(|t| t.as_str()) == Some("tool_result")) {
        return None;
    }
    arr.iter().filter(|b| b.get("type").and_then(|t| t.as_str()) == Some("text")).find_map(|b| take(b.get("text")?.as_str()?))
}

/// One backwards walk over the tail: main entry, latest input and model. A partial last line,
/// a line cut by the window or bookkeeping all just step back.
pub fn tail_info_of(text: &str) -> Option<TailInfo> {
    let (mut entry, mut prompt, mut model) = (None, String::new(), String::new());
    for line in text.lines().rev().filter(|l| !l.trim().is_empty()).take(80) {
        let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else { continue };
        let t = v.get("type").and_then(|x| x.as_str()).unwrap_or("");
        if t != "user" && t != "assistant" {
            continue;
        }
        if model.is_empty() && t == "assistant" {
            model = v.pointer("/message/model").and_then(|x| x.as_str()).unwrap_or("").to_string();
        }
        if prompt.is_empty() && t == "user" {
            prompt = user_text(&v).unwrap_or_default();
        }
        entry.get_or_insert(v);
        if !model.is_empty() && !prompt.is_empty() {
            break;
        }
    }
    entry.map(|entry| TailInfo { entry, prompt, model })
}

pub fn tail_info(path: &Path) -> Option<TailInfo> {
    let mut f = std::fs::File::open(path).ok()?;
    let len = f.metadata().map(|m| m.len()).unwrap_or(0);
    f.seek(SeekFrom::Start(len.saturating_sub(TAIL_BYTES))).ok()?;
    let mut raw = Vec::new();
    f.read_to_end(&mut raw).ok()?;
    tail_info_of(&String::from_utf8_lossy(&raw))
}

struct Track {
    session: String,
    cwd: String,
    last_append: u64,
    kind: Kind,
    /// Last state pushed, so a quiet file does not repeat it
    sent: &'static str,
    /// The last user entry is an interruption ("[Request interrupted...]"): done quickly
    interrupted: bool,
    prompt: String,
    model: String,
}

impl Track {
    fn event(&self, e: &str) -> HookEvent {
        HookEvent {
            e: e.into(),
            session_id: self.session.clone(),
            ppid: 0,
            cwd: self.cwd.clone(),
            prompt: self.prompt.clone(),
            message: String::new(),
            tool_name: String::new(),
            tool_cmd: String::new(),
            model: self.model.clone(),
            src: Source::Watch,
        }
    }
}

/// The inference, separate from the file watching so it can be tested with plain values.
#[derive(Default)]
pub struct Tracker {
    tracks: HashMap<PathBuf, Track>,
}

impl Tracker {
    /// A transcript was appended: record it and report running.
    pub fn ingest(&mut self, path: &Path, info: TailInfo, now: u64) -> Option<HookEvent> {
        let v = info.entry;
        // Transcript fields are camelCase (sessionId), unlike the hook stdin
        let session = v
            .get("sessionId")
            .and_then(|x| x.as_str())
            .map(String::from)
            .unwrap_or_else(|| path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default());
        if session.is_empty() {
            return None;
        }
        let mut cwd = v.get("cwd").and_then(|x| x.as_str()).unwrap_or("").to_string();
        // A desktop session's cwd is an internal outputs path that means nothing to the user
        if cwd.contains("local-agent-mode-sessions") {
            cwd = "Claude desktop".into();
        }
        let typ = v.get("type").and_then(|x| x.as_str()).unwrap_or("");
        let kind = match typ {
            "user" => Kind::User,
            "assistant" => {
                let tool = v.pointer("/message/content").and_then(|c| c.as_array()).is_some_and(|a| {
                    a.iter().any(|b| b.get("type").and_then(|t| t.as_str()) == Some("tool_use"))
                });
                if tool {
                    Kind::AsstTool
                } else {
                    Kind::AsstText
                }
            }
            _ => Kind::Other,
        };
        let interrupted = typ == "user" && v.get("message").is_some_and(|m| m.to_string().to_lowercase().contains("interrupt"));
        let t = self.tracks.entry(path.to_path_buf()).or_insert(Track {
            session: session.clone(),
            cwd: cwd.clone(),
            last_append: 0,
            kind,
            sent: "",
            interrupted: false,
            prompt: String::new(),
            model: String::new(),
        });
        t.session = session;
        if !cwd.is_empty() {
            t.cwd = cwd;
        }
        t.last_append = now;
        t.kind = kind;
        t.interrupted = interrupted;
        if !info.prompt.is_empty() {
            t.prompt = info.prompt;
        }
        if !info.model.is_empty() {
            t.model = info.model;
        }
        t.sent = "running";
        Some(t.event("running"))
    }

    pub fn last_append(&self, path: &Path) -> u64 {
        self.tracks.get(path).map(|t| t.last_append).unwrap_or(0)
    }

    /// Quiet-time decisions: done or (inferred) attention.
    pub fn evaluate(&mut self, now: u64) -> Vec<HookEvent> {
        let mut out = Vec::new();
        for t in self.tracks.values_mut() {
            let quiet = now.saturating_sub(t.last_append);
            let next = match t.kind {
                Kind::AsstText if quiet > QUIET_DONE_MS => "done",
                Kind::AsstTool if quiet > QUIET_ATTN_MS => "attention",
                Kind::User if t.interrupted && quiet > QUIET_DONE_MS => "done",
                Kind::User if quiet > QUIET_USER_DONE_MS => "done",
                Kind::Other if quiet > QUIET_OTHER_DONE_MS => "done",
                _ => continue,
            };
            if t.sent != next {
                t.sent = next;
                out.push(t.event(next));
            }
        }
        out
    }
}

fn walk(dir: &Path, depth: usize, out: &mut Vec<PathBuf>) {
    if depth > 10 {
        return;
    }
    for e in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let p = e.path();
        if p.is_dir() {
            walk(&p, depth + 1, out);
        } else if is_session_jsonl(&p) {
            out.push(p);
        }
    }
}

/// Session files active within the freshness window and newer than what we last saw: covers new
/// session directories and lost notify events alike.
fn rescan(tracker: &mut Tracker, sink: &super::hook_server::Sink) {
    let now = crate::support::time::now_ms();
    let mut found = Vec::new();
    for r in roots().into_iter().filter(|r| r.exists()) {
        walk(&r, 0, &mut found);
    }
    for p in found {
        let Some(mtime) = std::fs::metadata(&p)
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_millis() as u64)
        else {
            continue;
        };
        if now.saturating_sub(mtime) <= FRESH_WINDOW_MS && mtime > tracker.last_append(&p) {
            ingest_file(tracker, &p, sink);
        }
    }
}

fn ingest_file(tracker: &mut Tracker, path: &Path, sink: &super::hook_server::Sink) {
    if let Some(ev) = tail_info(path).and_then(|info| tracker.ingest(path, info, crate::support::time::now_ms())) {
        sink(ev);
    }
}

pub fn start(sink: super::hook_server::Sink) {
    std::thread::Builder::new()
        .name("transcript-watcher".into())
        .spawn(move || {
            use crate::platform::{Platform, Processes};
            Platform.lower_current_thread_priority();
            let (tx, rx) = channel();
            let Ok(mut w) = notify::recommended_watcher(move |res| {
                let _ = tx.send(res);
            }) else {
                crate::diagnostics::log("watcher: cannot create a file watcher");
                return;
            };
            crate::diagnostics::clear_watch_log();
            crate::diagnostics::watch_log(&format!("watcher started v{}", env!("CARGO_PKG_VERSION")));
            // Roots that do not exist yet (the CLI never ran) are retried every minute
            let mut pending: Vec<PathBuf> = roots();
            let mut try_watch = |pending: &mut Vec<PathBuf>, w: &mut notify::RecommendedWatcher| {
                pending.retain(|r| {
                    let ok = r.exists() && w.watch(r, RecursiveMode::Recursive).is_ok();
                    crate::diagnostics::watch_log(&format!("{}: {}", if ok { "watching" } else { "not available yet" }, r.display()));
                    !ok
                });
            };
            try_watch(&mut pending, &mut w);
            let mut last_retry = Instant::now();
            let mut tracker = Tracker::default();
            // Adopt sessions that were already active
            rescan(&mut tracker, &sink);
            let mut last_scan = Instant::now();
            let mut last_ingest: HashMap<PathBuf, Instant> = HashMap::new();
            let mut dirty: HashSet<PathBuf> = HashSet::new();
            loop {
                let mut take = |res: notify::Result<notify::Event>| {
                    if let Ok(ev) = res {
                        dirty.extend(ev.paths.into_iter().filter(|p| is_session_jsonl(p)));
                    }
                };
                match rx.recv_timeout(Duration::from_millis(400)) {
                    Ok(res) => {
                        take(res);
                        // Everything already queued in one go, instead of waking for each
                        while let Ok(res) = rx.try_recv() {
                            take(res);
                        }
                    }
                    Err(RecvTimeoutError::Timeout) => {}
                    Err(RecvTimeoutError::Disconnected) => return,
                }
                let now = Instant::now();
                let due: Vec<PathBuf> =
                    dirty.iter().filter(|p| last_ingest.get(*p).is_none_or(|t| now.duration_since(*t) >= INGEST_MIN_GAP)).cloned().collect();
                for p in due {
                    dirty.remove(&p);
                    last_ingest.insert(p.clone(), now);
                    ingest_file(&mut tracker, &p, &sink);
                }
                if last_ingest.len() > 512 {
                    last_ingest.retain(|_, t| now.duration_since(*t) < Duration::from_secs(600));
                }
                for ev in tracker.evaluate(crate::support::time::now_ms()) {
                    sink(ev);
                }
                if last_scan.elapsed() > RESCAN {
                    last_scan = Instant::now();
                    rescan(&mut tracker, &sink);
                }
                if !pending.is_empty() && last_retry.elapsed() > Duration::from_secs(60) {
                    last_retry = Instant::now();
                    try_watch(&mut pending, &mut w);
                }
            }
        })
        .expect("failed to start the transcript watcher");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn info(lines: &[&str]) -> TailInfo {
        tail_info_of(&lines.join("\n")).unwrap()
    }

    const USER: &str = r#"{"type":"user","sessionId":"s1","cwd":"/w/notch","message":{"content":"run the tests"}}"#;
    const TOOL: &str = r#"{"type":"assistant","sessionId":"s1","message":{"model":"claude-opus-5-5","content":[{"type":"tool_use","name":"Bash"}]}}"#;
    const TEXT: &str = r#"{"type":"assistant","sessionId":"s1","message":{"model":"claude-opus-5-5","content":[{"type":"text","text":"Done."}]}}"#;

    #[test]
    fn tail_finds_entry_prompt_and_model_past_bookkeeping_and_a_cut_line() {
        let t = info(&[USER, TOOL, r#"{"type":"summary"}"#, r#"{"type":"assist"#]);
        assert_eq!(t.entry["type"], "assistant");
        assert_eq!((t.prompt.as_str(), t.model.as_str()), ("run the tests", "claude-opus-5-5"));
    }

    #[test]
    fn quiet_text_is_done_quiet_tool_is_attention() {
        let p = Path::new("/h/.claude/projects/x/s1.jsonl");
        let mut tr = Tracker::default();
        assert_eq!(tr.ingest(p, info(&[USER, TEXT]), 0).unwrap().e, "running");
        assert!(tr.evaluate(QUIET_DONE_MS).is_empty());
        let done = tr.evaluate(QUIET_DONE_MS + 1);
        assert_eq!((done[0].e.as_str(), done[0].prompt.as_str()), ("done", "run the tests"));
        assert!(tr.evaluate(QUIET_DONE_MS + 2).is_empty(), "sent once");

        tr.ingest(p, info(&[USER, TOOL]), 100_000);
        assert!(tr.evaluate(100_000 + QUIET_DONE_MS + 1).is_empty(), "a running tool is not done");
        assert_eq!(tr.evaluate(100_000 + QUIET_ATTN_MS + 1)[0].e, "attention");
    }

    #[test]
    fn an_interrupted_turn_is_done_quickly() {
        let p = Path::new("/h/.claude/projects/x/s1.jsonl");
        let mut tr = Tracker::default();
        let stop = r#"{"type":"user","sessionId":"s1","message":{"content":"[Request interrupted by user]"}}"#;
        tr.ingest(p, info(&[stop]), 0);
        assert_eq!(tr.evaluate(QUIET_DONE_MS + 1)[0].e, "done");
    }

    #[test]
    fn only_real_session_transcripts_count() {
        assert!(is_session_jsonl(Path::new("/h/.claude/projects/p/abc.jsonl")));
        assert!(!is_session_jsonl(Path::new("/h/.claude/projects/p/subagents/a.jsonl")));
        assert!(!is_session_jsonl(Path::new("/h/.claude/projects/p/audit.jsonl")));
        assert!(!is_session_jsonl(Path::new("/h/other/p/abc.jsonl")));
        assert!(!is_session_jsonl(Path::new("/h/.claude/projects/p/abc.json")));
    }

    #[test]
    fn desktop_sessions_get_a_friendly_title() {
        let p = Path::new("/c/Claude/local-agent-mode-sessions/x/.claude/projects/y/s.jsonl");
        let line = r#"{"type":"user","sessionId":"s9","cwd":"/c/Claude/local-agent-mode-sessions/x/outputs","message":{"content":"hi"}}"#;
        let ev = Tracker::default().ingest(p, info(&[line]), 0).unwrap();
        assert_eq!(ev.cwd, "Claude desktop");
    }
}
