//! Usage providers.
//!
//! A provider turns one service's API into a [`UsageSnapshot`]. Adding a
//! service means adding a module here and registering it in
//! [`default_provider`]; nothing in the UI or the poller changes. See
//! `CONTRIBUTING.md` for a walkthrough.

pub mod claude;

use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use crate::snapshot::UsageSnapshot;

pub type FetchFuture<'a> =
    Pin<Box<dyn Future<Output = Result<UsageSnapshot, ProviderError>> + Send + 'a>>;

/// A source of usage readings. Implementations must never log or return credentials.
pub trait UsageProvider: Send + Sync {
    /// Reads the current usage. Called by the poller, never concurrently.
    fn fetch<'a>(&'a self, http: &'a reqwest::Client) -> FetchFuture<'a>;
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

pub fn default_provider() -> Box<dyn UsageProvider> {
    Box::new(claude::ClaudeProvider::new())
}
