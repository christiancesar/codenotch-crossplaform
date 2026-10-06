//! The typed IPC contract: every command and event the webview can reach is registered here,
//! and `src/libs/ipc/bindings.ts` is generated from this list.

use tauri_specta::{collect_commands, collect_events, Builder};

pub fn builder() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new()
        .commands(collect_commands![
            crate::commands::system::app_version,
            crate::commands::spike::spike_echo,
        ])
        .events(collect_events![crate::app::events::SpikeTick])
        // Epoch-ms timestamps and counters, all far below 2^53, so `number` is exact for them
        .dangerously_cast_bigints_to_number()
}

pub fn export(builder: &Builder<tauri::Wry>) -> Result<(), String> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../src/libs/ipc/bindings.ts");
    builder
        .export(specta_typescript::Typescript::default(), path)
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    /// Regenerates the bindings on every `cargo test`; CI then fails on a diff, so a Rust change
    /// that alters the contract cannot land without the matching bindings.ts.
    #[test]
    fn bindings_export() {
        super::export(&super::builder()).unwrap();
    }
}
