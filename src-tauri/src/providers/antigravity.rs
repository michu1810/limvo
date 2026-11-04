//! Google Antigravity limits, read with the sign-in the Antigravity app keeps.
//!
//! Antigravity stores its Google sign-in in the Windows Credential Manager
//! (`gemini:antigravity`) and reports a weekly quota per model through the
//! Gemini Code Assist backend. The app renews the short-lived token itself;
//! Limvo only borrows it and reports _sign in again_ once it has expired.

use std::collections::HashSet;
use std::path::PathBuf;

use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, USER_AGENT};
use serde::Deserialize;
use serde_json::json;

use super::{FetchFuture, ProviderError, UsageProvider, clamp_percent, severity_of, status_error};
use crate::snapshot::{PlanBadge, UsageSnapshot, UsageWindow, WindowKind};

pub const PROVIDER_ID: &str = "antigravity";
const API: &str = "https://cloudcode-pa.googleapis.com/v1internal";
const CREDENTIAL: &str = "gemini:antigravity";
/// The quota window. An unused model always reports its reset exactly a week ahead.
const WEEK_SECONDS: u64 = 604_800;

#[derive(Debug, Default)]
pub struct AntigravityProvider;

impl AntigravityProvider {
    pub fn new() -> Self {
        Self
    }
}

impl UsageProvider for AntigravityProvider {
    fn service(&self) -> &'static str {
        PROVIDER_ID
    }

    fn name(&self) -> &'static str {
        "Antigravity"
    }

    fn fetch<'a>(&'a self, http: &'a reqwest::Client) -> FetchFuture<'a> {
        Box::pin(fetch(http))
    }

    /// The agent writes every conversation here.
    fn activity_paths(&self) -> Vec<PathBuf> {
        std::env::home_dir()
            .map(|home| {
                home.join(".gemini")
                    .join("antigravity")
                    .join("conversations")
            })
            .into_iter()
            .collect()
    }
}

#[derive(Deserialize)]
struct StoredSignIn {
    token: StoredToken,
}

#[derive(Deserialize)]
struct StoredToken {
    access_token: String,
    expiry: Option<String>,
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

fn read_sign_in() -> Result<SignIn, ProviderError> {
    let blob = credential::read(CREDENTIAL).ok_or(ProviderError::NoCredentials)?;
    let stored: StoredSignIn =
        serde_json::from_slice(&blob).map_err(|_| ProviderError::NoCredentials)?;
    // The app renews the token while it runs; an old one would only be rejected.
    let expired = stored
        .token
        .expiry
        .as_deref()
        .and_then(|expiry| expiry.parse::<jiff::Timestamp>().ok())
        .is_some_and(|expiry| expiry <= jiff::Timestamp::now());
    if expired || stored.token.access_token.is_empty() {
        return Err(ProviderError::AuthRequired);
    }
    Ok(SignIn {
        token: stored.token.access_token,
    })
}

async fn fetch(http: &reqwest::Client) -> Result<UsageSnapshot, ProviderError> {
    let sign_in = read_sign_in()?;
    let assist: AssistResponse = call(
        http,
        &sign_in,
        "loadCodeAssist",
        &json!({ "metadata": { "ideType": "ANTIGRAVITY", "pluginType": "GEMINI" } }),
    )
    .await?;
    let body = match &assist.cloudaicompanion_project {
        Some(project) => json!({ "project": project }),
        None => json!({}),
    };
    let models: ModelsResponse = call(http, &sign_in, "fetchAvailableModels", &body).await?;
    Ok(to_snapshot(
        &assist,
        &models,
        jiff::Timestamp::now().to_string(),
    ))
}

async fn call<T: for<'de> Deserialize<'de>>(
    http: &reqwest::Client,
    sign_in: &SignIn,
    method: &str,
    body: &serde_json::Value,
) -> Result<T, ProviderError> {
    let response = http
        .post(format!("{API}:{method}"))
        .header(AUTHORIZATION, format!("Bearer {}", sign_in.token))
        .header(CONTENT_TYPE, "application/json")
        .header(USER_AGENT, "antigravity")
        .body(body.to_string())
        .send()
        .await
        .map_err(|error| ProviderError::Unreachable(error.without_url().to_string()))?;
    if let Some(error) = status_error(&response) {
        return Err(error);
    }
    response
        .json()
        .await
        .map_err(|error| ProviderError::Unexpected(error.without_url().to_string()))
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct AssistResponse {
    cloudaicompanion_project: Option<String>,
    current_tier: Option<Tier>,
    paid_tier: Option<Tier>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct Tier {
    id: Option<String>,
    name: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct ModelsResponse {
    /// Keyed by model id. The order does not matter: quotas are sorted by use.
    models: serde_json::Map<String, serde_json::Value>,
    default_agent_model_id: Option<String>,
    agent_model_sorts: Vec<ModelSort>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct ModelSort {
    groups: Vec<ModelGroup>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct ModelGroup {
    model_ids: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct Model {
    display_name: Option<String>,
    is_internal: bool,
    quota_info: Option<QuotaInfo>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct QuotaInfo {
    remaining_fraction: Option<f64>,
    reset_time: Option<String>,
}

/// One quota as the interface draws it, plus whether the model picker offers it.
struct Quota {
    window: UsageWindow,
    agent: bool,
    default: bool,
}

fn to_snapshot(
    assist: &AssistResponse,
    models: &ModelsResponse,
    fetched_at: String,
) -> UsageSnapshot {
    let agent: HashSet<&str> = models
        .agent_model_sorts
        .iter()
        .flat_map(|sort| &sort.groups)
        .flat_map(|group| &group.model_ids)
        .map(String::as_str)
        .collect();

    // Several model ids share a display name and a quota; keep one row per name.
    let mut quotas: Vec<Quota> = Vec::new();
    for (id, value) in &models.models {
        let Ok(model) = serde_json::from_value::<Model>(value.clone()) else {
            continue;
        };
        let (Some(name), Some(quota)) = (model.display_name, model.quota_info) else {
            continue;
        };
        // Internal helpers (tab completion and the like) have no reset and no quota to watch.
        if model.is_internal || quota.reset_time.is_none() {
            continue;
        }
        let used = clamp_percent((1.0 - quota.remaining_fraction.unwrap_or(1.0)) * 100.0);
        let is_agent = agent.contains(id.as_str());
        let is_default = models.default_agent_model_id.as_deref() == Some(id.as_str());
        if let Some(existing) = quotas
            .iter_mut()
            .find(|quota| quota.window.label.as_deref() == Some(name.as_str()))
        {
            if used > existing.window.percent {
                existing.window.percent = used;
                existing.window.severity = severity_of(used);
            }
            existing.agent |= is_agent;
            existing.default |= is_default;
            continue;
        }
        quotas.push(Quota {
            window: UsageWindow {
                id: format!("scoped:{}", slug(&name)),
                kind: WindowKind::Scoped,
                label: Some(name),
                percent: used,
                resets_at: quota.reset_time,
                period_seconds: Some(WEEK_SECONDS),
                severity: severity_of(used),
            },
            agent: is_agent,
            default: is_default,
        });
    }

    // The fullest quota leads; with nothing used yet, the model the agent starts with.
    quotas.sort_by(|a, b| {
        b.window
            .percent
            .total_cmp(&a.window.percent)
            .then(b.default.cmp(&a.default))
            .then(b.agent.cmp(&a.agent))
    });
    let mut quotas = quotas.into_iter();
    let headline = quotas.next().map(|quota| quota.window);
    // The agent models, and any other model that has been used, are worth a row.
    let windows = quotas
        .filter(|quota| quota.agent || quota.window.percent > 0.0)
        .map(|quota| quota.window)
        .collect();

    UsageSnapshot {
        provider_id: PROVIDER_ID.to_owned(),
        provider_name: "Antigravity".to_owned(),
        plan: plan_badge(assist),
        fetched_at,
        headline,
        windows,
        breakdown: Vec::new(),
        extra_usage: None,
        usage_page_url: None,
    }
}

/// A paid Google AI plan names the badge; otherwise the tier the account is on.
fn plan_badge(assist: &AssistResponse) -> Option<PlanBadge> {
    let tier = assist
        .paid_tier
        .as_ref()
        .filter(|tier| tier.name.is_some())
        .or(assist.current_tier.as_ref())?;
    let label = tier.name.clone()?;
    let id = tier.id.as_deref().map_or_else(|| slug(&label), slug);
    Some(PlanBadge {
        id: format!("antigravity_{id}"),
        label,
    })
}

fn slug(text: &str) -> String {
    text.to_ascii_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

#[cfg(windows)]
mod credential {
    use windows::Win32::Security::Credentials::{
        CRED_TYPE_GENERIC, CREDENTIALW, CredFree, CredReadW,
    };
    use windows::core::HSTRING;

    /// The secret of a generic Windows credential, read for this user only.
    pub fn read(target: &str) -> Option<Vec<u8>> {
        let mut credential: *mut CREDENTIALW = std::ptr::null_mut();
        // SAFETY: CredReadW fills `credential` with a buffer it owns; it is
        // copied out and released with CredFree before returning.
        unsafe {
            CredReadW(
                &HSTRING::from(target),
                CRED_TYPE_GENERIC,
                None,
                &raw mut credential,
            )
            .ok()?;
            let blob = (*credential).CredentialBlob;
            let size = usize::try_from((*credential).CredentialBlobSize).ok();
            let secret = match size {
                Some(size) if !blob.is_null() && size > 0 => {
                    Some(std::slice::from_raw_parts(blob, size).to_vec())
                }
                _ => None,
            };
            CredFree(credential.cast());
            secret
        }
    }
}

#[cfg(not(windows))]
mod credential {
    pub fn read(_target: &str) -> Option<Vec<u8>> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> UsageSnapshot {
        let assist: AssistResponse = serde_json::from_str(include_str!(
            "../../../fixtures/antigravity-assist.sample.json"
        ))
        .unwrap();
        let models = serde_json::from_str(include_str!(
            "../../../fixtures/antigravity-models.sample.json"
        ))
        .unwrap();
        to_snapshot(&assist, &models, "2026-11-04T00:00:00Z".to_owned())
    }

    #[test]
    fn the_fullest_model_quota_is_the_headline() {
        let headline = sample().headline.expect("headline");
        assert_eq!(headline.label.as_deref(), Some("Gemini 3 Flash"));
        assert!((headline.percent - 64.0).abs() < 1e-9);
        assert_eq!(headline.kind, WindowKind::Scoped);
        assert_eq!(headline.period_seconds, Some(WEEK_SECONDS));
        assert_eq!(headline.resets_at.as_deref(), Some("2026-11-11T00:12:51Z"));
    }

    #[test]
    fn lists_agent_models_and_used_ones_once_each() {
        let labels: Vec<_> = sample()
            .windows
            .into_iter()
            .map(|window| (window.label.unwrap_or_default(), window.percent.round()))
            .collect();
        assert_eq!(
            labels,
            [
                ("Gemini 3.5 Flash Lite".to_owned(), 25.0),
                ("Claude Sonnet 4.6 (Thinking)".to_owned(), 10.0),
                ("Claude Opus 4.6 (Thinking)".to_owned(), 0.0),
                ("GPT-OSS 120B (Medium)".to_owned(), 0.0),
            ]
        );
    }

    #[test]
    fn prefers_the_paid_plan_for_the_badge() {
        let plan = sample().plan.unwrap();
        assert_eq!(
            (plan.id.as_str(), plan.label.as_str()),
            ("antigravity_g1_plus_tier", "Google AI Plus")
        );
    }

    #[test]
    fn with_nothing_used_the_agent_default_leads() {
        let models: ModelsResponse = serde_json::from_value(json!({
            "defaultAgentModelId": "b",
            "agentModelSorts": [{ "groups": [{ "modelIds": ["a", "b"] }] }],
            "models": {
                "a": { "displayName": "A", "quotaInfo": { "remainingFraction": 1, "resetTime": "2026-11-11T00:00:00Z" } },
                "b": { "displayName": "B", "quotaInfo": { "remainingFraction": 1, "resetTime": "2026-11-11T00:00:00Z" } }
            }
        }))
        .unwrap();
        let snapshot = to_snapshot(&AssistResponse::default(), &models, String::new());
        assert_eq!(snapshot.headline.unwrap().label.as_deref(), Some("B"));
        assert_eq!(snapshot.windows.len(), 1);
        assert!(snapshot.plan.is_none());
    }
}
