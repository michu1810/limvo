//! OpenAI Codex limits, read with the sign-in the Codex CLI and app keep.
//!
//! The ChatGPT backend reports the windows of the account's plan: a weekly
//! allowance and, on some plans, a five-hour one. Like the Claude provider,
//! everything is read-only: the token in `auth.json` is never refreshed or
//! written, so Limvo can never sign the user out of Codex.

use std::path::PathBuf;
use std::time::Duration;

use reqwest::StatusCode;
use reqwest::header::{AUTHORIZATION, RETRY_AFTER, USER_AGENT};
use serde::Deserialize;

use super::{FetchFuture, ProviderError, UsageProvider};
use crate::snapshot::{PlanBadge, Severity, UsageSnapshot, UsageWindow, WindowKind};

pub const PROVIDER_ID: &str = "codex";
const USAGE_URL: &str = "https://chatgpt.com/backend-api/wham/usage";
const USAGE_PAGE_URL: &str = "https://chatgpt.com/codex/settings/usage";
/// A window of a day or more is a weekly allowance; Codex sends 604800 and 18000 today.
const WEEKLY_THRESHOLD_SECONDS: u64 = 86_400;

#[derive(Debug, Default)]
pub struct CodexProvider;

impl CodexProvider {
    pub fn new() -> Self {
        Self
    }
}

impl UsageProvider for CodexProvider {
    fn id(&self) -> &'static str {
        PROVIDER_ID
    }

    fn name(&self) -> &'static str {
        "Codex"
    }

    fn fetch<'a>(&'a self, http: &'a reqwest::Client) -> FetchFuture<'a> {
        Box::pin(fetch(http))
    }

    /// The CLI and the app write a session log for every conversation.
    fn activity_paths(&self) -> Vec<PathBuf> {
        codex_home()
            .map(|home| home.join("sessions"))
            .into_iter()
            .collect()
    }
}

/// `CODEX_HOME` when set, otherwise `~/.codex`, as the Codex CLI resolves it.
fn codex_home() -> Option<PathBuf> {
    std::env::var_os("CODEX_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::home_dir().map(|home| home.join(".codex")))
}

#[derive(Deserialize)]
struct AuthFile {
    tokens: Option<Tokens>,
}

#[derive(Deserialize)]
struct Tokens {
    access_token: Option<String>,
    account_id: Option<String>,
}

struct SignIn {
    token: String,
    account_id: Option<String>,
}

impl std::fmt::Debug for SignIn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SignIn")
            .field("token", &"<redacted>")
            .finish_non_exhaustive()
    }
}

fn read_sign_in() -> Option<SignIn> {
    let text = std::fs::read_to_string(codex_home()?.join("auth.json")).ok()?;
    let tokens = serde_json::from_str::<AuthFile>(&text).ok()?.tokens?;
    let token = tokens.access_token.filter(|token| !token.is_empty())?;
    Some(SignIn {
        token,
        account_id: tokens.account_id.filter(|id| !id.is_empty()),
    })
}

async fn fetch(http: &reqwest::Client) -> Result<UsageSnapshot, ProviderError> {
    let sign_in = read_sign_in().ok_or(ProviderError::NoCredentials)?;
    let mut request = http
        .get(USAGE_URL)
        .header(AUTHORIZATION, format!("Bearer {}", sign_in.token))
        .header(USER_AGENT, "codex-cli");
    if let Some(account_id) = &sign_in.account_id {
        request = request.header("ChatGPT-Account-Id", account_id);
    }
    let response = request
        .send()
        .await
        .map_err(|error| ProviderError::Unreachable(error.without_url().to_string()))?;

    match response.status() {
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => Err(ProviderError::AuthRequired),
        StatusCode::TOO_MANY_REQUESTS => Err(ProviderError::RateLimited {
            retry_after: response
                .headers()
                .get(RETRY_AFTER)
                .and_then(|value| value.to_str().ok()?.trim().parse().ok())
                .map(Duration::from_secs),
        }),
        status if status.is_server_error() => {
            Err(ProviderError::Unreachable(format!("HTTP {status}")))
        }
        status if !status.is_success() => Err(ProviderError::Unexpected(format!("HTTP {status}"))),
        _ => {
            let body: UsageResponse = response
                .json()
                .await
                .map_err(|error| ProviderError::Unexpected(error.without_url().to_string()))?;
            Ok(to_snapshot(body, jiff::Timestamp::now().to_string()))
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct UsageResponse {
    plan_type: Option<String>,
    rate_limit: Option<RateLimit>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct RateLimit {
    limit_reached: bool,
    primary_window: Option<Window>,
    secondary_window: Option<Window>,
}

#[derive(Debug, Deserialize)]
struct Window {
    used_percent: f64,
    reset_at: Option<i64>,
    limit_window_seconds: Option<u64>,
}

fn to_snapshot(response: UsageResponse, fetched_at: String) -> UsageSnapshot {
    let rate_limit = response.rate_limit.unwrap_or_default();
    let limit_reached = rate_limit.limit_reached;
    let mut weekly = None;
    let mut session = None;
    // Codex does not promise which slot holds which window, so sort them by length.
    for window in [rate_limit.primary_window, rate_limit.secondary_window]
        .into_iter()
        .flatten()
    {
        let is_weekly = window
            .limit_window_seconds
            .is_none_or(|seconds| seconds >= WEEKLY_THRESHOLD_SECONDS);
        let slot = if is_weekly { &mut weekly } else { &mut session };
        if slot.is_none() {
            *slot = Some(usage_window(&window, is_weekly, limit_reached));
        }
    }

    let (headline, windows) = match weekly {
        Some(weekly) => (Some(weekly), session.into_iter().collect()),
        None => (session, Vec::new()),
    };

    UsageSnapshot {
        provider_id: PROVIDER_ID.to_owned(),
        provider_name: "Codex".to_owned(),
        plan: response.plan_type.as_deref().and_then(plan_badge),
        fetched_at,
        headline,
        windows,
        breakdown: Vec::new(),
        extra_usage: None,
        usage_page_url: Some(USAGE_PAGE_URL.to_owned()),
    }
}

fn usage_window(window: &Window, is_weekly: bool, limit_reached: bool) -> UsageWindow {
    let percent = if window.used_percent.is_finite() {
        window.used_percent.clamp(0.0, 100.0)
    } else {
        0.0
    };
    let severity = if percent >= 100.0 || (limit_reached && percent >= 99.0) {
        Severity::Critical
    } else if percent >= 80.0 {
        Severity::Warning
    } else {
        Severity::Normal
    };
    UsageWindow {
        id: if is_weekly { "weekly_all" } else { "session" }.to_owned(),
        kind: if is_weekly {
            WindowKind::Weekly
        } else {
            WindowKind::Session
        },
        label: None,
        percent,
        resets_at: window
            .reset_at
            .and_then(|seconds| jiff::Timestamp::from_second(seconds).ok())
            .map(|instant| instant.to_string()),
        period_seconds: window.limit_window_seconds,
        severity,
    }
}

/// ChatGPT plan names, prefixed so their badges never collide with Claude plan colours.
fn plan_badge(plan: &str) -> Option<PlanBadge> {
    let label = match plan.trim().to_ascii_lowercase().as_str() {
        "" => return None,
        "free" => "Free",
        "go" => "Go",
        "plus" => "Plus",
        "pro" => "Pro",
        "prolite" | "pro_lite" => "Pro Lite",
        "team" => "Team",
        "business" => "Business",
        "enterprise" => "Enterprise",
        "edu" => "Edu",
        other => {
            return Some(PlanBadge {
                id: format!("chatgpt_{other}"),
                label: other.to_owned(),
            });
        }
    };
    Some(PlanBadge {
        id: format!("chatgpt_{}", plan.trim().to_ascii_lowercase()),
        label: label.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> UsageSnapshot {
        let response: UsageResponse =
            serde_json::from_str(include_str!("../../../fixtures/codex-usage.sample.json"))
                .unwrap();
        to_snapshot(response, "2026-11-03T23:00:00Z".to_owned())
    }

    #[test]
    fn the_weekly_window_is_the_headline_and_the_session_is_another_limit() {
        let snapshot = sample();
        let headline = snapshot.headline.expect("headline");
        assert_eq!(headline.kind, WindowKind::Weekly);
        assert!((headline.percent - 23.0).abs() < f64::EPSILON);
        assert_eq!(headline.period_seconds, Some(604_800));
        assert_eq!(snapshot.windows.len(), 1);
        assert_eq!(snapshot.windows[0].kind, WindowKind::Session);
        assert_eq!(snapshot.windows[0].severity, Severity::Normal);
    }

    #[test]
    fn reset_times_become_rfc_3339() {
        let headline = sample().headline.unwrap();
        assert_eq!(headline.resets_at.as_deref(), Some("2026-11-10T21:14:37Z"));
    }

    #[test]
    fn names_the_plan() {
        let plan = sample().plan.unwrap();
        assert_eq!(
            (plan.id.as_str(), plan.label.as_str()),
            ("chatgpt_prolite", "Pro Lite")
        );
    }

    #[test]
    fn a_plan_without_windows_has_no_headline() {
        let snapshot = to_snapshot(
            serde_json::from_str(r#"{"plan_type": "free"}"#).unwrap(),
            String::new(),
        );
        assert!(snapshot.headline.is_none());
        assert_eq!(snapshot.windows, []);
    }
}
