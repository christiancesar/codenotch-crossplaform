//! The user's settings, `config.json`. Its shape is frozen for compatibility with v0.3 installs;
//! see docs/phase0-compat-gate.md.

mod migrate;
mod model;
mod stored;

pub use model::{Config, Slot, Theme, TrayMode, SCALE_MAX, SCALE_MIN};

use crate::storage::versioned::{self, Loaded};
use std::path::Path;

pub struct LoadResult {
    pub config: Config,
    /// True only when there was no file at all. The caller writes defaults then, so the hook can
    /// read the port; a file that was there but unreadable is never written over.
    pub missing: bool,
    /// Where an unreadable file was moved, so the user can be told
    pub quarantined: Option<std::path::PathBuf>,
}

pub fn load(dir: &Path) -> LoadResult {
    match versioned::load::<stored::StoredConfig>(&dir.join(crate::storage::paths::CONFIG), migrate::MIGRATIONS) {
        Loaded::Ok(s) => LoadResult { config: s.into(), missing: false, quarantined: None },
        Loaded::Missing => LoadResult { config: Config::default(), missing: true, quarantined: None },
        Loaded::Quarantined(p) => LoadResult { config: Config::default(), missing: false, quarantined: Some(p) },
    }
}

pub fn save(dir: &Path, config: &Config) -> std::io::Result<()> {
    let stored = stored::StoredConfig::from(config.clone());
    versioned::save(&dir.join(crate::storage::paths::CONFIG), &stored, migrate::MIGRATIONS)
}

#[cfg(test)]
mod tests;
