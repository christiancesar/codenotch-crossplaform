//! On Linux a plain pipe is enough: `agy` prints its normal output to a redirected stdout
//! (checked live). What still matters is the hard timeout and killing the whole process group.

use std::io::Read;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

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
                // Negative pid: the whole group
                let _ = Command::new("kill").args(["-KILL", &format!("-{}", child.id())]).status();
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
}
