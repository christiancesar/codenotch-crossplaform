//! States ordered by how much they need you: attention > running > done > idle. Done stays until
//! the next prompt in that session, the user's dismiss, the seen-clears-it scan, or 24 h.

use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;

const RUNNING_STALE_MS: u64 = 30 * 60 * 1000; // running with no event for 30 min: an abnormal exit
const DONE_STALE_MS: u64 = 24 * 3600 * 1000;
const IDLE_DROP_MS: u64 = 10 * 60 * 1000;
/// Hook data counts as fresh this long; transcript inference yields to it meanwhile
const HOOK_FRESH_MS: u64 = 5 * 60 * 1000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(rename_all = "lowercase")]
pub enum SessionState {
    Attention,
    Running,
    Done,
    Idle,
}

impl SessionState {
    fn rank(self) -> u8 {
        match self {
            SessionState::Attention => 0,
            SessionState::Running => 1,
            SessionState::Done => 2,
            SessionState::Idle => 3,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Type)]
pub struct Session {
    pub id: String,
    pub title: String,
    pub state: SessionState,
    /// Start of the current activity, epoch ms
    pub started: u64,
    /// Elapsed time frozen at done, ms
    pub total: u64,
    /// Last action ("🔧 Bash: cargo test")
    pub last: String,
    /// What attention is about (a permission request, a question)
    pub attn: String,
    /// The user's latest input: the card shows what you said, not the agent's action
    pub prompt: String,
    /// The model the session actually uses
    pub model: String,
    #[serde(skip)]
    pub ppid: u32,
    #[serde(skip)]
    pub cwd: String,
    #[serde(skip)]
    last_event: u64,
    #[serde(skip)]
    last_hook: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Type)]
pub struct SessionsSnapshot {
    pub sessions: Vec<Session>,
    /// The state that needs you most across all sessions
    pub agg: SessionState,
    pub counts: HashMap<SessionState, u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// A real hook event
    Hook,
    /// Inferred from transcript appends
    Watch,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HookEvent {
    /// session_start | running | attention | done | session_end
    pub e: String,
    pub session_id: String,
    pub ppid: u32,
    pub cwd: String,
    pub prompt: String,
    pub message: String,
    pub tool_name: String,
    pub tool_cmd: String,
    pub model: String,
    pub src: Source,
}

#[derive(Default)]
pub struct Store {
    map: HashMap<String, Session>,
}

fn truncate(s: &str, n: usize) -> String {
    let mut out: String = s.chars().take(n).collect();
    if s.chars().count() > n {
        out.push('…');
    }
    out
}

fn title_of(cwd: &str, id: &str) -> String {
    let base = cwd.replace('\\', "/").rsplit('/').find(|p| !p.is_empty()).unwrap_or("claude").to_string();
    format!("{base} · {}", id.chars().take(4).collect::<String>())
}

impl Store {
    /// Returns whether anything visible changed, so the watcher's rapid appends cannot cause a
    /// broadcast storm.
    pub fn apply(&mut self, ev: HookEvent, now: u64) -> bool {
        if ev.e == "session_end" {
            return self.map.remove(&ev.session_id).is_some();
        }
        let s = self.map.entry(ev.session_id.clone()).or_insert_with(|| Session {
            id: ev.session_id.clone(),
            title: title_of(&ev.cwd, &ev.session_id),
            state: SessionState::Idle,
            started: now,
            total: 0,
            last: String::new(),
            attn: String::new(),
            prompt: String::new(),
            model: String::new(),
            ppid: 0,
            cwd: ev.cwd.clone(),
            last_event: now,
            last_hook: None,
        });
        // A session with fresh hook data ignores inference: the CLI's hooks are exact
        if ev.src == Source::Watch && s.last_hook.is_some_and(|h| now.saturating_sub(h) < HOOK_FRESH_MS) {
            return false;
        }
        if ev.src == Source::Hook {
            s.last_hook = Some(now);
        }
        let before = s.clone();
        s.last_event = now;
        if ev.ppid != 0 {
            s.ppid = ev.ppid;
        }
        if !ev.model.is_empty() {
            s.model = ev.model.clone();
        }
        if !ev.cwd.is_empty() && s.cwd.is_empty() {
            s.cwd = ev.cwd.clone();
            s.title = title_of(&ev.cwd, &s.id);
        }
        match ev.e.as_str() {
            "session_start" if s.state != SessionState::Running => s.state = SessionState::Idle,
            "running" => {
                if s.state != SessionState::Running {
                    s.started = now;
                }
                s.state = SessionState::Running;
                s.attn.clear();
                if !ev.prompt.is_empty() {
                    s.prompt = truncate(&ev.prompt, 120);
                }
                if !ev.tool_name.is_empty() {
                    s.last = if ev.tool_cmd.is_empty() {
                        format!("🔧 {}", ev.tool_name)
                    } else {
                        format!("🔧 {}: {}", ev.tool_name, truncate(&ev.tool_cmd, 60))
                    };
                }
            }
            "attention" => {
                s.state = SessionState::Attention;
                if !ev.message.is_empty() {
                    s.attn = truncate(&ev.message, 200);
                }
            }
            "done" => {
                if s.state != SessionState::Done {
                    s.total = now.saturating_sub(s.started);
                }
                s.state = SessionState::Done;
                s.attn.clear();
            }
            _ => {}
        }
        (&s.state, &s.last, &s.attn, &s.prompt, &s.model, &s.title) != (&before.state, &before.last, &before.attn, &before.prompt, &before.model, &before.title)
    }

    pub fn dismiss(&mut self, id: &str) -> bool {
        self.map.remove(id).is_some()
    }

    pub fn has_done(&self) -> bool {
        self.map.values().any(|s| s.state == SessionState::Done)
    }

    pub fn is_active(&self) -> bool {
        !self.map.is_empty()
    }

    /// Seen-clears-it: done sessions the predicate matches become idle, and the sweep drops them.
    pub fn ack_done(&mut self, now: u64, seen: impl Fn(&Session) -> bool) -> bool {
        let mut changed = false;
        for s in self.map.values_mut().filter(|s| s.state == SessionState::Done) {
            if seen(s) {
                s.state = SessionState::Idle;
                s.last_event = now;
                changed = true;
            }
        }
        changed
    }

    pub fn sweep(&mut self, now: u64) -> bool {
        let mut changed = false;
        for s in self.map.values_mut() {
            if s.state == SessionState::Running && now.saturating_sub(s.last_event) > RUNNING_STALE_MS {
                s.state = SessionState::Idle;
                changed = true;
            }
        }
        let before = self.map.len();
        self.map.retain(|_, s| {
            let age = now.saturating_sub(s.last_event);
            !(s.state == SessionState::Idle && age > IDLE_DROP_MS || s.state == SessionState::Done && age > DONE_STALE_MS)
        });
        changed || self.map.len() != before
    }

    pub fn ppid_of(&self, id: &str) -> Option<u32> {
        self.map.get(id).map(|s| s.ppid).filter(|p| *p != 0)
    }

    pub fn snapshot(&self) -> SessionsSnapshot {
        let mut sessions: Vec<Session> = self.map.values().cloned().collect();
        sessions.sort_by(|a, b| a.state.rank().cmp(&b.state.rank()).then(b.started.cmp(&a.started)));
        let counts: HashMap<SessionState, u32> = [SessionState::Attention, SessionState::Running, SessionState::Done]
            .into_iter()
            .map(|k| (k, sessions.iter().filter(|s| s.state == k).count() as u32))
            .collect();
        let agg = [SessionState::Attention, SessionState::Running, SessionState::Done]
            .into_iter()
            .find(|k| counts[k] > 0)
            .unwrap_or(SessionState::Idle);
        SessionsSnapshot { sessions, agg, counts }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ev(e: &str, src: Source) -> HookEvent {
        HookEvent {
            e: e.into(),
            session_id: "abcd1234".into(),
            ppid: 0,
            cwd: "/home/u/projects/notch".into(),
            prompt: String::new(),
            message: String::new(),
            tool_name: String::new(),
            tool_cmd: String::new(),
            model: String::new(),
            src,
        }
    }

    #[test]
    fn a_turn_goes_running_attention_running_done() {
        let mut st = Store::default();
        assert!(st.apply(HookEvent { prompt: "fix the bug".into(), ..ev("running", Source::Hook) }, 1_000));
        let s = &st.snapshot().sessions[0];
        assert_eq!((s.title.as_str(), s.state, s.prompt.as_str()), ("notch · abcd", SessionState::Running, "fix the bug"));
        assert!(st.apply(HookEvent { message: "Allow Bash?".into(), ..ev("attention", Source::Hook) }, 2_000));
        assert_eq!(st.snapshot().agg, SessionState::Attention);
        assert!(st.apply(HookEvent { tool_name: "Bash".into(), tool_cmd: "cargo test".into(), ..ev("running", Source::Hook) }, 3_000));
        assert_eq!(st.snapshot().sessions[0].last, "🔧 Bash: cargo test");
        assert!(st.apply(ev("done", Source::Hook), 10_000));
        // Coming back from attention starts a new stretch of activity, so the total counts from 3 s
        let s = &st.snapshot().sessions[0];
        assert_eq!((s.state, s.total), (SessionState::Done, 7_000));
        assert!(!st.apply(ev("done", Source::Hook), 11_000), "no visible change, no broadcast");
    }

    #[test]
    fn inference_yields_to_fresh_hook_data() {
        let mut st = Store::default();
        st.apply(ev("running", Source::Hook), 0);
        assert!(!st.apply(ev("done", Source::Watch), HOOK_FRESH_MS - 1));
        assert!(st.apply(ev("done", Source::Watch), HOOK_FRESH_MS + 1));
    }

    #[test]
    fn sweep_ages_out_stuck_and_old_sessions() {
        let mut st = Store::default();
        st.apply(ev("running", Source::Hook), 0);
        assert!(!st.sweep(RUNNING_STALE_MS));
        // Running with no event for 30 min is an abnormal exit; already older than the idle limit
        assert!(st.sweep(RUNNING_STALE_MS + 1));
        assert!(st.snapshot().sessions.is_empty());
        st.apply(ev("done", Source::Hook), 0);
        assert!(!st.sweep(DONE_STALE_MS));
        assert!(st.sweep(DONE_STALE_MS + 1));
    }

    #[test]
    fn seen_clears_done_and_session_end_removes() {
        let mut st = Store::default();
        st.apply(ev("done", Source::Hook), 0);
        assert!(st.ack_done(5, |s| s.id == "abcd1234"));
        assert_eq!(st.snapshot().agg, SessionState::Idle);
        assert!(st.apply(ev("session_end", Source::Hook), 6));
        assert!(!st.is_active());
    }

    #[test]
    fn the_snapshot_serializes_states_in_lowercase() {
        let mut st = Store::default();
        st.apply(ev("running", Source::Hook), 0);
        let v = serde_json::to_value(st.snapshot()).unwrap();
        assert_eq!(v["agg"], "running");
        assert_eq!(v["sessions"][0]["state"], "running");
        assert!(v["sessions"][0].get("ppid").is_none());
    }
}
