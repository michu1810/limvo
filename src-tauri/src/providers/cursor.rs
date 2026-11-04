//! Cursor limits, read with the sign-in the Cursor editor keeps.
//!
//! Cursor stores its session in the editor's settings database
//! (`state.vscdb`, a SQLite file) and reports usage per billing cycle: the
//! share of the included usage spent so far, split into _Auto_ and the named
//! models (_API_). The database is opened read-only and the token is never
//! refreshed or written, so Limvo can never sign the user out of Cursor.

use std::path::PathBuf;

use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use rusqlite::{Connection, OpenFlags, OptionalExtension, types::Value};
use serde::Deserialize;

use super::{FetchFuture, ProviderError, UsageProvider, clamp_percent, severity_of, status_error};
use crate::snapshot::{PlanBadge, UsageSnapshot, UsageWindow, WindowKind};

pub const PROVIDER_ID: &str = "cursor";
const API: &str = "https://api2.cursor.sh/aiserver.v1.DashboardService";
const USAGE_PAGE_URL: &str = "https://cursor.com/dashboard?tab=usage";
const TOKEN_KEY: &str = "cursorAuth/accessToken";

#[derive(Debug, Default)]
pub struct CursorProvider;

impl CursorProvider {
    pub fn new() -> Self {
        Self
    }
}

impl UsageProvider for CursorProvider {
    fn service(&self) -> &'static str {
        PROVIDER_ID
    }

    fn name(&self) -> &'static str {
        "Cursor"
    }

    fn fetch<'a>(&'a self, http: &'a reqwest::Client) -> FetchFuture<'a> {
        Box::pin(fetch(http))
    }

    /// The editor saves each workspace's chats and state here while in use.
    fn activity_paths(&self) -> Vec<PathBuf> {
        user_dir()
            .map(|dir| dir.join("workspaceStorage"))
            .into_iter()
            .collect()
    }
}

/// `%APPDATA%\Cursor\User`, where the editor keeps its settings.
fn user_dir() -> Option<PathBuf> {
    std::env::var_os("APPDATA")
        .filter(|value| !value.is_empty())
        .map(|appdata| PathBuf::from(appdata).join("Cursor").join("User"))
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

fn read_sign_in() -> Option<SignIn> {
    let database = user_dir()?.join("globalStorage").join("state.vscdb");
    if !database.is_file() {
        return None;
    }
    let token = read_token(&database).ok().flatten()?;
    Some(SignIn { token })
}

/// Reads the token without ever writing to the editor's database.
fn read_token(database: &std::path::Path) -> rusqlite::Result<Option<String>> {
    let flags = OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX;
    // While Cursor runs the database is in WAL mode and readable as is. When it
    // is closed and the journal files are gone, `immutable` reads the file alone.
    let connection = Connection::open_with_flags(database, flags).or_else(|_| {
        let uri = format!("file:{}?immutable=1", database.display()).replace('\\', "/");
        Connection::open_with_flags(uri, flags | OpenFlags::SQLITE_OPEN_URI)
    })?;
    let value: Option<Value> = connection
        .query_row(
            "SELECT value FROM ItemTable WHERE key = ?1",
            [TOKEN_KEY],
            |row| row.get(0),
        )
        .optional()?;
    Ok(match value {
        Some(Value::Text(text)) => Some(text),
        Some(Value::Blob(bytes)) => String::from_utf8(bytes).ok(),
        _ => None,
    }
    .map(|token| token.trim().trim_matches('"').to_owned())
    .filter(|token| !token.is_empty()))
}

async fn fetch(http: &reqwest::Client) -> Result<UsageSnapshot, ProviderError> {
    let sign_in = read_sign_in().ok_or(ProviderError::NoCredentials)?;
    let usage: UsageResponse = call(http, &sign_in, "GetCurrentPeriodUsage").await?;
    // The plan only names the badge; a failure there must not hide the reading.
    let plan: Option<PlanResponse> = call(http, &sign_in, "GetPlanInfo").await.ok();
    Ok(to_snapshot(usage, plan, jiff::Timestamp::now().to_string()))
}

/// One Connect call: a JSON `POST` with an empty request message.
async fn call<T: for<'de> Deserialize<'de>>(
    http: &reqwest::Client,
    sign_in: &SignIn,
    method: &str,
) -> Result<T, ProviderError> {
    let response = http
        .post(format!("{API}/{method}"))
        .header(AUTHORIZATION, format!("Bearer {}", sign_in.token))
        .header(CONTENT_TYPE, "application/json")
        .header("Connect-Protocol-Version", "1")
        .body("{}")
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
struct UsageResponse {
    /// Milliseconds since the epoch, sent as a string.
    billing_cycle_start: Option<String>,
    billing_cycle_end: Option<String>,
    plan_usage: Option<PlanUsage>,
}

// The field names follow the API.
#[allow(clippy::struct_field_names)]
#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct PlanUsage {
    total_percent_used: Option<f64>,
    auto_percent_used: Option<f64>,
    api_percent_used: Option<f64>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct PlanResponse {
    plan_info: Option<PlanInfo>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct PlanInfo {
    plan_name: Option<String>,
}

fn to_snapshot(
    usage: UsageResponse,
    plan: Option<PlanResponse>,
    fetched_at: String,
) -> UsageSnapshot {
    let start = parse_millis(usage.billing_cycle_start.as_deref());
    let end = parse_millis(usage.billing_cycle_end.as_deref());
    let resets_at = end
        .and_then(|millis| jiff::Timestamp::from_millisecond(millis).ok())
        .map(|instant| instant.to_string());
    let period_seconds = match (start, end) {
        (Some(start), Some(end)) if end > start => u64::try_from((end - start) / 1000).ok(),
        _ => None,
    };
    let window = |id: &str, label: Option<&str>, percent: f64| {
        let percent = clamp_percent(percent);
        UsageWindow {
            id: id.to_owned(),
            kind: WindowKind::Monthly,
            label: label.map(str::to_owned),
            percent,
            resets_at: resets_at.clone(),
            period_seconds,
            severity: severity_of(percent),
        }
    };

    let plan_usage = usage.plan_usage.unwrap_or_default();
    let headline = plan_usage
        .total_percent_used
        .map(|percent| window("monthly_included", None, percent));
    // Auto and the named models draw from the same included usage; both are worth a row.
    let windows = [
        ("monthly:auto", "Auto", plan_usage.auto_percent_used),
        ("monthly:api", "API", plan_usage.api_percent_used),
    ]
    .into_iter()
    .filter_map(|(id, label, percent)| Some(window(id, Some(label), percent?)))
    .collect();

    UsageSnapshot {
        provider_id: PROVIDER_ID.to_owned(),
        provider_name: "Cursor".to_owned(),
        plan: plan
            .and_then(|plan| plan.plan_info?.plan_name)
            .and_then(|name| plan_badge(&name)),
        fetched_at,
        headline,
        windows,
        breakdown: Vec::new(),
        extra_usage: None,
        usage_page_url: Some(USAGE_PAGE_URL.to_owned()),
    }
}

fn parse_millis(value: Option<&str>) -> Option<i64> {
    value?.trim().parse().ok()
}

/// Cursor names its plans itself ("Free", "Pro", "Pro+", "Ultra", "Business").
fn plan_badge(name: &str) -> Option<PlanBadge> {
    let label = name.trim();
    if label.is_empty() {
        return None;
    }
    let slug: String = label
        .to_ascii_lowercase()
        .replace('+', "_plus")
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    Some(PlanBadge {
        id: format!("cursor_{slug}"),
        label: label.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::snapshot::Severity;

    // The shape of real answers; the numbers are made up so every branch is exercised.
    fn sample() -> UsageSnapshot {
        let usage =
            serde_json::from_str(include_str!("../../../fixtures/cursor-usage.sample.json"))
                .unwrap();
        let plan = serde_json::from_str(include_str!("../../../fixtures/cursor-plan.sample.json"))
            .unwrap();
        to_snapshot(usage, Some(plan), "2026-11-04T00:00:00Z".to_owned())
    }

    #[test]
    fn the_included_usage_is_the_monthly_headline() {
        let headline = sample().headline.expect("headline");
        assert_eq!(headline.kind, WindowKind::Monthly);
        assert!((headline.percent - 42.5).abs() < f64::EPSILON);
        assert_eq!(
            headline.resets_at.as_deref(),
            Some("2026-11-26T11:46:12.12Z")
        );
        assert_eq!(headline.period_seconds, Some(2_678_400));
    }

    #[test]
    fn auto_and_named_models_are_other_limits() {
        let windows = sample().windows;
        let rows: Vec<_> = windows
            .iter()
            .map(|window| (window.label.as_deref(), window.percent, window.severity))
            .collect();
        assert_eq!(
            rows,
            [
                (Some("Auto"), 30.0, Severity::Normal),
                (Some("API"), 85.0, Severity::Warning),
            ]
        );
    }

    #[test]
    fn names_the_plan() {
        let plan = sample().plan.unwrap();
        assert_eq!(
            (plan.id.as_str(), plan.label.as_str()),
            ("cursor_pro_plus", "Pro+")
        );
    }

    #[test]
    fn an_empty_answer_has_no_headline() {
        let snapshot = to_snapshot(UsageResponse::default(), None, String::new());
        assert!(snapshot.headline.is_none());
        assert_eq!(snapshot.windows, []);
        assert!(snapshot.plan.is_none());
    }

    #[test]
    fn reads_the_token_from_a_settings_database() {
        let dir = std::env::temp_dir().join(format!("limvo-cursor-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("state.vscdb");
        let connection = Connection::open(&path).unwrap();
        connection
            .execute_batch(
                "CREATE TABLE ItemTable (key TEXT UNIQUE ON CONFLICT REPLACE, value BLOB);
                 INSERT INTO ItemTable VALUES ('cursorAuth/accessToken', 'header.payload.signature');",
            )
            .unwrap();
        drop(connection);
        assert_eq!(
            read_token(&path).unwrap().as_deref(),
            Some("header.payload.signature")
        );
        let _ = std::fs::remove_dir_all(dir);
    }
}
