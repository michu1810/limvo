//! Usage providers.
//!
//! A provider turns one service's API into a [`UsageSnapshot`]. Adding a
//! service means adding a module here and registering it in [`all`];
//! nothing in the UI or the poller changes. See
//! `CONTRIBUTING.md` for a walkthrough.

pub mod antigravity;
pub mod claude;
pub mod codex;
pub mod copilot;
mod credential;
pub mod cursor;

use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::time::Duration;

use reqwest::StatusCode;
use reqwest::header::RETRY_AFTER;

use crate::accounts::{Account, Service};
use crate::snapshot::{Severity, UsageSnapshot};

pub type FetchFuture<'a> =
    Pin<Box<dyn Future<Output = Result<UsageSnapshot, ProviderError>> + Send + 'a>>;

/// A source of usage readings. Implementations must never log or return credentials.
pub trait UsageProvider: Send + Sync {
    /// The service: names the optional brand mark file and the plan colours.
    fn service(&self) -> &'static str;

    /// Stable identifier of this account's reports, pins and history: the
    /// service id for the default account, `<service>~<n>` for an extra one.
    fn id(&self) -> &str {
        self.service()
    }

    /// Name of the account as the interface shows it.
    fn name(&self) -> &str;

    /// Reads the current usage. Called by the poller, never concurrently.
    fn fetch<'a>(&'a self, http: &'a reqwest::Client) -> FetchFuture<'a>;

    /// Folders the service's own tools write to while the user works with
    /// them. Changes there make the adaptive schedule read sooner. Files are
    /// watched, never opened.
    fn activity_paths(&self) -> Vec<PathBuf> {
        Vec::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    /// Nothing on this machine can sign in to the service.
    #[error("no credentials found")]
    NoCredentials,
    /// Credentials exist but the service rejected all of them.
    #[error("credentials were rejected")]
    AuthRequired,
    /// The service could not be reached, or answered with a server error.
    #[error("service unreachable: {0}")]
    Unreachable(String),
    /// The service asked us to slow down.
    #[error("rate limited")]
    RateLimited { retry_after: Option<Duration> },
    /// The service answered with something we do not understand.
    #[error("unexpected response: {0}")]
    Unexpected(String),
}

/// Every provider, in the order the interface lists them: each service's
/// default account, then its extra accounts in the order they were added.
pub fn all(accounts: &[Account]) -> Vec<Box<dyn UsageProvider>> {
    let extra = |service: Service| accounts.iter().filter(move |a| a.service == service);
    let mut providers: Vec<Box<dyn UsageProvider>> = vec![Box::new(claude::ClaudeProvider::new())];
    providers.extend(
        extra(Service::Claude)
            .map(|a| Box::new(claude::ClaudeProvider::for_account(a)) as Box<dyn UsageProvider>),
    );
    providers.push(Box::new(codex::CodexProvider::new()));
    providers.extend(
        extra(Service::Codex)
            .map(|a| Box::new(codex::CodexProvider::for_account(a)) as Box<dyn UsageProvider>),
    );
    providers.push(Box::new(cursor::CursorProvider::new()));
    providers.push(Box::new(antigravity::AntigravityProvider::new()));
    providers.push(Box::new(copilot::CopilotProvider::new()));
    providers
}

/// The identity of an extra account inside its provider.
#[derive(Debug, Clone)]
pub(crate) struct AccountSource {
    pub id: String,
    pub name: String,
    pub folder: PathBuf,
}

impl From<&Account> for AccountSource {
    fn from(account: &Account) -> Self {
        Self {
            id: account.id.clone(),
            name: account.display_name(),
            folder: account.folder.clone(),
        }
    }
}

/// The error an unsuccessful answer stands for, or `None` for a success.
pub(crate) fn status_error(response: &reqwest::Response) -> Option<ProviderError> {
    match response.status() {
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => Some(ProviderError::AuthRequired),
        StatusCode::TOO_MANY_REQUESTS => Some(ProviderError::RateLimited {
            retry_after: response
                .headers()
                .get(RETRY_AFTER)
                .and_then(|value| value.to_str().ok()?.trim().parse().ok())
                .map(Duration::from_secs),
        }),
        status if status.is_server_error() => {
            Some(ProviderError::Unreachable(format!("HTTP {status}")))
        }
        status if !status.is_success() => Some(ProviderError::Unexpected(format!("HTTP {status}"))),
        _ => None,
    }
}

/// A reported share as a percentage the interface can draw: finite and within 0 to 100.
pub(crate) fn clamp_percent(value: f64) -> f64 {
    if value.is_finite() {
        value.clamp(0.0, 100.0)
    } else {
        0.0
    }
}

/// The colour a reading earns when the service does not grade it itself.
pub(crate) fn severity_of(percent: f64) -> Severity {
    if percent >= 100.0 {
        Severity::Critical
    } else if percent >= 80.0 {
        Severity::Warning
    } else {
        Severity::Normal
    }
}
