use crate::support::time::now_ms;

pub struct Credential {
    pub token: String,
    /// The file's own expiry says it is past. Only a hint for the note: Claude Code refreshes
    /// the token on its next use, and the server is the judge.
    pub expired: bool,
}

/// `~/.claude/.credentials.json` (older CLIs: `credentials.json`), the `claudeAiOauth` object or
/// the file's top level.
pub fn read() -> Option<Credential> {
    let home = dirs::home_dir()?;
    [".credentials.json", "credentials.json"]
        .into_iter()
        .find_map(|name| std::fs::read_to_string(home.join(".claude").join(name)).ok())
        .and_then(|text| from_json(&text, now_ms()))
}

fn from_json(text: &str, now: u64) -> Option<Credential> {
    let v: serde_json::Value = serde_json::from_str(text).ok()?;
    let oauth = v.get("claudeAiOauth").unwrap_or(&v);
    let token = oauth.get("accessToken")?.as_str()?.to_string();
    let expired = oauth.get("expiresAt").and_then(|x| x.as_f64()).map(|ms| (ms as u64) <= now).unwrap_or(false);
    Some(Credential { token, expired })
}

/// For doctor: never prints the token, only its length.
pub fn probe() -> String {
    match read() {
        Some(c) => format!(
            "credential: found (token {} chars, {})",
            c.token.len(),
            if c.expired { "expired — Claude Code refreshes it on its next use" } else { "valid" }
        ),
        None => "credential: ~/.claude/.credentials.json not found (needsAuth; signing in once with the Claude Code CLI creates it)".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::from_json;

    #[test]
    fn reads_the_nested_and_the_flat_shape() {
        let nested = r#"{"claudeAiOauth":{"accessToken":"t1","expiresAt":2000}}"#;
        let c = from_json(nested, 1000).unwrap();
        assert_eq!((c.token.as_str(), c.expired), ("t1", false));
        assert!(from_json(nested, 2000).unwrap().expired);
        assert_eq!(from_json(r#"{"accessToken":"t2"}"#, 0).unwrap().token, "t2");
        assert!(from_json(r#"{"other":1}"#, 0).is_none());
    }
}
