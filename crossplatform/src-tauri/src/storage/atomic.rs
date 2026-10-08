use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

/// Temp file in the same directory, fsync, rename. A crash mid-write leaves either the old file
/// or the new one, never a truncated one that the next start would have to throw away.
pub fn write(dest: &Path, data: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    let name = dest.file_name().and_then(|n| n.to_str()).unwrap_or("file");
    let temp = dest.with_file_name(format!(
        "{name}.tmp.{}.{}.{id}",
        std::process::id(),
        crate::support::time::now_ms()
    ));

    let result = (|| {
        let mut f = std::fs::OpenOptions::new().write(true).create_new(true).open(&temp)?;
        f.write_all(data)?;
        f.sync_all()?;
        std::fs::rename(&temp, dest)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temp);
    }
    result
}

#[cfg(test)]
mod tests {
    #[test]
    fn replaces_the_file_and_leaves_no_temp_behind() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("sub/config.json");
        super::write(&p, b"one").unwrap();
        super::write(&p, b"two").unwrap();
        assert_eq!(std::fs::read(&p).unwrap(), b"two");
        let names: Vec<_> = std::fs::read_dir(p.parent().unwrap()).unwrap().collect();
        assert_eq!(names.len(), 1);
    }
}
