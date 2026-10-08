//! Upgrades v0.3's `config::load()` used to apply inline on every start, as versioned steps.

use crate::storage::versioned::Migration;
use serde_json::{json, Value};

pub const MIGRATIONS: &[Migration] = &[v1_to_v2];

/// v1 is every v0.3 file (no `schema` key).
fn v1_to_v2(mut doc: Value) -> Value {
    let Some(obj) = doc.as_object_mut() else { return doc };

    // A file that predates `tray_mode` keeps the plain mark it already had; only a machine with no
    // config at all gets the numbers icon. Upgrading never changes anyone's icon unasked.
    obj.entry("tray_mode").or_insert_with(|| json!("off"));

    // Before slots, the tray and the notch took a plain provider list, each provider showing its
    // fullest window. That is exactly a slot with an empty `window`.
    let tray_default = json!(["claude", "codex"]);
    let tray = slots_from(obj.get("tray_providers").unwrap_or(&tray_default));
    fill_if_empty(obj, "tray_slots", tray);
    let notch = slots_from(obj.get("notch_providers").unwrap_or(&json!([])));
    fill_if_empty(obj, "notch_slots", notch);
    doc
}

fn slots_from(providers: &Value) -> Value {
    let list = providers.as_array().cloned().unwrap_or_default();
    Value::Array(
        list.into_iter()
            .filter_map(|p| p.as_str().map(|p| json!({ "provider": p, "window": "" })))
            .collect(),
    )
}

fn fill_if_empty(obj: &mut serde_json::Map<String, Value>, key: &str, slots: Value) {
    let empty = obj.get(key).and_then(Value::as_array).map(|a| a.is_empty()).unwrap_or(true);
    if empty {
        obj.insert(key.into(), slots);
    }
}
