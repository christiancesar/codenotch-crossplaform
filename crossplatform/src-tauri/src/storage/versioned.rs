//! Versioned JSON files. A file carries `"schema": N`; no key means 1, the v0.3 format. Loading
//! reads a raw Value, runs the migrations from its version up, then deserializes.

use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use std::path::{Path, PathBuf};

pub const SCHEMA_KEY: &str = "schema";

#[derive(Debug)]
pub enum Loaded<T> {
    /// No file: first run, defaults are safe to save.
    Missing,
    Ok(T),
    /// The file existed but could not be read. It was moved aside, never overwritten.
    Quarantined(PathBuf),
}

/// `migrations[i]` upgrades a document from schema `i + 1` to `i + 2`.
pub type Migration = fn(Value) -> Value;

pub fn load<T: DeserializeOwned>(path: &Path, migrations: &[Migration]) -> Loaded<T> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Loaded::Missing,
        Err(_) => return Loaded::Quarantined(quarantine(path)),
    };
    match parse(&text, migrations) {
        Some(value) => Loaded::Ok(value),
        None => Loaded::Quarantined(quarantine(path)),
    }
}

pub fn parse<T: DeserializeOwned>(text: &str, migrations: &[Migration]) -> Option<T> {
    let mut doc = serde_json::from_str::<Value>(text).ok()?;
    if !doc.is_object() {
        return None;
    }
    let from = doc.get(SCHEMA_KEY).and_then(Value::as_u64).unwrap_or(1).max(1) as usize;
    for migrate in migrations.iter().skip(from - 1) {
        doc = migrate(doc);
    }
    if let Some(obj) = doc.as_object_mut() {
        obj.remove(SCHEMA_KEY);
    }
    serde_json::from_value(doc).ok()
}

/// Writes `value` stamped with the current schema (`migrations.len() + 1`).
pub fn save<T: Serialize>(path: &Path, value: &T, migrations: &[Migration]) -> std::io::Result<()> {
    let mut doc = serde_json::to_value(value)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    if let Some(obj) = doc.as_object_mut() {
        obj.insert(SCHEMA_KEY.into(), Value::from(migrations.len() as u64 + 1));
    }
    let text = serde_json::to_string_pretty(&doc)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    super::atomic::write(path, text.as_bytes())
}

/// Renames to `<name>.unreadable-<ms>` and logs it. If even the rename fails, the original stays
/// where it is, which is still better than writing defaults over it.
fn quarantine(path: &Path) -> PathBuf {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("file");
    let aside = path.with_file_name(format!("{name}.unreadable-{}", crate::support::time::now_ms()));
    match std::fs::rename(path, &aside) {
        Ok(()) => {
            crate::diagnostics::log(&format!("storage: {} could not be read, moved to {}", path.display(), aside.display()));
            aside
        }
        Err(e) => {
            crate::diagnostics::log(&format!("storage: {} could not be read and could not be moved aside: {e}", path.display()));
            path.to_path_buf()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq)]
    struct Doc {
        a: u32,
        #[serde(default)]
        b: u32,
    }

    fn add_b(mut v: Value) -> Value {
        v["b"] = Value::from(v["a"].as_u64().unwrap_or(0) * 10);
        v
    }

    #[test]
    fn missing_file_is_missing() {
        let dir = tempfile::tempdir().unwrap();
        assert!(matches!(load::<Doc>(&dir.path().join("x.json"), &[]), Loaded::Missing));
    }

    #[test]
    fn unversioned_file_runs_every_migration_and_saved_file_runs_none() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("x.json");
        std::fs::write(&p, r#"{"a": 2}"#).unwrap();
        let Loaded::Ok(doc) = load::<Doc>(&p, &[add_b]) else { panic!() };
        assert_eq!(doc, Doc { a: 2, b: 20 });

        save(&p, &Doc { a: 3, b: 7 }, &[add_b]).unwrap();
        assert!(std::fs::read_to_string(&p).unwrap().contains(r#""schema": 2"#));
        let Loaded::Ok(doc) = load::<Doc>(&p, &[add_b]) else { panic!() };
        assert_eq!(doc, Doc { a: 3, b: 7 });
    }

    #[test]
    fn unreadable_file_is_moved_aside_with_its_bytes_intact() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("config.json");
        std::fs::write(&p, r#"{ "a": 1, "b""#).unwrap();
        let Loaded::Quarantined(aside) = load::<Doc>(&p, &[]) else { panic!() };
        assert!(!p.exists());
        assert_eq!(std::fs::read_to_string(aside).unwrap(), r#"{ "a": 1, "b""#);
    }
}
