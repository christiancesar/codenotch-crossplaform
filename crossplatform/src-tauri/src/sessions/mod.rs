//! Claude Code session tracking. Two sources feed one store: the hook events codenotch-hook
//! posts (exact), and inference from transcript appends (the desktop app, whose hooks do not
//! fire). Nothing here knows Tauri; `app/` applies the events and broadcasts changes.

pub mod hook_server;
pub mod hooks_install;
pub mod store;
pub mod sweep;
pub mod watcher;

pub use store::{HookEvent, SessionsSnapshot, Source, Store};
