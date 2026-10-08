use crate::config::Config;
use crate::glyphs::Glyph;
use crate::notch::NotchState;
use crate::providers::scheduler::Handle;
use crate::providers::{Activity, ProviderId, UsageProvider, UsageSnapshot};
use crate::sessions::Store;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct ProviderSlot {
    pub provider: Arc<dyn UsageProvider>,
    pub handle: Handle,
}

pub struct AppState {
    pub dir: PathBuf,
    pub config: Mutex<Config>,
    pub sessions: Arc<Mutex<Store>>,
    /// In display order
    pub providers: Vec<ProviderSlot>,
    pub activity: Mutex<Vec<Activity>>,
    pub glyphs: Mutex<HashMap<String, Glyph>>,
    pub notch: Arc<NotchState>,
}

impl AppState {
    pub fn snapshot(&self, id: ProviderId) -> UsageSnapshot {
        self.providers.iter().find(|p| p.provider.id() == id).map(|p| p.handle.snapshot.lock().unwrap().clone()).unwrap_or_default()
    }

    /// Saves after a change; a failed write is logged, never fatal.
    pub fn update_config<T>(&self, change: impl FnOnce(&mut Config) -> T) -> T {
        let mut c = self.config.lock().unwrap();
        let out = change(&mut c);
        if let Err(e) = crate::config::save(&self.dir, &c) {
            crate::diagnostics::log(&format!("config: save failed: {e}"));
        }
        out
    }
}
