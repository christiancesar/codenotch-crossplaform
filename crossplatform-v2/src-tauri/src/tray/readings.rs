//! Which number each tray slot shows. The same rule as the notch ring, so the two never disagree.

use crate::config::Slot;
use crate::providers::{ProviderStatus, UsageSnapshot};

/// The fullest metered window as a whole percentage. A count window (requests today) has no
/// denominator, so its `used` is not a share of anything and taking a max over it would invent a
/// number.
pub fn headline_pct(s: &UsageSnapshot) -> Option<u32> {
    let top = s.windows.iter().filter(|w| w.count.is_none()).map(|w| w.used).fold(f64::NAN, f64::max);
    (!top.is_nan()).then(|| (top * 100.0).round().clamp(0.0, 100.0) as u32)
}

/// Empty or "top" means the fullest window. A pinned window the provider no longer reports falls
/// back to the fullest one, as the notch does, rather than leaving a dash.
pub fn for_slot(slot: &Slot, snap: &UsageSnapshot) -> Option<u32> {
    if snap.status == ProviderStatus::Absent {
        return None;
    }
    if slot.window.is_empty() || slot.window == "top" {
        return headline_pct(snap);
    }
    snap.windows
        .iter()
        .find(|w| w.id == slot.window && w.count.is_none())
        .map(|w| (w.used * 100.0).round().clamp(0.0, 100.0) as u32)
        .or_else(|| headline_pct(snap))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::LimitWindow;

    fn snap() -> UsageSnapshot {
        let w = |id: &str, used: f64, count: Option<i64>| LimitWindow { id: id.into(), label: id.into(), used, count, ..Default::default() };
        UsageSnapshot { status: ProviderStatus::Ok, windows: vec![w("session", 0.15, None), w("weekly_all", 0.42, None), w("requests", 0.99, Some(31))], ..Default::default() }
    }

    #[test]
    fn fullest_metered_window_pinned_window_and_fallback() {
        let slot = |w: &str| Slot { provider: "claude".into(), window: w.into() };
        assert_eq!(for_slot(&slot(""), &snap()), Some(42), "the count window is ignored");
        assert_eq!(for_slot(&slot("session"), &snap()), Some(15));
        assert_eq!(for_slot(&slot("gone"), &snap()), Some(42));
        assert_eq!(for_slot(&slot(""), &UsageSnapshot { status: ProviderStatus::Absent, ..snap() }), None);
        assert_eq!(headline_pct(&UsageSnapshot::default()), None);
    }
}
