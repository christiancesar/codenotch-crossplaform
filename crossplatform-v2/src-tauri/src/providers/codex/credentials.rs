use crate::support::time::now_ms;
use std::path::PathBuf;

pub fn auth_path() -> Option<PathBuf> {
    super::codex_home().map(|h| h.join("auth.json"))
}

pub struct Credential {
    pub access_token: String,
    pub account_id: String,
    /// chatgpt_plan_type from the id_token (pro, plus, free...), a label only
    pub plan: Option<String>,
    /// The access token's exp has passed. The request is still sent, the server decides; this
    /// only changes the wording of a rejection.
    pub expired: bool,
}

/// The JWT's claims, unverified: used for a label and an expiry hint, never for trust.
fn jwt_claims(token: &str) -> Option<serde_json::Value> {
    let raw = crate::support::base64::decode(token.split('.').nth(1)?)?;
    serde_json::from_slice(&raw).ok()
}

pub fn load() -> Option<Credential> {
    from_json(&std::fs::read_to_string(auth_path()?).ok()?, now_ms())
}

fn from_json(text: &str, now: u64) -> Option<Credential> {
    let v: serde_json::Value = serde_json::from_str(text).ok()?;
    let tokens = v.get("tokens")?;
    let access_token = tokens.get("access_token")?.as_str()?.trim().to_string();
    let account_id = tokens.get("account_id")?.as_str()?.trim().to_string();
    if access_token.is_empty() || account_id.is_empty() {
        return None;
    }
    let expired = jwt_claims(&access_token)
        .and_then(|c| c.get("exp")?.as_f64())
        .map(|exp| exp * 1000.0 <= now as f64)
        .unwrap_or(false);
    let plan = tokens
        .get("id_token")
        .and_then(|x| x.as_str())
        .and_then(jwt_claims)
        .and_then(|c| c.get("https://api.openai.com/auth")?.get("chatgpt_plan_type")?.as_str().map(String::from));
    Some(Credential { access_token, account_id, plan, expired })
}

/// For doctor: no secrets.
pub fn probe() -> String {
    match load() {
        Some(c) => format!(
            "auth.json usable{}{}",
            if c.expired { " (access_token expired)" } else { "" },
            c.plan.map(|p| format!(", plan={p}")).unwrap_or_default()
        ),
        None if auth_path().map(|p| p.is_file()).unwrap_or(false) => "auth.json present but has no token".into(),
        None => "auth.json not found".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::from_json;
    use crate::support::base64::encode;

    fn jwt(claims: &str) -> String {
        format!("h.{}.s", encode(claims.as_bytes()).trim_end_matches('='))
    }

    #[test]
    fn reads_tokens_plan_and_expiry() {
        let id = jwt(r#"{"https://api.openai.com/auth":{"chatgpt_plan_type":"plus"}}"#);
        let access = jwt(r#"{"exp":100}"#);
        let text = format!(r#"{{"tokens":{{"access_token":"{access}","account_id":"acc","id_token":"{id}"}}}}"#);
        let c = from_json(&text, 99_000).unwrap();
        assert_eq!((c.account_id.as_str(), c.plan.as_deref(), c.expired), ("acc", Some("plus"), false));
        assert!(from_json(&text, 100_000).unwrap().expired);
    }

    #[test]
    fn a_missing_or_empty_field_means_not_signed_in() {
        assert!(from_json(r#"{"tokens":{"access_token":"a"}}"#, 0).is_none());
        assert!(from_json(r#"{"tokens":{"access_token":" ","account_id":"x"}}"#, 0).is_none());
    }
}
