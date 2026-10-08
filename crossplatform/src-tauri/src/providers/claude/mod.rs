//! Claude, official: GET https://api.anthropic.com/api/oauth/usage with Claude Code's own OAuth
//! token. The app runs no OAuth flow of its own; it borrows the CLI's credential, read only.

mod activity;
mod api;
mod credentials;
mod parse;
mod renew;

use super::{Activity, FetchError, ProviderId, Reading, UsageProvider};

const EXPIRED_NOTE: &str = "Credential expired — run claude once in a terminal to renew it";

#[derive(Default)]
pub struct Claude {
    net: std::sync::Mutex<activity::NetProbe>,
    renewer: std::sync::Mutex<renew::Renewer>,
}

impl UsageProvider for Claude {
    fn id(&self) -> ProviderId {
        ProviderId::Claude
    }

    /// Only cloud sessions of the desktop app; local sessions come from hooks and transcripts
    fn activity(&self) -> Vec<Activity> {
        self.net.lock().unwrap().read(crate::support::time::now_ms())
    }

    fn fetch(&self) -> Result<Reading, FetchError> {
        let Some(mut cred) = credentials::read() else {
            return Err(FetchError::NeedsAuth("No Claude Code credential found".into()));
        };
        if self.renewer.lock().unwrap().maybe_renew(&cred, crate::support::time::now_ms()) == Some(true) {
            cred = credentials::read().unwrap_or(cred);
        }
        // The endpoint answers an expired token with an hour-long 429, not 401: never send one
        if cred.expired {
            return Err(FetchError::NeedsAuth(EXPIRED_NOTE.into()));
        }
        // Claude Code may have refreshed the token since we read it: re-read once and retry
        let result = match api::fetch(&cred.token) {
            Err(api::Error::Rejected) => match credentials::read() {
                Some(again) if again.token != cred.token => api::fetch(&again.token),
                _ => Err(api::Error::Rejected),
            },
            other => other,
        };
        match result {
            Ok(body) => Ok(Reading::live(parse::windows(&body), "")),
            Err(api::Error::Rejected) => Err(FetchError::NeedsAuth("Credential rejected (switched accounts?)".into())),
            Err(api::Error::RateLimited(ra)) => Err(FetchError::RateLimited { retry_after_secs: ra }),
            Err(api::Error::Other(msg)) => Err(FetchError::Other(msg)),
        }
    }

    fn probe(&self) -> String {
        format!("{}; {}", credentials::probe(), renew::probe())
    }

    /// 60 s while a Claude session runs, 5 min otherwise
    fn poll_secs(&self, session_active: bool) -> u64 {
        if session_active {
            60
        } else {
            300
        }
    }
}
