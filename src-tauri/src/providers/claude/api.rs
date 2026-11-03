//! The `/api/oauth/usage` response and its mapping to a [`UsageSnapshot`].
//!
//! The endpoint is undocumented, so every field is optional and unknown
//! fields are ignored: a change on the service side should degrade one row,
//! not the whole reading.

use serde::Deserialize;

use crate::snapshot::{BreakdownRow, Severity, UsageSnapshot, UsageWindow, WindowKind};

pub const USAGE_URL: &str = "https://api.anthropic.com/api/oauth/usage";
pub const USAGE_PAGE_URL: &str = "https://claude.ai/settings/usage";
pub const OAUTH_BETA: &str = "oauth-2025-04-20";

const SESSION_SECONDS: u64 = 5 * 3600;
const WEEK_SECONDS: u64 = 7 * 24 * 3600;

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct UsageResponse {
    five_hour: Option<Bucket>,
    seven_day: Option<Bucket>,
    limits: Vec<Limit>,
    seven_day_breakdown: Option<Breakdown>,
}

#[derive(Debug, Deserialize)]
struct Bucket {
    utilization: Option<f64>,
    resets_at: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct Limit {
    kind: Option<String>,
    percent: Option<f64>,
    severity: Option<String>,
    resets_at: Option<String>,
    scope: Option<LimitScope>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct LimitScope {
    model: Option<LimitModel>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct LimitModel {
    display_name: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct Breakdown {
    rows: Vec<BreakdownEntry>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct BreakdownEntry {
    key: Option<String>,
    display_name: Option<String>,
    percent: Option<f64>,
}

/// Maps a response into the neutral model. `fetched_at` is passed in so the
/// mapping stays pure and testable.
pub fn to_snapshot(response: UsageResponse, fetched_at: String) -> UsageSnapshot {
    let weekly_limit = response
        .limits
        .iter()
        .find(|limit| limit.kind.as_deref() == Some("weekly_all"));

    let weekly = response.seven_day.as_ref().and_then(|bucket| {
        window_from_bucket(
            bucket,
            "weekly_all",
            WindowKind::Weekly,
            WEEK_SECONDS,
            weekly_limit,
        )
    });
    let session = response.five_hour.as_ref().and_then(|bucket| {
        window_from_bucket(
            bucket,
            "session",
            WindowKind::Session,
            SESSION_SECONDS,
            None,
        )
    });

    let scoped: Vec<UsageWindow> = response
        .limits
        .iter()
        .filter(|limit| limit.kind.as_deref() == Some("weekly_scoped"))
        .filter_map(|limit| scoped_window(limit, weekly.as_ref()))
        .collect();

    // The weekly window is the headline when the plan has one; otherwise the session is.
    let (headline, windows) = match weekly {
        Some(weekly) => (Some(weekly), session.into_iter().chain(scoped).collect()),
        None => (session, scoped),
    };

    let breakdown = response
        .seven_day_breakdown
        .map(|breakdown| {
            breakdown
                .rows
                .into_iter()
                .filter_map(breakdown_row)
                .collect()
        })
        .unwrap_or_default();

    UsageSnapshot {
        provider_id: super::PROVIDER_ID.to_owned(),
        provider_name: "Claude".to_owned(),
        // The plan comes with the sign-in, not with the usage answer.
        plan: None,
        fetched_at,
        headline,
        windows,
        breakdown,
        usage_page_url: Some(USAGE_PAGE_URL.to_owned()),
    }
}

fn window_from_bucket(
    bucket: &Bucket,
    id: &str,
    kind: WindowKind,
    period_seconds: u64,
    limit: Option<&Limit>,
) -> Option<UsageWindow> {
    let percent = clamp_percent(bucket.utilization?);
    Some(UsageWindow {
        id: id.to_owned(),
        kind,
        label: None,
        percent,
        resets_at: bucket.resets_at.clone(),
        period_seconds: Some(period_seconds),
        severity: severity(limit.and_then(|limit| limit.severity.as_deref()), percent),
    })
}

/// A model-specific weekly cap, such as "Fable". Entries without a name are
/// placeholders on the service side and are skipped.
fn scoped_window(limit: &Limit, weekly: Option<&UsageWindow>) -> Option<UsageWindow> {
    let name = limit
        .scope
        .as_ref()?
        .model
        .as_ref()?
        .display_name
        .as_deref()?
        .trim();
    if name.is_empty() {
        return None;
    }
    let percent = clamp_percent(limit.percent.unwrap_or(0.0));
    Some(UsageWindow {
        id: format!("scoped:{}", slug(name)),
        kind: WindowKind::Scoped,
        label: Some(name.to_owned()),
        percent,
        // Scoped caps share the weekly clock when they do not report their own.
        resets_at: limit
            .resets_at
            .clone()
            .or_else(|| weekly.and_then(|weekly| weekly.resets_at.clone())),
        period_seconds: Some(WEEK_SECONDS),
        severity: severity(limit.severity.as_deref(), percent),
    })
}

fn breakdown_row(entry: BreakdownEntry) -> Option<BreakdownRow> {
    let key = entry.key?;
    let label = entry
        .display_name
        .filter(|name| !name.trim().is_empty())
        .unwrap_or_else(|| key.clone());
    Some(BreakdownRow {
        key,
        label,
        percent: clamp_percent(entry.percent.unwrap_or(0.0)),
    })
}

/// Uses the service's own judgement when it gives one we know, otherwise derives it.
fn severity(reported: Option<&str>, percent: f64) -> Severity {
    match reported {
        Some("normal") => Severity::Normal,
        Some("warning" | "elevated" | "high") => Severity::Warning,
        Some("critical" | "exceeded" | "blocked") => Severity::Critical,
        _ if percent >= 100.0 => Severity::Critical,
        _ if percent >= 80.0 => Severity::Warning,
        _ => Severity::Normal,
    }
}

fn clamp_percent(value: f64) -> f64 {
    if value.is_finite() {
        value.clamp(0.0, 100.0)
    } else {
        0.0
    }
}

fn slug(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> UsageSnapshot {
        let response: UsageResponse = serde_json::from_str(include_str!(
            "../../../../fixtures/claude-usage.sample.json"
        ))
        .unwrap();
        to_snapshot(response, "2026-11-08T14:57:00Z".to_owned())
    }

    #[test]
    fn weekly_window_is_the_headline() {
        let snapshot = sample();
        let headline = snapshot.headline.expect("headline");
        assert_eq!(headline.id, "weekly_all");
        assert_eq!(headline.kind, WindowKind::Weekly);
        assert!((headline.percent - 51.0).abs() < f64::EPSILON);
        assert_eq!(headline.period_seconds, Some(WEEK_SECONDS));
        assert_eq!(headline.severity, Severity::Normal);
    }

    #[test]
    fn named_model_caps_become_other_limits_and_unnamed_ones_are_skipped() {
        let snapshot = sample();
        let ids: Vec<_> = snapshot
            .windows
            .iter()
            .map(|window| window.id.as_str())
            .collect();
        assert_eq!(ids, ["scoped:fable"]);
        assert_eq!(snapshot.windows[0].label.as_deref(), Some("Fable"));
    }

    #[test]
    fn breakdown_keeps_every_product_row() {
        let snapshot = sample();
        let labels: Vec<_> = snapshot
            .breakdown
            .iter()
            .map(|row| row.label.as_str())
            .collect();
        assert_eq!(labels, ["Claude Code", "Cowork", "Chats", "Other"]);
    }

    #[test]
    fn session_takes_the_headline_when_there_is_no_weekly_window() {
        let response: UsageResponse = serde_json::from_str(
            r#"{"five_hour": {"utilization": 12.5, "resets_at": "2026-11-08T18:00:00Z"}, "seven_day": null}"#,
        )
        .unwrap();
        let snapshot = to_snapshot(response, String::new());
        assert_eq!(
            snapshot.headline.map(|window| window.kind),
            Some(WindowKind::Session)
        );
        assert_eq!(snapshot.windows, []);
    }

    #[test]
    fn tolerates_an_empty_or_unfamiliar_body() {
        let response: UsageResponse =
            serde_json::from_str(r#"{"something_new": [1, 2, 3]}"#).unwrap();
        let snapshot = to_snapshot(response, String::new());
        assert!(snapshot.headline.is_none());
        assert!(snapshot.windows.is_empty() && snapshot.breakdown.is_empty());
    }

    #[test]
    fn clamps_out_of_range_numbers() {
        assert!((clamp_percent(140.0) - 100.0).abs() < f64::EPSILON);
        assert!(clamp_percent(-3.0).abs() < f64::EPSILON);
        assert!(clamp_percent(f64::NAN).abs() < f64::EPSILON);
    }

    #[test]
    fn derives_severity_when_the_service_does_not_say() {
        assert_eq!(severity(None, 100.0), Severity::Critical);
        assert_eq!(severity(None, 85.0), Severity::Warning);
        assert_eq!(severity(Some("unheard-of"), 10.0), Severity::Normal);
    }
}
