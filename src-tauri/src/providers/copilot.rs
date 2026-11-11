//! GitHub Copilot limits, read with the sign-in of the GitHub CLI.
//!
//! Copilot reports its monthly quotas (premium requests on paid plans, chat
//! and code completions on Copilot Free) on the same endpoint its editors
//! use. Limvo borrows the token `gh auth login` keeps, from the Windows
//! Credential Manager or, when it was stored in plain text, from `hosts.yml`.

use std::path::PathBuf;

use reqwest::header::{ACCEPT, AUTHORIZATION, USER_AGENT};
use serde::Deserialize;

use super::{
    FetchFuture, ProviderError, UsageProvider, clamp_percent, credential, severity_of, status_error,
};
use crate::snapshot::{PlanBadge, UsageSnapshot, UsageWindow, WindowKind};

pub const PROVIDER_ID: &str = "copilot";
const API: &str = "https://api.github.com/copilot_internal/user";
const HOST: &str = "github.com";
const USAGE_PAGE: &str = "https://github.com/settings/copilot/features";

/// The quotas worth a row, in the order they matter: the first one that is
/// limited leads. Unlimited ones (chat and completions on paid plans) are left out.
const QUOTAS: [(&str, &str, &str); 3] = [
    (
        "premium_interactions",
        "monthly_premium",
        "Premium requests",
    ),
    ("chat", "monthly_chat", "Chat"),
    ("completions", "monthly_completions", "Completions"),
];

#[derive(Debug, Default)]
pub struct CopilotProvider;

impl CopilotProvider {
    pub fn new() -> Self {
        Self
    }
}

impl UsageProvider for CopilotProvider {
    fn service(&self) -> &'static str {
        PROVIDER_ID
    }

    fn name(&self) -> &'static str {
        "Copilot"
    }

    fn fetch<'a>(&'a self, http: &'a reqwest::Client) -> FetchFuture<'a> {
        Box::pin(fetch(http))
    }
}

struct SignIn {
    token: String,
}

impl std::fmt::Debug for SignIn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SignIn")
            .field("token", &"<redacted>")
            .finish()
    }
}

/// The folder the GitHub CLI keeps its configuration in.
fn config_dir() -> Option<PathBuf> {
    std::env::var_os("GH_CONFIG_DIR")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("APPDATA").map(|dir| PathBuf::from(dir).join("GitHub CLI")))
}

/// The active account's token: the Credential Manager entry the CLI keeps for
/// it, then the one named after its user, then a token stored in plain text.
fn read_sign_in() -> Option<SignIn> {
    let hosts = config_dir()
        .and_then(|dir| std::fs::read_to_string(dir.join("hosts.yml")).ok())
        .map(|text| host_entries(&text, HOST))
        .unwrap_or_default();
    let stored =
        |target: String| credential::read(&target).and_then(|blob| String::from_utf8(blob).ok());
    let token = stored(format!("gh:{HOST}:"))
        .or_else(|| {
            hosts
                .user
                .as_ref()
                .and_then(|user| stored(format!("gh:{HOST}:{user}")))
        })
        .or(hosts.oauth_token)?;
    let token = token.trim().to_owned();
    (!token.is_empty()).then_some(SignIn { token })
}

/// What `hosts.yml` says about one host.
#[derive(Debug, Default, PartialEq, Eq)]
struct HostEntries {
    user: Option<String>,
    oauth_token: Option<String>,
}

/// Reads the two keys this provider needs from the host's block of the
/// CLI's `hosts.yml`, without a YAML parser: the file is flat key-value
/// pairs under each host, and only the host's own keys (its first level of
/// indentation) count, not those of each user under `users:`.
fn host_entries(text: &str, host: &str) -> HostEntries {
    let mut entries = HostEntries::default();
    let mut inside = false;
    let mut indent: Option<usize> = None;
    for line in text.lines() {
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            continue;
        }
        let depth = line.len() - line.trim_start().len();
        if depth == 0 {
            inside = line.trim_end().trim_end_matches(':').trim_matches('"') == host;
            indent = None;
            continue;
        }
        if !inside {
            continue;
        }
        let level = *indent.get_or_insert(depth);
        if depth != level {
            continue;
        }
        let Some((key, value)) = line.trim().split_once(':') else {
            continue;
        };
        let value = value.trim().trim_matches('"').to_owned();
        if value.is_empty() {
            continue;
        }
        match key.trim() {
            "user" => entries.user = Some(value),
            "oauth_token" => entries.oauth_token = Some(value),
            _ => {}
        }
    }
    entries
}

async fn fetch(http: &reqwest::Client) -> Result<UsageSnapshot, ProviderError> {
    let sign_in = read_sign_in().ok_or(ProviderError::NoCredentials)?;
    let response = http
        .get(API)
        .header(AUTHORIZATION, format!("Bearer {}", sign_in.token))
        .header(ACCEPT, "application/json")
        .header(USER_AGENT, "Limvo")
        .send()
        .await
        .map_err(|error| ProviderError::Unreachable(error.without_url().to_string()))?;
    // A GitHub account without Copilot answers 404: there is nothing to watch.
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(ProviderError::NoCredentials);
    }
    if let Some(error) = status_error(&response) {
        return Err(error);
    }
    let user: UserResponse = response
        .json()
        .await
        .map_err(|error| ProviderError::Unexpected(error.without_url().to_string()))?;
    Ok(to_snapshot(&user, jiff::Timestamp::now().to_string()))
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct UserResponse {
    access_type_sku: Option<String>,
    copilot_plan: Option<String>,
    quota_snapshots: serde_json::Map<String, serde_json::Value>,
    quota_reset_date_utc: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct Quota {
    entitlement: Option<f64>,
    remaining: Option<f64>,
    percent_remaining: Option<f64>,
    unlimited: bool,
}

impl Quota {
    /// Share used, or `None` for a quota that is unlimited or not part of the plan.
    fn used(&self) -> Option<f64> {
        let entitlement = self.entitlement.filter(|entitlement| *entitlement > 0.0)?;
        if self.unlimited {
            return None;
        }
        let remaining = self.percent_remaining.or_else(|| {
            self.remaining
                .map(|remaining| remaining / entitlement * 100.0)
        })?;
        Some(clamp_percent(100.0 - remaining))
    }
}

fn to_snapshot(user: &UserResponse, fetched_at: String) -> UsageSnapshot {
    let resets_at = user
        .quota_reset_date_utc
        .as_deref()
        .and_then(|date| date.parse::<jiff::Timestamp>().ok());
    let period_seconds = resets_at.and_then(month_before);
    let resets_at = resets_at.map(|instant| instant.to_string());

    let mut windows = QUOTAS.iter().filter_map(|(key, id, label)| {
        let quota: Quota = serde_json::from_value(user.quota_snapshots.get(*key)?.clone()).ok()?;
        let percent = quota.used()?;
        Some(UsageWindow {
            id: (*id).to_owned(),
            kind: WindowKind::Monthly,
            label: Some((*label).to_owned()),
            percent,
            resets_at: resets_at.clone(),
            period_seconds,
            severity: severity_of(percent),
        })
    });
    let headline = windows.next();
    let windows = windows.collect();

    UsageSnapshot {
        provider_id: PROVIDER_ID.to_owned(),
        provider_name: "Copilot".to_owned(),
        plan: plan_badge(user),
        fetched_at,
        headline,
        windows,
        breakdown: Vec::new(),
        extra_usage: None,
        usage_page_url: Some(USAGE_PAGE.to_owned()),
    }
}

/// Length of the month that ends at `reset`: quotas reset monthly, on the
/// day the plan started, so the period runs from the same day a month back.
fn month_before(reset: jiff::Timestamp) -> Option<u64> {
    let end = reset.to_zoned(jiff::tz::TimeZone::UTC);
    let start = end.checked_sub(jiff::Span::new().months(1)).ok()?;
    u64::try_from(end.timestamp().as_second() - start.timestamp().as_second()).ok()
}

/// The plan by its billing SKU first, which tells Free from Pro on an
/// individual account, then by the kind of account.
fn plan_badge(user: &UserResponse) -> Option<PlanBadge> {
    let sku = user.access_type_sku.as_deref().unwrap_or_default();
    let (id, label) = if sku.contains("free") {
        ("copilot_free", "Free")
    } else {
        match user.copilot_plan.as_deref()? {
            "business" => ("copilot_business", "Business"),
            "enterprise" => ("copilot_enterprise", "Enterprise"),
            _ if sku.contains("plus") => ("copilot_pro_plus", "Pro+"),
            _ => ("copilot_pro", "Pro"),
        }
    };
    Some(PlanBadge {
        id: id.to_owned(),
        label: label.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample() -> UserResponse {
        serde_json::from_str(include_str!("../../../fixtures/copilot-user.sample.json")).unwrap()
    }

    #[test]
    fn copilot_free_leads_with_chat_and_lists_completions() {
        let snapshot = to_snapshot(&sample(), "2026-11-11T20:00:00Z".to_owned());
        let headline = snapshot.headline.expect("headline");
        assert_eq!(headline.id, "monthly_chat");
        assert_eq!(headline.label.as_deref(), Some("Chat"));
        assert!((headline.percent - 27.5).abs() < 1e-9);
        assert_eq!(headline.kind, WindowKind::Monthly);
        assert_eq!(headline.resets_at.as_deref(), Some("2026-12-01T00:00:00Z"));
        // November has 30 days.
        assert_eq!(headline.period_seconds, Some(30 * 86_400));
        let windows: Vec<_> = snapshot.windows.iter().map(|w| w.id.as_str()).collect();
        assert_eq!(windows, ["monthly_completions"]);
        let plan = snapshot.plan.unwrap();
        assert_eq!(
            (plan.id.as_str(), plan.label.as_str()),
            ("copilot_free", "Free")
        );
        assert_eq!(snapshot.usage_page_url.as_deref(), Some(USAGE_PAGE));
    }

    #[test]
    fn a_paid_plan_leads_with_premium_requests() {
        let user: UserResponse = serde_json::from_value(json!({
            "access_type_sku": "plus_monthly_subscriber_quota",
            "copilot_plan": "individual",
            "quota_reset_date_utc": "2026-12-01T00:00:00.000Z",
            "quota_snapshots": {
                "premium_interactions": { "entitlement": 1500, "remaining": 1200, "unlimited": false },
                "chat": { "entitlement": 0, "unlimited": true, "percent_remaining": 100 },
                "completions": { "entitlement": 0, "unlimited": true, "percent_remaining": 100 }
            }
        }))
        .unwrap();
        let snapshot = to_snapshot(&user, String::new());
        let headline = snapshot.headline.unwrap();
        assert_eq!(headline.id, "monthly_premium");
        assert!((headline.percent - 20.0).abs() < 1e-9);
        assert_eq!(snapshot.windows.len(), 0);
        assert_eq!(snapshot.plan.unwrap().label, "Pro+");
    }

    #[test]
    fn names_business_and_enterprise_plans() {
        let plan = |copilot_plan: &str| {
            plan_badge(&UserResponse {
                access_type_sku: Some("copilot_for_business_seat".to_owned()),
                copilot_plan: Some(copilot_plan.to_owned()),
                ..UserResponse::default()
            })
            .unwrap()
            .id
        };
        assert_eq!(plan("business"), "copilot_business");
        assert_eq!(plan("enterprise"), "copilot_enterprise");
    }

    #[test]
    fn reads_the_hosts_own_keys_only() {
        let hosts = "github.com:\n    users:\n        octo:\n            oauth_token: user-level\n    git_protocol: https\n    user: octo\n    oauth_token: host-level\nghe.example.com:\n    user: other\n";
        assert_eq!(
            host_entries(hosts, "github.com"),
            HostEntries {
                user: Some("octo".to_owned()),
                oauth_token: Some("host-level".to_owned()),
            }
        );
        assert_eq!(
            host_entries("ghe.example.com:\n    user: other\n", "github.com"),
            HostEntries::default()
        );
    }

    #[test]
    fn a_sign_in_never_shows_its_token() {
        let sign_in = SignIn {
            token: "secret".to_owned(),
        };
        assert!(!format!("{sign_in:?}").contains("secret"));
    }
}
