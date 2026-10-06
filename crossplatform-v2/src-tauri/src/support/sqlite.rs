//! Opening another app's SQLite database without disturbing it.

use rusqlite::{Connection, OpenFlags};
use std::path::Path;

/// `mode=ro` first: it sees what the app just wrote into its WAL. If that fails, or opens but
/// cannot query (the app has exited and the -shm sidecar is gone), `immutable=1`: by then the WAL
/// is checkpointed, so ignoring it costs nothing. `probe_table` is queried to tell the two apart.
pub fn open_ro(path: &Path, probe_table: &str) -> Option<Connection> {
    if !path.is_file() {
        return None;
    }
    if let Ok(c) = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX) {
        let probe = format!("SELECT 1 FROM {probe_table} LIMIT 1");
        if c.prepare(&probe).and_then(|mut s| s.query([]).map(|_| ())).is_ok() {
            return Some(c);
        }
    }
    Connection::open_with_flags(
        immutable_uri(path),
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .ok()
}

/// Only the URI form takes `immutable=1`. A Windows path becomes file:///C:/... with / separators.
fn immutable_uri(path: &Path) -> String {
    let p = path.to_string_lossy().replace('\\', "/");
    format!("file:///{}?immutable=1", p.trim_start_matches('/').replace('#', "%23").replace('?', "%3F"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uri_escapes_and_normalizes_paths() {
        assert_eq!(immutable_uri(Path::new("/home/a b/x#1.db")), "file:///home/a b/x%231.db?immutable=1");
        assert_eq!(immutable_uri(Path::new(r"C:\Users\x\state.vscdb")), "file:///C:/Users/x/state.vscdb?immutable=1");
    }

    #[test]
    fn falls_back_to_immutable_when_the_wal_sidecar_cannot_be_created() {
        // The editor-has-exited case: WAL checkpointed, and a read-only directory where mode=ro
        // would have to recreate -shm. Directory modes mean nothing on Windows or to root.
        if !cfg!(unix) {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.vscdb");
        {
            let c = Connection::open(&path).unwrap();
            c.execute_batch("CREATE TABLE ItemTable(key TEXT PRIMARY KEY, value TEXT); INSERT INTO ItemTable VALUES ('k','v'); PRAGMA journal_mode=WAL; PRAGMA wal_checkpoint(TRUNCATE);").unwrap();
        }
        let original = std::fs::metadata(dir.path()).unwrap().permissions();
        let mut ro = original.clone();
        ro.set_readonly(true);
        std::fs::set_permissions(dir.path(), ro).unwrap();
        let blocked = std::fs::File::create(dir.path().join(".probe")).is_err();
        let conn = open_ro(&path, "ItemTable");
        std::fs::set_permissions(dir.path(), original).unwrap();
        let conn = conn.expect("one of the two modes opens it");
        if blocked {
            let v: String = conn.query_row("SELECT value FROM ItemTable", [], |r| r.get(0)).unwrap();
            assert_eq!(v, "v");
        }
    }
}
