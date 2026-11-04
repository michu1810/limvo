//! Claude plan limits, read from the same endpoint the Claude apps use.

mod api;
mod credentials;
mod desktop;
mod plan;
mod spend;

use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use reqwest::StatusCode;
use reqwest::header::{AUTHORIZATION, RETRY_AFTER};

use super::{FetchFuture, ProviderError, UsageProvider};
use crate::snapshot::{PlanBadge, UsageSnapshot};

pub const PROVIDER_ID: &str = "claude";

/// Sign-ins tried per refresh. More would only repeat the same rejection.
const MAX_ATTEMPTS: usize = 3;
/// Plans change rarely; the profile is read again after this long.
const PROFILE_TTL: Duration = Duration::from_hours(6);

#[derive(Debug, Default)]
pub struct ClaudeProvider {
    /// The plan from the account profile, and when it was read.
    profile_plan: Mutex<Option<(Instant, PlanBadge)>>,
}

impl ClaudeProvider {
    pub fn new() -> Self {
        Self::default()
    }

    /// The current plan from the profile, read at most every few hours.
    /// `None` when the profile cannot be read; the token's plan stands in then.
    async fn current_plan(&self, http: &reqwest::Client, token: &str) -> Option<PlanBadge> {
        let cached = self
            .profile_plan
            .lock()
            .ok()
            .and_then(|cache| cache.clone())
            .filter(|(read_at, _)| read_at.elapsed() < PROFILE_TTL);
        if let Some((_, plan)) = cached {
            return Some(plan);
        }
        let plan = read_profile(http, token)
            .await
            .and_then(|profile| plan::from_profile(&profile))?;
        if let Ok(mut cache) = self.profile_plan.lock() {
            *cache = Some((Instant::now(), plan.clone()));
        }
        Some(plan)
    }
}

async fn read_profile(http: &reqwest::Client, token: &str) -> Option<plan::Profile> {
    let response = http
        .get(plan::PROFILE_URL)
        .header(AUTHORIZATION, format!("Bearer {token}"))
        .header("anthropic-beta", api::OAUTH_BETA)
        .send()
        .await
        .ok()?;
    if !response.status().is_success() {
        log::debug!("the profile was not readable ({})", response.status());
        return None;
    }
    response.json().await.ok()
}

impl UsageProvider for ClaudeProvider {
    fn id(&self) -> &'static str {
        PROVIDER_ID
    }

    fn name(&self) -> &'static str {
        "Claude"
    }

    fn fetch<'a>(&'a self, http: &'a reqwest::Client) -> FetchFuture<'a> {
        Box::pin(fetch(self, http))
    }

    /// Claude Code appends to a transcript under `projects/` on every turn.
    fn activity_paths(&self) -> Vec<PathBuf> {
        credentials::claude_config_dir()
            .map(|dir| dir.join("projects"))
            .into_iter()
            .collect()
    }
}

async fn fetch(
    provider: &ClaudeProvider,
    http: &reqwest::Client,
) -> Result<UsageSnapshot, ProviderError> {
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
                snapshot.plan = match provider.current_plan(http, token.secret()).await {
                    Some(plan) => Some(plan),
                    None => token.plan.clone(),
                };
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
