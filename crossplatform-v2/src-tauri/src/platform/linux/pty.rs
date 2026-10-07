//! On Linux a plain pipe is enough: `agy` prints its normal output to a redirected stdout
//! (checked live). What still matters is the hard timeout and killing the whole process group.

use std::io::Read;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

extern "C" {
    // libc's kill(2); std has no process-group kill and the crate takes no libc dependency for one call
    fn kill(pid: i32, sig: i32) -> i32;
}
const SIGKILL: i32 = 9;

/// Kills the process group `pgid` (the child's own, from `process_group(0)`). A direct syscall,
/// not the `kill` program: no argument parsing between us and which processes die. Never for
/// pgid 0 or 1, which would mean our own group or every process we may signal.
fn kill_group(pgid: u32) {
    if pgid > 1 {
        if let Ok(pid) = i32::try_from(pgid) {
            // Negative pid: the whole group
            unsafe { kill(-pid, SIGKILL) };
        }
    }
}

pub fn run_captured(program: &Path, args: &[&str], cwd: Option<&Path>, timeout: Duration) -> Result<String, String> {
    if !program.is_file() {
        return Err(format!("Program not found: {}", program.display()));
    }
    let mut cmd = Command::new(program);
    cmd.args(args).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::null());
    // Own process group, so a timeout takes the CLI's children down with it
    cmd.process_group(0);
    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }
    let mut child = cmd.spawn().map_err(|e| format!("Cannot start {}: {e}", program.display()))?;

    // Drained on its own thread: a child that never exits must not block us past the timeout
    let out = child.stdout.take().expect("piped stdout");
    let reader = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = out.take(crate::platform::MAX_CAPTURE as u64 + 1).read_to_end(&mut buf);
        buf
    });

    let deadline = Instant::now() + timeout;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() >= deadline => {
                kill_group(child.id());
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("{} timed out after {timeout:?}", program.display()));
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(50)),
            Err(e) => return Err(format!("Cannot wait on {}: {e}", program.display())),
        }
    };
    let raw = reader.join().unwrap_or_default();
    if !status.success() {
        return Err(format!("{} failed with exit code {}", program.display(), status.code().unwrap_or(-1)));
    }
    if raw.len() > crate::platform::MAX_CAPTURE {
        return Err("output is too large".into());
    }
    Ok(String::from_utf8_lossy(&raw).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn captures_output_and_enforces_the_timeout() {
        let sh = Path::new("/bin/sh");
        let out = run_captured(sh, &["-c", "echo hello"], None, Duration::from_secs(5)).unwrap();
        assert_eq!(out, "hello\n");
        let t = Instant::now();
        let err = run_captured(sh, &["-c", "sleep 30"], None, Duration::from_millis(300)).unwrap_err();
        assert!(err.contains("timed out"));
        assert!(t.elapsed() < Duration::from_secs(5));
        assert!(run_captured(sh, &["-c", "exit 3"], None, Duration::from_secs(5)).unwrap_err().contains("exit code 3"));
    }

    #[test]
    fn a_timeout_kills_the_grandchildren_too() {
        // `sh` starts a `sleep` of its own; the group kill must take it down with the shell
        let dir = tempfile::tempdir().unwrap();
        let pidfile = dir.path().join("pid");
        let script = format!("sleep 30 & echo $! > {}; wait", pidfile.display());
        assert!(run_captured(Path::new("/bin/sh"), &["-c", &script], None, Duration::from_millis(500)).is_err());
        let pid = std::fs::read_to_string(&pidfile).unwrap().trim().to_string();
        std::thread::sleep(Duration::from_millis(100));
        // Gone, or a zombie awaiting its reaper
        let state = std::fs::read_to_string(format!("/proc/{pid}/stat")).unwrap_or_default();
        assert!(state.is_empty() || state.split_whitespace().nth(2) == Some("Z"), "{state}");
    }
}
