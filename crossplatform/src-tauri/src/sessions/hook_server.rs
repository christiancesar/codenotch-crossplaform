//! Receives codenotch-hook's `POST /event?e=<event>&ppid=<pid>` on 127.0.0.1 (a frozen route),
//! with the Claude Code hook's stdin JSON as the body. Lenient: no missing field is an error.

use super::{HookEvent, Source};
use std::io::Read;
use std::sync::Arc;

pub type Sink = Arc<dyn Fn(HookEvent) + Send + Sync>;

/// Binds and serves on its own thread. A port already taken is logged, not fatal: the notch
/// still shows usage without sessions.
pub fn start(port: u16, sink: Sink) {
    std::thread::Builder::new()
        .name("hook-server".into())
        .spawn(move || {
            let server = match tiny_http::Server::http(("127.0.0.1", port)) {
                Ok(s) => s,
                Err(e) => {
                    crate::diagnostics::log(&format!("hook server: cannot bind 127.0.0.1:{port}: {e}"));
                    return;
                }
            };
            for mut req in server.incoming_requests() {
                let url = req.url().to_string();
                let mut body = String::new();
                let _ = req.as_reader().take(256 * 1024).read_to_string(&mut body);
                if url.starts_with("/event") {
                    sink(parse(&url, &body));
                }
                let _ = req.respond(tiny_http::Response::from_string("ok"));
            }
        })
        .expect("failed to start the hook server thread");
}

fn query_param(url: &str, key: &str) -> String {
    let q = url.split_once('?').map(|(_, q)| q).unwrap_or("");
    q.split('&').find_map(|pair| pair.split_once('=').filter(|(k, _)| *k == key).map(|(_, v)| v.to_string())).unwrap_or_default()
}

pub fn parse(url: &str, body: &str) -> HookEvent {
    let v: serde_json::Value = serde_json::from_str(body).unwrap_or(serde_json::Value::Null);
    let s = |k: &str| v.get(k).and_then(|x| x.as_str()).unwrap_or("").to_string();
    let id = s("session_id");
    HookEvent {
        e: query_param(url, "e"),
        session_id: if id.is_empty() { "unknown".into() } else { id },
        ppid: query_param(url, "ppid").parse().unwrap_or(0),
        cwd: s("cwd"),
        prompt: s("prompt"),
        message: s("message"),
        tool_name: s("tool_name"),
        // tool_input.command (Bash and friends) feeds the "last action" line
        tool_cmd: v.pointer("/tool_input/command").and_then(|c| c.as_str()).unwrap_or("").to_string(),
        model: s("model"),
        src: Source::Hook,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_hook_stdin_and_the_query() {
        let body = r#"{"session_id":"s1","cwd":"/p","tool_name":"Bash","tool_input":{"command":"ls"},"model":"claude-opus-5-5"}"#;
        let e = parse("/event?e=running&ppid=4242", body);
        assert_eq!((e.e.as_str(), e.ppid, e.session_id.as_str(), e.tool_cmd.as_str()), ("running", 4242, "s1", "ls"));
        let e = parse("/event?e=done", "not json");
        assert_eq!((e.session_id.as_str(), e.ppid), ("unknown", 0));
    }

    #[test]
    fn serves_the_frozen_route_on_loopback() {
        let port = std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
        let (tx, rx) = std::sync::mpsc::channel();
        let tx = std::sync::Mutex::new(tx);
        start(port, Arc::new(move |e| tx.lock().unwrap().send(e).unwrap()));
        std::thread::sleep(std::time::Duration::from_millis(200));
        let r = ureq::post(&format!("http://127.0.0.1:{port}/event?e=attention&ppid=7")).send_string(r#"{"session_id":"x","message":"Allow?"}"#).unwrap();
        assert_eq!(r.into_string().unwrap(), "ok");
        let e = rx.recv_timeout(std::time::Duration::from_secs(2)).unwrap();
        assert_eq!((e.e.as_str(), e.message.as_str(), e.ppid), ("attention", "Allow?", 7));
    }
}
