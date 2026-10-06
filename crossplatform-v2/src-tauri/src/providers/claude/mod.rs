//! Claude, official: GET https://api.anthropic.com/api/oauth/usage with Claude Code's own OAuth
//! token. The app runs no OAuth flow of its own; it borrows the CLI's credential, read only.

mod api;
mod credentials;
mod parse;

use super::{FetchError, ProviderId, Reading, UsageProvider};

pub struct Claude;

impl UsageProvider for Claude {
    fn id(&self) -> ProviderId {
        ProviderId::Claude
    }

    fn fetch(&self) -> Result<Reading, FetchError> {
        let Some(cred) = credentials::read() else {
            return Err(FetchError::NeedsAuth("No Claude Code credential found".into()));
        };
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
            Err(api::Error::Rejected) => Err(FetchError::NeedsAuth(
                if cred.expired {
                    "Credential expired — run any claude command (or chat with Claude) to refresh it"
                } else {
                    "Credential rejected (switched accounts?)"
                }
                .into(),
            )),
            Err(api::Error::RateLimited(ra)) => Err(FetchError::RateLimited { retry_after_secs: ra }),
            Err(api::Error::Other(msg)) => Err(FetchError::Other(msg)),
        }
    }

    fn probe(&self) -> String {
        credentials::probe()
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
