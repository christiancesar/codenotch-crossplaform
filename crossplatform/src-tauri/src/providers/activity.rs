//! One 2-second loop asks every present provider what it is doing and reports only changes.

use super::{Activity, UsageProvider};
use crate::platform::{Platform, Processes};
use std::sync::Arc;
use std::time::Duration;

const INTERVAL: Duration = Duration::from_secs(2);
/// Presence (finding executables, reading credentials) once a minute is plenty
const PRESENCE_EVERY: u32 = 30;

pub fn start(providers: Vec<Arc<dyn UsageProvider>>, on_change: Arc<dyn Fn(&[Activity]) + Send + Sync>) {
    std::thread::Builder::new()
        .name("provider-activity".into())
        .spawn(move || {
            Platform.lower_current_thread_priority();
            let mut present = vec![false; providers.len()];
            let mut last: Vec<Activity> = Vec::new();
            for tick in 0u32.. {
                if tick % PRESENCE_EVERY == 0 {
                    present = providers.iter().map(|p| p.is_present()).collect();
                }
                let now: Vec<Activity> =
                    providers.iter().zip(&present).filter(|(_, on)| **on).flat_map(|(p, _)| p.activity()).collect();
                if now != last {
                    on_change(&now);
                    last = now;
                }
                std::thread::sleep(INTERVAL);
            }
        })
        .expect("failed to start the activity loop");
}
