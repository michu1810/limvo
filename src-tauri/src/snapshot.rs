//! Provider-neutral usage model shared with the UI.
//!
//! Mirrors `src/core/snapshot.ts`. Both sides are checked against
//! `fixtures/report.sample.json`, so a field renamed on one side fails a test.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WindowKind {
    Session,
    Weekly,
    /// A monthly allowance that follows the billing cycle.
    Monthly,
    /// A weekly cap of one model or feature, named by `label`.
    Scoped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Normal,
    Warning,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageWindow {
    /// Stable id, for example `weekly_all`, `session` or `scoped:fable`.
    pub id: String,
    pub kind: WindowKind,
    /// Name supplied by the provider; `None` lets the UI label it by kind.
    pub label: Option<String>,
    /// Share of the allowance used, 0 to 100.
    pub percent: f64,
    /// RFC 3339 instant of the next reset.
    pub resets_at: Option<String>,
    /// Length of the rolling window, used for pace.
    pub period_seconds: Option<u64>,
    pub severity: Severity,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BreakdownRow {
    pub key: String,
    pub label: String,
    /// Share of the headline window's usage, 0 to 100.
    pub percent: f64,
}

/// The plan the account is on, shown as a badge. Plan names are product names and are not translated.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanBadge {
    /// Stable id the interface styles by, for example `max_20x`.
    pub id: String,
    pub label: String,
}

/// Paid credits beyond the plan, when the user turned them on.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtraUsage {
    /// Amount spent, in minor units of `currency` (cents for `USD` with exponent 2).
    pub used_minor: i64,
    /// The monthly cap in the same units, or `None` when uncapped.
    pub limit_minor: Option<i64>,
    /// ISO 4217 code.
    pub currency: String,
    /// Number of minor-unit digits: the amount is `used_minor / 10^exponent`.
    pub exponent: u32,
    /// Share of the cap spent, 0 to 100, when there is a cap.
    pub percent: Option<f64>,
    pub severity: Severity,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageSnapshot {
    pub provider_id: String,
    pub provider_name: String,
    pub plan: Option<PlanBadge>,
    /// RFC 3339 instant the provider answered.
    pub fetched_at: String,
    pub headline: Option<UsageWindow>,
    pub windows: Vec<UsageWindow>,
    pub breakdown: Vec<BreakdownRow>,
    pub extra_usage: Option<ExtraUsage>,
    pub usage_page_url: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReportState {
    Loading,
    Ok,
    Offline,
    AuthRequired,
    NoCredentials,
    Error,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderReport {
    /// Which account this is about, known even before the first reading:
    /// the service id for the default account, `<service>~<n>` for an extra one.
    pub provider_id: String,
    /// The service, for its mark and plan colours.
    pub service_id: String,
    pub provider_name: String,
    pub state: ReportState,
    /// The last good reading, kept when a later refresh fails.
    pub snapshot: Option<UsageSnapshot>,
    /// Diagnostic detail for errors. Never contains credentials.
    pub detail: Option<String>,
}

impl ProviderReport {
    pub fn loading(provider_id: &str, service_id: &str, provider_name: &str) -> Self {
        Self {
            provider_id: provider_id.to_owned(),
            service_id: service_id.to_owned(),
            provider_name: provider_name.to_owned(),
            state: ReportState::Loading,
            snapshot: None,
            detail: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_the_contract_fixture_shared_with_the_ui() {
        let fixture = include_str!("../../fixtures/report.sample.json");
        let report: ProviderReport = serde_json::from_str(fixture).expect("fixture parses");
        let round_trip: serde_json::Value = serde_json::to_value(&report).unwrap();
        let original: serde_json::Value = serde_json::from_str(fixture).unwrap();
        assert_eq!(round_trip, original);
    }
}
