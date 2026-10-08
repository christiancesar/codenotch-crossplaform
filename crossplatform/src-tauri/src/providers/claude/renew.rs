//! Keeps ~/.claude/.credentials.json alive for someone who only uses the Claude desktop app.
//! Only the standalone `claude` CLI writes that file; the copy bundled in the desktop app keeps
//! its token in the desktop app's own store. Used through the desktop app alone, the file's token
//! expires about eight hours after the CLI last ran and every reading freezes. Starting the CLI
//! is what renews it, so we start it. Ported from upstream's Windows renewer (vinzdg/codenotch
//! #192, #228), itself the Windows twin of the macOS ClaudeTokenRefresher.

use super::credentials::{self, Credential};
use crate::platform::{Executables, Platform};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// Renew this close to expiry. Must stay under Claude Code's own five minutes: its start-up
/// renews only when now + 300 s >= expiresAt, so an earlier launch is a no-op judged a failure.
const MARGIN_MS: u64 = 4 * 60 * 1000;
const COOLDOWN_MS: u64 = 10 * 60 * 1000;
/// A token that did not renew is tried again, each wait twice the last, at most an hour apart:
/// one failure (asleep, offline, a slow start) must not freeze the ring until a terminal is opened
const RETRY_CAP_MS: u64 = 60 * 60 * 1000;
const TIMEOUT: Duration = Duration::from_secs(30);

/// The desktop app's own copies: renewing with them would change nothing in our file.
fn is_desktop_owned(p: &Path) -> bool {
    let s = p.to_string_lossy().to_ascii_lowercase().replace('/', "\\");
    s.contains("\\anthropicclaude\\") || s.contains("\\claude\\claude-code\\") || s.contains("\\windowsapps\\")
}

/// The standalone CLI: its own installer's location first, then the package managers', then PATH.
fn find_cli() -> Option<PathBuf> {
    let names = Platform.exe_names("claude");
    let mut dirs_: Vec<PathBuf> = Vec::new();
    if let Some(h) = dirs::home_dir() {
        dirs_.push(h.join(".local").join("bin"));
    }
    if let Some(d) = dirs::config_dir() {
        dirs_.push(d.join("npm"));
    }
    if let Some(d) = dirs::data_local_dir() {
        dirs_.push(d.join("pnpm"));
    }
    if let Some(h) = dirs::home_dir() {
        for sub in [".volta", ".npm-global", ".bun"] {
            dirs_.push(h.join(sub).join("bin"));
        }
    }
    dirs_.extend(std::env::var_os("PATH").map(|p| std::env::split_paths(&p).filter(|d| d.is_absolute()).collect::<Vec<_>>()).unwrap_or_default());
    dirs_.iter().flat_map(|d| names.iter().map(move |n| d.join(n))).find(|p| p.is_file() && !is_desktop_owned(p))
}

/// Whether a launch is worth making. Pure, so every branch is testable without a clock.
fn should_renew(expires_at: Option<u64>, now: u64, attempted_for: Option<u64>, last_attempt: Option<u64>, failures: u32) -> bool {
    // Never launch on a guess, nor while the CLI's own gate is still shut
    let Some(exp) = expires_at else { return false };
    if exp > now + MARGIN_MS {
        return false;
    }
    let Some(t) = last_attempt else { return true };
    let wait = if attempted_for == Some(exp) { retry_wait_ms(failures) } else { COOLDOWN_MS };
    now.saturating_sub(t) >= wait
}

fn retry_wait_ms(failures: u32) -> u64 {
    COOLDOWN_MS.saturating_mul(1u64 << failures.min(16)).min(RETRY_CAP_MS)
}

/// `claude -p` with a null stdin starts up (where it renews an aged token), then exits non-zero
/// for want of a prompt: no conversation, no transcript. Output goes nowhere, since a token could
/// in principle be echoed into it.
fn run(cli: &Path) -> std::io::Result<()> {
    use std::process::{Command, Stdio};
    let mut cmd = Command::new(cli);
    cmd.arg("-p").stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).current_dir(std::env::temp_dir());
    // Started by a hook from inside a Claude Code session, the app inherits that session's
    // variables: the child would use the host's auth and leave the file alone. CLAUDE_CONFIG_DIR
    // would point it at another account than the ~/.claude one we read.
    for (k, _) in std::env::vars_os() {
        let k = k.to_string_lossy();
        if k == "CLAUDECODE" || k == "CLAUDE_CONFIG_DIR" || k.starts_with("CLAUDE_CODE_") {
            cmd.env_remove(k.as_ref());
        }
    }
    Platform.hide_console(&mut cmd);
    let mut child = cmd.spawn()?;
    let deadline = Instant::now() + TIMEOUT;
    while child.try_wait()?.is_none() {
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    Ok(())
}

#[derive(Default)]
pub struct Renewer {
    attempted_for: Option<u64>,
    last_attempt: Option<u64>,
    /// Launches in a row that left `attempted_for` where it was
    failures: u32,
}

impl Renewer {
    /// Renews when the token is about to expire. Some(true) = the expiry moved. Judged on the
    /// outcome, never the exit status: refusing the empty prompt is a non-zero exit even when the
    /// renewal worked.
    pub fn maybe_renew(&mut self, cred: &Credential, now: u64) -> Option<bool> {
        if !should_renew(cred.expires_at, now, self.attempted_for, self.last_attempt, self.failures) {
            return None;
        }
        if self.attempted_for != cred.expires_at {
            self.failures = 0;
        }
        self.last_attempt = Some(now);
        self.attempted_for = cred.expires_at;
        self.failures = self.failures.saturating_add(1);
        let Some(cli) = find_cli() else {
            crate::diagnostics::log("claude: token about to expire and no standalone claude CLI found to renew it");
            return Some(false);
        };
        if let Err(e) = run(&cli) {
            crate::diagnostics::log(&format!("claude: token renewal could not start ({}): {e}", cli.display()));
            return Some(false);
        }
        let after = credentials::read().and_then(|c| c.expires_at);
        let renewed = matches!((after, cred.expires_at), (Some(a), Some(b)) if a > b);
        crate::diagnostics::log(&if renewed {
            format!("claude: token renewed via {}", cli.display())
        } else {
            format!("claude: ran {} but the token expiry did not move", cli.display())
        });
        Some(renewed)
    }
}

/// For doctor: which CLI a renewal would run.
pub fn probe() -> String {
    match find_cli() {
        Some(p) => format!("renewal CLI: {}", p.display()),
        None => "renewal CLI: none (install the standalone claude CLI; the desktop app's copy does not write the credential)".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXP: u64 = 1_000_000_000;

    #[test]
    fn renews_only_inside_the_margin() {
        assert!(!should_renew(None, EXP, None, None, 0), "never launch on a guess");
        assert!(!should_renew(Some(EXP), EXP - MARGIN_MS - 1, None, None, 0), "plenty of time left");
        assert!(should_renew(Some(EXP), EXP - MARGIN_MS, None, None, 0));
        assert!(should_renew(Some(EXP), EXP + 3_600_000, None, None, 0), "already expired still renews");
    }

    #[test]
    fn a_new_token_waits_out_the_cooldown() {
        let now = EXP + 1;
        assert!(!should_renew(Some(EXP + 5), now, Some(EXP), Some(now - 1000), 1));
        assert!(should_renew(Some(EXP + 5), now, Some(EXP), Some(now - COOLDOWN_MS), 1));
    }

    #[test]
    fn a_failed_token_is_retried_on_a_doubling_wait() {
        let now = EXP + 1;
        assert!(!should_renew(Some(EXP), now, Some(EXP), Some(now - COOLDOWN_MS), 1));
        assert!(should_renew(Some(EXP), now, Some(EXP), Some(now - 2 * COOLDOWN_MS), 1));
        assert!(!should_renew(Some(EXP), now, Some(EXP), Some(now - 2 * COOLDOWN_MS), 2), "the wait doubles");
        assert!(!should_renew(Some(EXP), now, Some(EXP), Some(now - RETRY_CAP_MS + 1), 30), "never a tight loop");
        assert!(should_renew(Some(EXP), now, Some(EXP), Some(now - RETRY_CAP_MS), 30), "never more than an hour apart");
    }

    #[test]
    fn the_desktop_apps_own_cli_is_refused() {
        assert!(is_desktop_owned(Path::new(r"C:\Users\u\AppData\Local\AnthropicClaude\app-1.2.3\claude.exe")));
        assert!(is_desktop_owned(Path::new(r"C:\Users\u\AppData\Roaming\Claude\claude-code\2.1.293\83cb\claude.exe")));
        assert!(!is_desktop_owned(Path::new(r"C:\Users\u\.local\bin\claude.exe")));
        assert!(!is_desktop_owned(Path::new("/home/u/.local/bin/claude")));
    }
}
