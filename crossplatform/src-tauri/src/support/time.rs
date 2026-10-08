use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Sleeps in one-second steps so a refresh request (the flag set to true) cuts the wait short.
/// The flag is consumed, so one request wakes one sleep.
pub fn sleep_interruptible(total_secs: u64, wake: &AtomicBool) {
    for _ in 0..total_secs {
        if wake.swap(false, Ordering::Relaxed) {
            return;
        }
        std::thread::sleep(Duration::from_secs(1));
    }
}

/// RFC 3339 string to epoch ms; vendors send these for reset times.
pub fn parse_iso(v: Option<&serde_json::Value>) -> Option<u64> {
    v.and_then(|x| x.as_str())
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.timestamp_millis().max(0) as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_iso_reads_offsets_and_rejects_garbage() {
        let v = serde_json::json!("2026-10-06T12:00:00+02:00");
        assert_eq!(parse_iso(Some(&v)), Some(1_791_280_800_000));
        assert_eq!(parse_iso(Some(&serde_json::json!("soon"))), None);
        assert_eq!(parse_iso(None), None);
    }

    #[test]
    fn a_pending_wake_ends_the_sleep_at_once() {
        let wake = AtomicBool::new(true);
        let t = std::time::Instant::now();
        sleep_interruptible(30, &wake);
        assert!(t.elapsed() < Duration::from_millis(500));
        assert!(!wake.load(Ordering::Relaxed));
    }
}
