//! The desktop app keeps real turn state in `~/.codex/thread_history_1.sqlite`; the CLI and the
//! editor extension only leave the rollout log, whose last entry is classified with a silence
//! threshold that depends on its kind.

use crate::providers::{Activity, ActivityState, ProviderId};
use crate::support::sqlite::{open_live, LiveQuery};
use crate::support::time::parse_iso;
use rusqlite::Connection;
use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Step {
    /// A tool is running, or waiting for approval
    Tool,
    /// The model is deciding the next step
    Thinking,
    /// Final answer or narration along the way
    AsstMsg,
    Ended,
}

/// The last meaningful rollout entry and its time. task_started / task_complete are not always
/// written, so the kind of entry carries the decision; bookkeeping (token_count...) is skipped.
pub fn last_step(text: &str) -> Option<(Step, u64)> {
    text.lines().rev().filter(|l| !l.trim().is_empty()).find_map(|line| {
        let v: serde_json::Value = serde_json::from_str(line).ok()?;
        let ts = parse_iso(v.get("timestamp")).unwrap_or(0);
        let pt = v.pointer("/payload/type").and_then(|x| x.as_str()).unwrap_or("");
        let step = match v.get("type").and_then(|x| x.as_str()).unwrap_or("") {
            "turn_context" => Step::Thinking,
            "response_item" => match pt {
                "function_call" | "local_shell_call" | "custom_tool_call" | "web_search_call" => Step::Tool,
                "function_call_output" | "custom_tool_call_output" | "reasoning" => Step::Thinking,
                "message" => match v.pointer("/payload/role").and_then(|x| x.as_str()) {
                    Some("assistant") => Step::AsstMsg,
                    Some("user") => Step::Thinking,
                    _ => return None,
                },
                _ => return None,
            },
            "event_msg" => match pt {
                "turn_aborted" | "task_complete" => Step::Ended,
                "task_started" | "item_started" | "exec_command_begin" | "user_message" | "agent_reasoning" | "agent_reasoning_raw_content" => Step::Thinking,
                "agent_message" => Step::AsstMsg,
                _ => return None,
            },
            _ => return None,
        };
        Some((step, ts))
    })
}

/// Busy while the silence is shorter than the step tolerates: tools can run for minutes, the
/// model can think for two, an assistant message is usually the end.
pub fn busy(step: Step, quiet_ms: u64) -> bool {
    match step {
        Step::Tool => quiet_ms <= 10 * 60_000,
        Step::Thinking => quiet_ms <= 120_000,
        Step::AsstMsg => quiet_ms <= 4_000,
        Step::Ended => false,
    }
}

/// Turns with status inProgress. "inProgress forever after a crash" is guarded: no new item for
/// 10 min while started over 2 min ago counts as stale.
fn turns_in_progress(conn: &Connection, names: Option<&Connection>, now: u64) -> Option<Vec<Activity>> {
    let mut stmt = conn.prepare("SELECT thread_id, started_at FROM thread_turns WHERE status = 'inProgress' ORDER BY started_at DESC LIMIT 8").ok()?;
    let rows: Vec<(String, rusqlite::types::Value)> = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?))).ok()?.flatten().collect();
    let mut out = Vec::new();
    for (thread_id, started) in rows {
        // Seconds or milliseconds, depending on the build that wrote it
        let started_ms = match started {
            rusqlite::types::Value::Integer(i) => (i as u64) * if i > 10_000_000_000 { 1 } else { 1000 },
            rusqlite::types::Value::Real(f) => (f * if f > 10_000_000_000.0 { 1.0 } else { 1000.0 }) as u64,
            _ => 0,
        };
        let (last_ms, last_type): (Option<i64>, Option<String>) = conn
            .query_row(
                "SELECT created_at_ms, item_type FROM thread_items WHERE thread_id = ?1 ORDER BY created_at_ms DESC LIMIT 1",
                [&thread_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap_or((None, None));
        let last = last_ms.map(|v| v as u64).unwrap_or(started_ms);
        if now.saturating_sub(last) > 10 * 60_000 && now.saturating_sub(started_ms) > 2 * 60_000 {
            continue;
        }
        let name = names
            .and_then(|c| {
                c.query_row(
                    "SELECT COALESCE(title,''), COALESCE(first_user_message,''), COALESCE(agent_nickname,'') FROM threads WHERE id = ?1",
                    [&thread_id],
                    |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?)),
                )
                .ok()
            })
            .and_then(|(title, first, nick)| {
                if !title.trim().is_empty() {
                    Some(title)
                } else if !first.trim().is_empty() {
                    Some(first.chars().take(40).collect())
                } else if !nick.trim().is_empty() {
                    Some(format!("Agent {nick}"))
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "Codex".into());
        let lt = last_type.unwrap_or_default().to_lowercase();
        let waiting = lt.contains("approval") || lt.contains("permission") || lt.contains("request_user");
        out.push(Activity {
            provider: ProviderId::Codex,
            state: if waiting { ActivityState::Waiting } else { ActivityState::Busy },
            name,
            detail: if waiting { "needs your input".into() } else { "Working".into() },
            since: started_ms,
        });
    }
    Some(out)
}

pub struct Probe {
    turns: LiveQuery<Vec<Activity>>,
    names: Option<Connection>,
    rollout: Option<PathBuf>,
    rollout_checked_at: u64,
    rollout_mtime: u64,
    rollout_last: Vec<Activity>,
}

impl Probe {
    pub fn new() -> Probe {
        let home = super::codex_home().unwrap_or_default();
        Probe {
            turns: LiveQuery::new(home.join("thread_history_1.sqlite")),
            names: None,
            rollout: None,
            rollout_checked_at: 0,
            rollout_mtime: 0,
            rollout_last: Vec::new(),
        }
    }

    pub fn read(&mut self, now: u64) -> Vec<Activity> {
        if self.names.is_none() {
            self.names = super::codex_home().and_then(|h| open_live(&h.join("state_5.sqlite")));
        }
        let names = self.names.as_ref();
        let turns = self.turns.get(|c| turns_in_progress(c, names, now));
        if !turns.is_empty() {
            return turns;
        }
        // The CLI: find the rollout every 30 s, re-read its tail only when it changed
        if self.rollout.is_none() || now.saturating_sub(self.rollout_checked_at) > 30_000 {
            self.rollout_checked_at = now;
            self.rollout = super::codex_home().and_then(|h| super::rollout::newest(&h.join("sessions")));
        }
        let Some(p) = self.rollout.clone() else { return vec![] };
        let mtime = std::fs::metadata(&p)
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        if mtime == self.rollout_mtime {
            self.rollout_last.retain(|a| now.saturating_sub(a.since) <= 10 * 60_000);
            return self.rollout_last.clone();
        }
        self.rollout_mtime = mtime;
        self.rollout_last = super::rollout::tail_text(&p)
            .and_then(|t| last_step(&t))
            .map(|(step, ts)| (step, ts.max(mtime)))
            .filter(|(step, at)| busy(*step, now.saturating_sub(*at)))
            .map(|(_, at)| vec![Activity { provider: ProviderId::Codex, state: ActivityState::Busy, name: "Codex".into(), detail: "Working".into(), since: at }])
            .unwrap_or_default();
        self.rollout_last.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(kind: &str, payload: &str) -> String {
        format!(r#"{{"timestamp":"2026-10-06T12:00:00Z","type":"{kind}","payload":{payload}}}"#)
    }

    #[test]
    fn the_last_meaningful_entry_decides() {
        let tool = line("response_item", r#"{"type":"function_call"}"#);
        let count = line("event_msg", r#"{"type":"token_count"}"#);
        assert_eq!(last_step(&[tool.clone(), count].join("\n")).unwrap().0, Step::Tool);
        let done = line("event_msg", r#"{"type":"task_complete"}"#);
        assert_eq!(last_step(&[tool, done].join("\n")).unwrap().0, Step::Ended);
        let sys = line("response_item", r#"{"type":"message","role":"developer"}"#);
        assert!(last_step(&sys).is_none());
    }

    #[test]
    fn silence_tolerance_depends_on_the_step() {
        assert!(busy(Step::Tool, 9 * 60_000) && !busy(Step::Tool, 11 * 60_000));
        assert!(busy(Step::Thinking, 119_000) && !busy(Step::Thinking, 121_000));
        assert!(busy(Step::AsstMsg, 3_000) && !busy(Step::AsstMsg, 5_000));
        assert!(!busy(Step::Ended, 0));
    }

    #[test]
    fn in_progress_turns_with_names_and_the_crash_guard() {
        let c = Connection::open_in_memory().unwrap();
        c.execute_batch(
            "CREATE TABLE thread_turns (thread_id TEXT, started_at INTEGER, status TEXT);
             CREATE TABLE thread_items (thread_id TEXT, created_at_ms INTEGER, item_type TEXT);
             INSERT INTO thread_turns VALUES ('t1', 1000, 'inProgress'), ('t2', 1, 'inProgress'), ('t3', 1000, 'completed');
             INSERT INTO thread_items VALUES ('t1', 1000500, 'exec_approval_request');",
        )
        .unwrap();
        let names = Connection::open_in_memory().unwrap();
        names.execute_batch("CREATE TABLE threads (id TEXT, title TEXT, first_user_message TEXT, agent_nickname TEXT); INSERT INTO threads VALUES ('t1', '', 'refactor the parser please', NULL);").unwrap();
        let a = turns_in_progress(&c, Some(&names), 1_001_000).unwrap();
        assert_eq!(a.len(), 1, "t2 started long ago with no items: stale");
        assert_eq!((a[0].state, a[0].name.as_str(), a[0].since), (ActivityState::Waiting, "refactor the parser please", 1_000_000));
    }
}
