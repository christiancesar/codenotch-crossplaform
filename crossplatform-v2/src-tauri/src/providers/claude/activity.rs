//! Cloud sessions in the Claude desktop app leave no local transcript. While output streams, the
//! app's network service process keeps receiving data, so a sustained socket rate above the
//! threshold means "streaming". Inferred, and labelled as such.

use crate::platform::{Platform, Processes};
use crate::providers::{Activity, ActivityState, ProviderId};

/// The idle heartbeat is far below this, streaming far above
const RATE_BPS: f64 = 2_500.0;
/// Tool calls leave 2-4 s gaps with no traffic; holding avoids flicker
const HOLD_MS: u64 = 10_000;
const PID_TTL_MS: u64 = 5 * 60_000;
/// A miss is cached too, or the process lookup (PowerShell on Windows) would run every tick
const MISS_TTL_MS: u64 = 60_000;

#[derive(Default)]
pub struct NetProbe {
    /// The Electron network service child: all socket traffic goes through it, so the IOCTL noise
    /// of the GPU and renderer processes stays out
    pid: Option<u32>,
    looked_up_at: u64,
    sample: Option<(u64, u64)>,
    hits: u32,
    last_active: u64,
}

impl NetProbe {
    fn net_pid(&mut self, now: u64) -> Option<u32> {
        let fresh = now.saturating_sub(self.looked_up_at) < if self.pid.is_some() { PID_TTL_MS } else { MISS_TTL_MS };
        if self.looked_up_at != 0 && fresh {
            return self.pid;
        }
        self.looked_up_at = now;
        self.pid = Platform
            .command_lines("claude")
            .into_iter()
            .find(|(_, cmd)| cmd.contains("network.mojom.NetworkService"))
            .map(|(pid, _)| pid);
        self.pid
    }

    pub fn read(&mut self, now: u64) -> Vec<Activity> {
        let Some(pid) = self.net_pid(now) else { return vec![] };
        let Some((other, _)) = Platform.io_counters(pid) else {
            self.pid = None;
            return vec![];
        };
        self.observe(now, other)
    }

    /// Two samples in a row above the threshold (about 4 s); a single spike (heartbeat, sync)
    /// does not count.
    fn observe(&mut self, now: u64, other: u64) -> Vec<Activity> {
        let rate = match self.sample {
            Some((at, prev)) if now > at && other >= prev => (other - prev) as f64 / ((now - at) as f64 / 1000.0),
            _ => 0.0,
        };
        self.sample = Some((now, other));
        if rate >= RATE_BPS {
            self.hits += 1;
            if self.hits >= 2 {
                self.last_active = now;
            }
        } else {
            self.hits = 0;
        }
        if self.last_active > 0 && now.saturating_sub(self.last_active) <= HOLD_MS {
            vec![Activity {
                provider: ProviderId::Claude,
                state: ActivityState::Busy,
                name: "Claude".into(),
                detail: "Streaming (network)".into(),
                since: self.last_active,
            }]
        } else {
            vec![]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn needs_a_sustained_rate_and_holds_through_short_gaps() {
        let mut p = NetProbe::default();
        assert!(p.observe(0, 0).is_empty());
        assert!(p.observe(2_000, 10_000).is_empty(), "one spike is not streaming");
        assert_eq!(p.observe(4_000, 20_000).len(), 1);
        assert_eq!(p.observe(6_000, 20_000).len(), 1, "held through a quiet gap");
        assert!(p.observe(4_000 + HOLD_MS + 1, 20_000).is_empty());
    }
}
