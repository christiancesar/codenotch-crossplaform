//! Thin #[tauri::command] adapters. Each one reads or updates state, calls one module function
//! and returns; no logic of its own.

pub mod spike;
pub mod system;
