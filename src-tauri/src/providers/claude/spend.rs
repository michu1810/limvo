//! Extra usage: paid credits that carry the account past its plan limits.
//!
//! The usage answer describes them twice. The newer `spend` object gives
//! amounts in minor units with their own exponent, so the currency is
//! self-describing. The older `extra_usage` object is read only when `spend`
//! is missing. Either way, nothing is shown unless the user turned it on.

use serde::Deserialize;

use crate::snapshot::{ExtraUsage, Severity};

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Spend {
    enabled: bool,
    used: Option<Money>,
    limit: Option<Money>,
    percent: Option<f64>,
    severity: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Money {
    amount_minor: i64,
    currency: Option<String>,
    #[serde(default)]
    exponent: u32,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct LegacyExtraUsage {
    is_enabled: bool,
    monthly_limit: Option<i64>,
    used_credits: Option<i64>,
    utilization: Option<f64>,
    currency: Option<String>,
    decimal_places: Option<u32>,
}

pub fn extra_usage(spend: Option<&Spend>, legacy: Option<&LegacyExtraUsage>) -> Option<ExtraUsage> {
    match spend {
        Some(spend) => from_spend(spend),
        None => legacy.and_then(from_legacy),
    }
}

fn from_spend(spend: &Spend) -> Option<ExtraUsage> {
    if !spend.enabled {
        return None;
    }
    let used = spend.used.as_ref()?;
    let limit_minor = spend.limit.as_ref().map(|limit| limit.amount_minor);
    let percent = spend
        .percent
        .or_else(|| ratio(used.amount_minor, limit_minor))
        .map(|value| value.clamp(0.0, 100.0));
    Some(ExtraUsage {
        used_minor: used.amount_minor.max(0),
        limit_minor,
        currency: used.currency.clone().unwrap_or_else(|| "USD".to_owned()),
        exponent: used.exponent,
        percent,
        severity: severity(spend.severity.as_deref(), percent),
    })
}

fn from_legacy(legacy: &LegacyExtraUsage) -> Option<ExtraUsage> {
    if !legacy.is_enabled {
        return None;
    }
    let used_minor = legacy.used_credits.unwrap_or(0).max(0);
    let percent = legacy
        .utilization
        .or_else(|| ratio(used_minor, legacy.monthly_limit))
        .map(|value| value.clamp(0.0, 100.0));
    Some(ExtraUsage {
        used_minor,
        limit_minor: legacy.monthly_limit,
        currency: legacy.currency.clone().unwrap_or_else(|| "USD".to_owned()),
        exponent: legacy.decimal_places.unwrap_or(2),
        percent,
        severity: severity(None, percent),
    })
}

#[allow(clippy::cast_precision_loss)]
fn ratio(used: i64, limit: Option<i64>) -> Option<f64> {
    let limit = limit.filter(|limit| *limit > 0)?;
    Some(used as f64 / limit as f64 * 100.0)
}

fn severity(reported: Option<&str>, percent: Option<f64>) -> Severity {
    match (reported, percent) {
        (Some("warning" | "elevated" | "high"), _) => Severity::Warning,
        (Some("critical" | "exceeded" | "blocked"), _) => Severity::Critical,
        (_, Some(value)) if value >= 100.0 => Severity::Critical,
        (_, Some(value)) if value >= 80.0 => Severity::Warning,
        _ => Severity::Normal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spend(json: &str) -> Spend {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn hides_extra_usage_that_is_turned_off() {
        let off = spend(
            r#"{"enabled": false, "used": {"amount_minor": 0, "currency": "USD", "exponent": 2}}"#,
        );
        assert_eq!(extra_usage(Some(&off), None), None);
    }

    #[test]
    fn reads_amounts_in_minor_units() {
        let on = spend(
            r#"{"enabled": true, "used": {"amount_minor": 1240, "currency": "EUR", "exponent": 2},
                "limit": {"amount_minor": 5000, "currency": "EUR", "exponent": 2}, "severity": "normal"}"#,
        );
        let extra = extra_usage(Some(&on), None).unwrap();
        assert_eq!((extra.used_minor, extra.limit_minor), (1240, Some(5000)));
        assert_eq!(extra.currency, "EUR");
        assert!((extra.percent.unwrap() - 24.8).abs() < 1e-9);
        assert_eq!(extra.severity, Severity::Normal);
    }

    #[test]
    fn has_no_percentage_without_a_cap() {
        let uncapped = spend(
            r#"{"enabled": true, "used": {"amount_minor": 300, "currency": "USD", "exponent": 2}}"#,
        );
        assert_eq!(extra_usage(Some(&uncapped), None).unwrap().percent, None);
    }

    #[test]
    fn falls_back_to_the_older_shape() {
        let legacy: LegacyExtraUsage = serde_json::from_str(
            r#"{"is_enabled": true, "monthly_limit": 2000, "used_credits": 1900, "currency": "USD", "decimal_places": 2}"#,
        )
        .unwrap();
        let extra = extra_usage(None, Some(&legacy)).unwrap();
        assert_eq!(extra.severity, Severity::Warning);
        assert!((extra.percent.unwrap() - 95.0).abs() < 1e-9);
    }
}
