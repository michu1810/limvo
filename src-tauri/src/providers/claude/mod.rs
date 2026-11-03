//! Claude plan limits, read from the same endpoint the Claude apps use.

mod api;
mod credentials;
mod desktop;
mod plan;
mod spend;

use std::path::PathBuf;
use std::time::Duration;

use reqwest::StatusCode;
use reqwest::header::{AUTHORIZATION, RETRY_AFTER};

use super::{FetchFuture, ProviderError, UsageProvider};
use crate::snapshot::UsageSnapshot;

pub const PROVIDER_ID: &str = "claude";

/// Sign-ins tried per refresh. More would only repeat the same rejection.
const MAX_ATTEMPTS: usize = 3;

#[derive(Debug, Default)]
pub struct ClaudeProvider;

impl ClaudeProvider {
    pub fn new() -> Self {
        Self
    }
}

impl UsageProvider for ClaudeProvider {
    fn fetch<'a>(&'a self, http: &'a reqwest::Client) -> FetchFuture<'a> {
        Box::pin(fetch(http))
    }

    /// Claude Code appends to a transcript under `projects/` on every turn.
    fn activity_paths(&self) -> Vec<PathBuf> {
        credentials::claude_config_dir()
            .map(|dir| dir.join("projects"))
            .into_iter()
            .collect()
    }
}

async fn fetch(http: &reqwest::Client) -> Result<UsageSnapshot, ProviderError> {
    let now = jiff::Timestamp::now();
    let candidates = credentials::discover(now.as_millisecond());
    if candidates.usable.is_empty() {
        return Err(if candidates.found_any {
            ProviderError::AuthRequired
        } else {
            ProviderError::NoCredentials
        });
    }

    for token in candidates.usable.iter().take(MAX_ATTEMPTS) {
        match request(http, token.secret()).await {
            Ok(response) => {
                log::debug!("usage read with a token from the {}", token.source);
                let mut snapshot = api::to_snapshot(response, jiff::Timestamp::now().to_string());
                snapshot.plan.clone_from(&token.plan);
                return Ok(snapshot);
            }
            // A token without the profile scope or a revoked one: try the next sign-in.
            Err(Attempt::Rejected(status)) => {
                log::info!("the {} token was rejected ({status})", token.source);
            }
            Err(Attempt::Failed(error)) => return Err(error),
        }
    }
    Err(ProviderError::AuthRequired)
}

enum Attempt {
    Rejected(StatusCode),
    Failed(ProviderError),
}

async fn request(http: &reqwest::Client, token: &str) -> Result<api::UsageResponse, Attempt> {
    let response = http
        .get(api::USAGE_URL)
        .header(AUTHORIZATION, format!("Bearer {token}"))
        .header("anthropic-beta", api::OAUTH_BETA)
        .send()
        .await
        .map_err(|error| {
            Attempt::Failed(ProviderError::Unreachable(error.without_url().to_string()))
        })?;

    let status = response.status();
    match status {
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => Err(Attempt::Rejected(status)),
        StatusCode::TOO_MANY_REQUESTS => Err(Attempt::Failed(ProviderError::RateLimited {
            retry_after: retry_after(&response),
        })),
        status if status.is_server_error() => Err(Attempt::Failed(ProviderError::Unreachable(
            format!("HTTP {status}"),
        ))),
        status if !status.is_success() => Err(Attempt::Failed(ProviderError::Unexpected(format!(
            "HTTP {status}"
        )))),
        _ => response
            .json::<api::UsageResponse>()
            .await
            .map_err(|error| {
                Attempt::Failed(ProviderError::Unexpected(error.without_url().to_string()))
            }),
    }
}

fn retry_after(response: &reqwest::Response) -> Option<Duration> {
    let seconds = response
        .headers()
        .get(RETRY_AFTER)?
        .to_str()
        .ok()?
        .trim()
        .parse::<u64>()
        .ok()?;
    Some(Duration::from_secs(seconds))
}
