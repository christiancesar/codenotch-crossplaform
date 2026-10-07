//! The reply: `limits: [{kind, percent, resets_at}]` is the forward-compatible main shape;
//! `five_hour` / `seven_day` (`{utilization, resets_at}`) are merged in as a fallback, because a
//! window that just rolled over disappears from `limits` for a while.

use crate::providers::LimitWindow;
use crate::support::time::parse_iso;
use serde_json::Value;

fn label_for(kind: &str) -> String {
    match kind {
        "session" => "Current session".into(),
        "seven_day" | "weekly_all" => "Weekly (all models)".into(),
        "seven_day_opus" | "weekly_opus" => "Weekly (Opus)".into(),
        "weekly_scoped" => "Weekly (model-scoped)".into(),
        // An unknown kind still gets a readable label
        other => crate::support::text::cap(&other.replace('_', " ")),
    }
}

pub fn windows(v: &Value) -> Vec<LimitWindow> {
    let mut out: Vec<LimitWindow> = Vec::new();
    for l in v.get("limits").and_then(Value::as_array).into_iter().flatten() {
        let (Some(kind), Some(pct)) = (l.get("kind").and_then(Value::as_str), l.get("percent").and_then(Value::as_f64)) else {
            continue;
        };
        // A window without a reset time is not shown
        let Some(resets_at) = parse_iso(l.get("resets_at")) else { continue };
        out.push(LimitWindow {
            id: kind.into(),
            label: label_for(kind),
            used: (pct / 100.0).clamp(0.0, 1.0),
            resets_at: Some(resets_at),
            ..Default::default()
        });
    }

    // The kinds in `limits` are weekly_all / weekly_scoped, not seven_day, so deduplicating by id
    // alone showed "Weekly all" and "Weekly (all models)" as twins. A fallback window is a
    // duplicate if its id is an alias, its label matches, or its reset and percentage match.
    let aliases: [(&str, &str, &[&str]); 2] = [
        ("five_hour", "session", &["session", "five_hour"]),
        ("seven_day", "seven_day", &["seven_day", "weekly_all", "weekly"]),
    ];
    for (field, id, alias) in aliases {
        let Some(u) = v.get(field).and_then(|w| w.get("utilization")).and_then(Value::as_f64) else { continue };
        let used = (u / 100.0).clamp(0.0, 1.0);
        let resets_at = parse_iso(v[field].get("resets_at"));
        let label = label_for(id);
        let dup = out.iter().any(|x| {
            alias.contains(&x.id.as_str())
                || x.label == label
                || (resets_at.is_some()
                    && x.resets_at.map(|r| r / 1000) == resets_at.map(|r| r / 1000)
                    && (x.used - used).abs() < 0.005)
        });
        if !dup {
            out.push(LimitWindow { id: id.into(), label, used, resets_at, ..Default::default() });
        }
    }
    out.sort_by_key(|w| if w.id == "session" { 0 } else { 1 });
    out
}

#[cfg(test)]
mod tests {
    use super::windows;
    use serde_json::json;

    /// Real /api/oauth/usage body (no secrets in it). If Anthropic changes the shape, this fails first.
    #[test]
    fn recorded_body_parses_to_session_and_weekly() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/vendors/claude-usage.json");
        let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        let w = windows(&v);
        assert_eq!(w.len(), 2, "{w:?}");
        assert_eq!((w[0].id.as_str(), w[0].label.as_str()), ("session", "Current session"));
        assert!((w[0].used - 0.46).abs() < 0.001 && w[0].resets_at.is_some());
        assert_eq!((w[1].id.as_str(), w[1].label.as_str()), ("weekly_all", "Weekly (all models)"));
        assert!((w[1].used - 0.18).abs() < 0.001 && w[1].resets_at.is_some());
    }

    #[test]
    fn fallback_fills_a_window_missing_from_limits_and_never_twins_one() {
        let v = json!({
            "limits": [{"kind": "weekly_all", "percent": 20, "resets_at": "2026-10-10T00:00:00Z"}],
            "five_hour": {"utilization": 35, "resets_at": "2026-10-06T15:00:00Z"},
            "seven_day": {"utilization": 20, "resets_at": "2026-10-10T00:00:00Z"}
        });
        let w = windows(&v);
        let ids: Vec<_> = w.iter().map(|x| x.id.as_str()).collect();
        assert_eq!(ids, vec!["session", "weekly_all"]);
        assert!((w[0].used - 0.35).abs() < 1e-9);
    }

    #[test]
    fn windows_without_a_reset_and_unknown_kinds() {
        let v = json!({"limits": [
            {"kind": "session", "percent": 10},
            {"kind": "monthly_extra", "percent": 150, "resets_at": "2026-11-01T00:00:00Z"}
        ]});
        let w = windows(&v);
        assert_eq!(w.len(), 1);
        assert_eq!((w[0].label.as_str(), w[0].used), ("Monthly extra", 1.0));
    }
}
