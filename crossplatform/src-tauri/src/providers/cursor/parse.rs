//! `{ billingCycleEnd, membershipType, isUnlimited, individualUsage: { plan: { totalPercentUsed,
//! apiPercentUsed, used, limit }, onDemand: { enabled, used, limit } } }`.
//! Cursor meters a percentage of the allowance: the dashboard's "Included usage · N% used" is
//! totalPercentUsed. On the free plan used/limit are always 0 (the allowance arrives as a bonus),
//! so reading those would report 10 % as 0 %. And 0 is a reading, not a gap.

use crate::providers::LimitWindow;
use crate::support::time::parse_iso;
use serde_json::Value;

fn pct(v: Option<&Value>) -> Option<f64> {
    v.and_then(Value::as_f64).map(|p| (p / 100.0).clamp(0.0, 1.0))
}

/// Windows, or none plus a note saying why (unlimited, or a plan with nothing to meter).
pub fn summary(v: &Value) -> (Vec<LimitWindow>, String) {
    let resets_at = parse_iso(v.get("billingCycleEnd"));
    let usage = v.get("individualUsage").cloned().unwrap_or(Value::Null);
    let plan = usage.get("plan").cloned().unwrap_or(Value::Null);
    let window = |id: &str, label: &str, used: f64| LimitWindow { id: id.into(), label: label.into(), used, resets_at, ..Default::default() };
    let mut out = Vec::new();
    if let Some(total) = pct(plan.get("totalPercentUsed")) {
        out.push(window("included", "Included usage", total));
    }
    if let Some(api) = pct(plan.get("apiPercentUsed")).filter(|a| *a > 0.0) {
        out.push(window("api", "API usage", api));
    }
    if let Some(od) = usage.get("onDemand") {
        let enabled = od.get("enabled").and_then(Value::as_bool).unwrap_or(false);
        let limit = od.get("limit").and_then(Value::as_f64).unwrap_or(0.0);
        if let (true, true, Some(u)) = (enabled, limit > 0.0, od.get("used").and_then(Value::as_f64)) {
            out.push(window("on_demand", "On demand", (u / limit).clamp(0.0, 1.0)));
        }
    }
    if !out.is_empty() {
        return (out, String::new());
    }
    let membership = v.get("membershipType").and_then(Value::as_str).unwrap_or("this");
    let note = if v.get("isUnlimited").and_then(Value::as_bool) == Some(true) {
        format!("Unlimited on the {membership} plan — nothing to meter")
    } else {
        format!("The {membership} plan has nothing for Cursor to meter yet")
    };
    (out, note)
}

#[cfg(test)]
mod tests {
    use super::summary;
    use serde_json::json;

    const USAGE_SUMMARY: &str = include_str!("../../../tests/fixtures/vendors/cursor_state.json");

    #[test]
    fn recorded_summary_is_official_with_all_three_windows() {
        let (w, note) = summary(&serde_json::from_str(USAGE_SUMMARY).unwrap());
        assert!(note.is_empty());
        let get = |id: &str| w.iter().find(|x| x.id == id).unwrap();
        assert!((get("included").used - 0.505).abs() < 1e-9);
        assert_eq!(get("included").label, "Included usage");
        assert!(get("included").resets_at.is_some() && !get("included").derived && get("included").count.is_none());
        assert!((get("api").used - 0.05).abs() < 1e-9);
        assert!((get("on_demand").used - 0.2).abs() < 1e-9);
    }

    #[test]
    fn unlimited_and_empty_plans_explain_themselves() {
        let unlimited = json!({"membershipType": "pro", "isUnlimited": true, "individualUsage": {"plan": {"used": 0, "limit": 0}}});
        let (w, note) = summary(&unlimited);
        assert!(w.is_empty() && note.contains("Unlimited"));
        let free = json!({"membershipType": "free", "isUnlimited": false, "individualUsage": {"plan": {"used": 0, "limit": 0}}});
        assert!(summary(&free).1.contains("nothing for Cursor to meter"));
    }

    #[test]
    fn zero_api_usage_is_left_out() {
        let v = json!({"individualUsage": {"plan": {"totalPercentUsed": 10.0, "apiPercentUsed": 0.0}}});
        let (w, _) = summary(&v);
        assert_eq!(w.iter().map(|x| x.id.as_str()).collect::<Vec<_>>(), vec!["included"]);
    }
}
