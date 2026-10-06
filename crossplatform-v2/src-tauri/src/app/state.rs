use crate::config::Config;
use std::path::PathBuf;
use std::sync::Mutex;

pub struct AppState {
    pub dir: PathBuf,
    pub config: Mutex<Config>,
}

impl AppState {
    pub fn load() -> AppState {
        let dir = crate::storage::paths::config_dir();
        let loaded = crate::config::load(&dir);
        // Only a missing file is written at startup (the hook reads the port from it). v0.3 saved
        // unconditionally here, which put defaults over a file it had failed to read.
        if loaded.missing {
            if let Err(e) = crate::config::save(&dir, &loaded.config) {
                crate::diagnostics::log(&format!("config: first save failed: {e}"));
            }
        }
        AppState { dir, config: Mutex::new(loaded.config) }
    }
}
