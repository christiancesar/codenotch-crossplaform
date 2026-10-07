//! `<config dir>/Cursor/User/globalStorage/state.vscdb`, table `ItemTable(key, value)`:
//! `cursorAuth/accessToken` + `cursorAuth/stripeMembershipAuthId` make the cookie
//! `WorkosCursorSessionToken=<authId>::<token>`; `cursorAuth/stripeMembershipType` is the plan.

use crate::support::sqlite::open_ro;
use std::path::{Path, PathBuf};

pub fn store_path() -> Option<PathBuf> {
    dirs::config_dir().map(|c| c.join("Cursor").join("User").join("globalStorage").join("state.vscdb"))
}

pub struct Creds {
    pub cookie: String,
    pub plan: Option<String>,
}

fn item(conn: &rusqlite::Connection, key: &str) -> Option<String> {
    conn.query_row("SELECT value FROM ItemTable WHERE key = ?1", [key], |r| r.get::<_, String>(0))
        .ok()
        .filter(|s| !s.is_empty())
}

pub fn read_at(path: &Path) -> Option<Creds> {
    let conn = open_ro(path, "ItemTable")?;
    let token = item(&conn, "cursorAuth/accessToken")?;
    let auth_id = item(&conn, "cursorAuth/stripeMembershipAuthId")?;
    Some(Creds {
        cookie: format!("WorkosCursorSessionToken={auth_id}::{token}"),
        plan: item(&conn, "cursorAuth/stripeMembershipType"),
    })
}

/// For doctor: no secret values.
pub fn probe() -> String {
    let Some(p) = store_path() else { return "Cursor: cannot locate the config directory".into() };
    if !p.is_file() {
        return format!("Cursor: {} not found (not installed, or not signed in)", p.display());
    }
    match read_at(&p) {
        Some(c) => format!("Cursor: session borrowed (cookie {} chars, plan={})", c.cookie.len(), c.plan.unwrap_or_else(|| "?".into())),
        None => format!("Cursor: {} exists but cursorAuth/* could not be read (editor not signed in, or SQLite failed to open)", p.display()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const STATE_SQL: &str = include_str!("../../../tests/fixtures/vendors/cursor_state.sql");

    #[test]
    fn cookie_is_assembled_from_the_recorded_state() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.vscdb");
        rusqlite::Connection::open(&path).unwrap().execute_batch(STATE_SQL).unwrap();
        let c = read_at(&path).expect("fixture has both auth fields");
        assert_eq!(c.cookie, "WorkosCursorSessionToken=test-auth-id-redacted::test-access-token-redacted");
        assert_eq!(c.plan.as_deref(), Some("free"));
    }

    #[test]
    fn the_store_lives_under_the_config_directory() {
        let p = store_path().unwrap();
        assert!(p.starts_with(dirs::config_dir().unwrap()));
        assert!(p.ends_with("Cursor/User/globalStorage/state.vscdb"));
    }
}
