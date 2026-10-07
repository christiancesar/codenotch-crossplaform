//! Subcommands, a frozen contract: `install-hooks`, `uninstall-hooks`, `autostart on|off`,
//! `doctor [deep]`. Anything else (including `--silent` from autostart) starts the app.

use crate::platform::{Autostart, Platform};

/// Some(exit message) when argv named a subcommand, None to start the app.
pub fn dispatch(args: &[String]) -> Option<()> {
    let dir = crate::storage::paths::config_dir();
    match args.get(1).map(String::as_str)? {
        "install-hooks" => report(&dir, crate::sessions::hooks_install::install()),
        "uninstall-hooks" => report(&dir, crate::sessions::hooks_install::uninstall()),
        "autostart" => report(
            &dir,
            match args.get(2).map(String::as_str) {
                Some("on") => Platform.enable(),
                Some("off") => Platform.disable(),
                _ => Err("usage: codenotch autostart on|off".into()),
            },
        ),
        "doctor" => {
            let out = if args.get(2).map(String::as_str) == Some("deep") { crate::diagnostics::run_deep() } else { crate::diagnostics::run() };
            println!("{out}");
            let _ = crate::storage::atomic::write(&dir.join("doctor.log"), out.as_bytes());
        }
        _ => return None,
    }
    Some(())
}

/// Printed and kept in install.log, so a double-clicked installer step leaves a trace.
fn report(dir: &std::path::Path, r: Result<String, String>) {
    let msg = match r {
        Ok(m) => format!("OK: {m}"),
        Err(e) => format!("FAILED: {e}"),
    };
    println!("{msg}");
    let _ = crate::storage::atomic::write(&dir.join("install.log"), msg.as_bytes());
}
