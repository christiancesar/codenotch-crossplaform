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

/// A plain read-only connection that sees the WAL (immutable would show the world as of the last
/// checkpoint, wrong for live state).
pub fn open_live(path: &Path) -> Option<Connection> {
    path.is_file().then(|| Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX).ok()).flatten()
}

fn mtime_ms(p: &Path) -> u64 {
    std::fs::metadata(p)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// A kept-open connection whose query runs again only when the database or its -wal changed.
/// Cursor's state.vscdb is over 2 GB; reopening it every 2 s for a scan made typing lag.
pub struct LiveQuery<T: Clone + Default> {
    path: std::path::PathBuf,
    conn: Option<Connection>,
    sig: Option<(u64, u64)>,
    last: T,
}

impl<T: Clone + Default> LiveQuery<T> {
    pub fn new(path: std::path::PathBuf) -> Self {
        LiveQuery { path, conn: None, sig: None, last: T::default() }
    }

    /// `query` returning None means it failed: the connection is dropped and reopened next time.
    pub fn get(&mut self, query: impl FnOnce(&Connection) -> Option<T>) -> T {
        let mut wal = self.path.as_os_str().to_owned();
        wal.push("-wal");
        let sig = (mtime_ms(&self.path), mtime_ms(Path::new(&wal)));
        if self.sig == Some(sig) {
            return self.last.clone();
        }
        self.sig = Some(sig);
        if self.conn.is_none() {
            self.conn = open_live(&self.path);
        }
        self.last = match self.conn.as_ref().and_then(query) {
            Some(v) => v,
            None => {
                self.conn = None;
                T::default()
            }
        };
        self.last.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn live_query_reruns_only_after_a_write() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("x.db");
        Connection::open(&path).unwrap().execute_batch("CREATE TABLE t(v INTEGER); INSERT INTO t VALUES (1);").unwrap();
        let mut q: LiveQuery<i64> = LiveQuery::new(path.clone());
        let runs = std::cell::Cell::new(0);
        let read = |q: &mut LiveQuery<i64>| q.get(|c| { runs.set(runs.get() + 1); c.query_row("SELECT SUM(v) FROM t", [], |r| r.get(0)).ok() });
        assert_eq!(read(&mut q), 1);
        assert_eq!(read(&mut q), 1);
        assert_eq!(runs.get(), 1);
        std::thread::sleep(std::time::Duration::from_millis(20));
        Connection::open(&path).unwrap().execute("INSERT INTO t VALUES (2)", []).unwrap();
        assert_eq!(read(&mut q), 3);
        assert_eq!(runs.get(), 2);
    }

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
