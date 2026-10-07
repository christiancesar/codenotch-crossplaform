use crate::providers::LimitWindow;
use serde_json::Value;

/// Codex names windows only by length, and "5h limit" says more than "primary".
pub fn label_for(window_minutes: Option<f64>, id: &str) -> String {
    match window_minutes {
        Some(m) if m > 0.0 => {
            if m < 60.0 {
                format!("{}m limit", m as i64)
            } else if m < 60.0 * 24.0 {
                format!("{}h limit", (m / 60.0) as i64)
            } else {
                match (m / (60.0 * 24.0)).round() as i64 {
                    7 => "Weekly limit".into(),
                    30 => "Monthly limit".into(),
                    d => format!("{d}d limit"),
                }
            }
        }
        _ if id == "primary" => "Current session".into(),
        _ => "Longer window".into(),
    }
}

pub fn num(v: Option<&Value>) -> Option<f64> {
    v.and_then(Value::as_f64)
}

/// `rate_limit.{primary_window,secondary_window}` to windows. `additional_rate_limits` and
/// `code_review_rate_limit` meter something else and stay out. The label comes from the length:
/// the primary window is not always five hours (a free plan has shown 30 days).
pub fn windows_from_usage(v: &Value, now: u64) -> Vec<LimitWindow> {
    let mut out = Vec::new();
    for (id, key) in [("primary", "primary_window"), ("secondary", "secondary_window")] {
        let Some(w) = v.pointer(&format!("/rate_limit/{key}")).filter(|x| x.is_object()) else { continue };
        let Some(pct) = num(w.get("used_percent")) else { continue };
        let resets_at = num(w.get("reset_at"))
            .map(|s| (s * 1000.0) as u64)
            .or_else(|| num(w.get("reset_after_seconds")).map(|s| now + (s * 1000.0) as u64));
        out.push(LimitWindow {
            id: id.into(),
            label: label_for(num(w.get("limit_window_seconds")).map(|s| s / 60.0), id),
            used: (pct / 100.0).clamp(0.0, 1.0),
            resets_at,
            ..Default::default()
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const WHAM: &str = include_str!("../../../tests/fixtures/vendors/codex-wham-usage.json");

    #[test]
    fn recorded_body_yields_the_monthly_primary_window() {
        let w = windows_from_usage(&serde_json::from_str(WHAM).unwrap(), 0);
        assert_eq!(w.len(), 1);
        assert_eq!((w[0].id.as_str(), w[0].label.as_str()), ("primary", "Monthly limit"));
        assert!((w[0].used - 1.0).abs() < 1e-9);
        assert_eq!(w[0].resets_at, Some(1_789_782_254_000));
    }

    #[test]
    fn an_empty_reply_is_no_windows_never_a_zero_ring() {
        let empty = serde_json::json!({"rate_limit": {"primary_window": null, "secondary_window": null}});
        assert!(windows_from_usage(&empty, 0).is_empty());
    }

    #[test]
    fn labels_follow_the_window_length() {
        assert_eq!(label_for(Some(300.0), "primary"), "5h limit");
        assert_eq!(label_for(Some(10080.0), "secondary"), "Weekly limit");
        assert_eq!(label_for(Some(45.0), "primary"), "45m limit");
        assert_eq!(label_for(Some(4320.0), "secondary"), "3d limit");
        assert_eq!(label_for(None, "secondary"), "Longer window");
    }
}
