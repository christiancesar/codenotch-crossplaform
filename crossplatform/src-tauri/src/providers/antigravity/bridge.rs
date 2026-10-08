//! The IDE's local language server. Its command line carries `--csrf_token <t>`; its port is
//! random at runtime and written nowhere, so it comes from the listening table. It opens two
//! ports and only one answers this RPC, so each is tried. The reply reports what remains.

use crate::platform::{Platform, Processes};
use crate::providers::LimitWindow;
use crate::support::time::parse_iso;
use std::sync::Arc;
use std::time::Duration;

const SERVICE: &str = "/exa.language_server_pb.LanguageServerService/RetrieveUserQuotaSummary";
/// Antigravity sits on the Codeium stack; the header name never changed
const CSRF_HEADER: &str = "x-codeium-csrf-token";

#[derive(Clone, Debug, PartialEq)]
pub struct Endpoint {
    pub ports: Vec<u16>,
    pub csrf: String,
}

fn flag_value(line: &str, flag: &str) -> Option<String> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    let i = parts.iter().position(|p| *p == flag)?;
    parts.get(i + 1).map(|s| s.trim_matches('"').to_string())
}

pub fn discover() -> Option<Endpoint> {
    let (pid, cmdline) = Platform.command_lines("language_server").into_iter().find(|(_, c)| c.contains("--csrf_token"))?;
    let csrf = flag_value(&cmdline, "--csrf_token")?;
    let ports = Platform.listening_ports(pid);
    (!ports.is_empty()).then_some(Endpoint { ports, csrf })
}

/// The self-signed certificate is accepted for 127.0.0.1 alone; never used for a public request.
fn loopback_agent() -> Option<ureq::Agent> {
    let tls = native_tls::TlsConnector::builder().danger_accept_invalid_certs(true).danger_accept_invalid_hostnames(true).build().ok()?;
    Some(ureq::AgentBuilder::new().tls_connector(Arc::new(tls)).timeout(Duration::from_secs(10)).build())
}

pub fn quota(ep: &Endpoint) -> Result<Vec<LimitWindow>, String> {
    let agent = loopback_agent().ok_or("TLS setup failed")?;
    let mut last = String::from("no port answered");
    for port in &ep.ports {
        // forceRefresh, or the server answers from its own cache
        let res = agent
            .post(&format!("https://127.0.0.1:{port}{SERVICE}"))
            .set("Content-Type", "application/json")
            .set(CSRF_HEADER, &ep.csrf)
            .send_string(r#"{"forceRefresh":true}"#);
        match res {
            Ok(r) => match r.into_json::<serde_json::Value>() {
                Ok(v) => {
                    let w = windows(&v);
                    if !w.is_empty() {
                        return Ok(w);
                    }
                    last = format!("port {port}: no recognisable groups");
                }
                Err(e) => last = format!("port {port}: {e}"),
            },
            Err(ureq::Error::Status(code, _)) => last = format!("port {port}: HTTP {code}"),
            Err(e) => last = format!("port {port}: {e}"),
        }
    }
    Err(last)
}

/// `{response:{groups:[{displayName, buckets:[{bucketId, displayName, remainingFraction,
/// resetTime}]}]}`. Flipped to fraction used here so the view never learns the difference; the
/// label is the group's, buckets only ever say "Weekly Limit Remaining".
pub fn windows(v: &serde_json::Value) -> Vec<LimitWindow> {
    let mut out = Vec::new();
    for g in v.pointer("/response/groups").and_then(|g| g.as_array()).into_iter().flatten() {
        let gname = g.get("displayName").and_then(|x| x.as_str());
        for b in g.get("buckets").and_then(|b| b.as_array()).into_iter().flatten() {
            let Some(rem) = b.get("remainingFraction").and_then(|x| x.as_f64()).filter(|r| (0.0..=1.0).contains(r)) else { continue };
            out.push(LimitWindow {
                id: b.get("bucketId").and_then(|x| x.as_str()).or(gname).unwrap_or("quota").to_string(),
                label: gname.or(b.get("displayName").and_then(|x| x.as_str())).unwrap_or("Usage").to_string(),
                used: (1.0 - rem).clamp(0.0, 1.0),
                resets_at: parse_iso(b.get("resetTime")),
                ..Default::default()
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recorded_bridge_reply_parses_to_two_windows() {
        let text = include_str!("../../../tests/fixtures/vendors/antigravity/agy-bridge.json");
        let w = windows(&serde_json::from_str(text).unwrap());
        assert_eq!(w.len(), 2, "{w:?}");
        assert_eq!((w[0].id.as_str(), w[0].label.as_str()), ("gemini-weekly", "Gemini · Weekly"));
        assert!((w[0].used - 0.06).abs() < 0.001 && w[0].resets_at.is_some());
        assert_eq!((w[1].id.as_str(), w[1].label.as_str()), ("claude-gpt-five-hour", "Claude/GPT · 5h"));
        assert!((w[1].used - 0.22).abs() < 0.001);
    }

    #[test]
    fn csrf_token_comes_from_the_command_line() {
        let cmd = r#"/opt/ag/language_server_linux_x64 --https_server_port 0 --csrf_token "abc-123" --x"#;
        assert_eq!(flag_value(cmd, "--csrf_token").as_deref(), Some("abc-123"));
        assert_eq!(flag_value(cmd, "--missing"), None);
    }
}
