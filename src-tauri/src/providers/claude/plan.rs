//! The Claude plan an account is on.
//!
//! The apps record the plan next to each token, but only as it was when the
//! token was issued: an upgrade does not show there until the next sign-in.
//! The account profile is current, so it wins whenever it can be read.

use serde::Deserialize;

use crate::snapshot::PlanBadge;

/// The account profile, read with the same token as the usage.
pub const PROFILE_URL: &str = "https://api.anthropic.com/api/oauth/profile";

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Profile {
    organization: Option<Organization>,
}

// The field names follow the API.
#[allow(clippy::struct_field_names)]
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct Organization {
    organization_type: Option<String>,
    rate_limit_tier: Option<String>,
    /// The name claude.ai shows, such as "Pro" or "Ultra".
    plan_display_name: Option<String>,
}

/// The badge for the profile: the plan's own name, coloured by its limits tier.
pub fn from_profile(profile: &Profile) -> Option<PlanBadge> {
    let organization = profile.organization.as_ref()?;
    let subscription = organization
        .organization_type
        .as_deref()
        .map(|kind| kind.trim_start_matches("claude_"));
    let mut badge = badge(subscription, organization.rate_limit_tier.as_deref())?;
    if let Some(name) = organization
        .plan_display_name
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty())
    {
        name.clone_into(&mut badge.label);
    }
    Some(badge)
}

/// Builds the badge from the `subscriptionType` and `rateLimitTier` fields.
/// Unknown values are shown as they are rather than hidden, so a new plan
/// still gets a badge.
pub fn badge(subscription: Option<&str>, tier: Option<&str>) -> Option<PlanBadge> {
    let subscription = subscription?.trim().to_ascii_lowercase();
    if subscription.is_empty() {
        return None;
    }
    let tier = tier.unwrap_or_default().to_ascii_lowercase();
    let (id, label) = match subscription.as_str() {
        "max" if tier.contains("20x") => ("max_20x", "Max 20\u{d7}"),
        "max" if tier.contains("5x") => ("max_5x", "Max 5\u{d7}"),
        "max" => ("max", "Max"),
        "pro" => ("pro", "Pro"),
        "team" => ("team", "Team"),
        "enterprise" => ("enterprise", "Enterprise"),
        "free" => ("free", "Free"),
        other => {
            return Some(PlanBadge {
                id: other.to_owned(),
                label: title_case(other),
            });
        }
    };
    Some(PlanBadge {
        id: id.to_owned(),
        label: label.to_owned(),
    })
}

fn title_case(value: &str) -> String {
    let mut chars = value.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(chars).collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn label(subscription: Option<&str>, tier: Option<&str>) -> Option<String> {
        badge(subscription, tier).map(|plan| plan.label)
    }

    #[test]
    fn tells_the_max_tiers_apart() {
        assert_eq!(
            label(Some("max"), Some("default_claude_max_20x")).as_deref(),
            Some("Max 20\u{d7}")
        );
        assert_eq!(
            label(Some("max"), Some("default_claude_max_5x")).as_deref(),
            Some("Max 5\u{d7}")
        );
        assert_eq!(label(Some("max"), None).as_deref(), Some("Max"));
    }

    #[test]
    fn names_the_other_plans() {
        assert_eq!(
            label(Some("pro"), Some("default_claude_ai")).as_deref(),
            Some("Pro")
        );
        assert_eq!(label(Some("Team"), None).as_deref(), Some("Team"));
    }

    #[test]
    fn keeps_plans_it_does_not_know() {
        assert_eq!(label(Some("studio"), None).as_deref(), Some("Studio"));
    }

    #[test]
    fn the_profile_names_the_plan_and_its_tier_picks_the_colour() {
        // The shape of a real answer: a Max 20x account that claude.ai calls Ultra.
        let profile: Profile = serde_json::from_str(
            r#"{"account": {"has_claude_max": true},
                "organization": {"organization_type": "claude_max",
                                 "rate_limit_tier": "default_claude_max_20x",
                                 "plan_display_name": "Ultra",
                                 "plan_display_label": "Ultra plan"}}"#,
        )
        .unwrap();
        let plan = from_profile(&profile).unwrap();
        assert_eq!(
            (plan.id.as_str(), plan.label.as_str()),
            ("max_20x", "Ultra")
        );
    }

    #[test]
    fn a_profile_without_a_display_name_keeps_the_tier_label() {
        let profile: Profile = serde_json::from_str(
            r#"{"organization": {"organization_type": "claude_pro", "rate_limit_tier": "default_claude_ai"}}"#,
        )
        .unwrap();
        let plan = from_profile(&profile).unwrap();
        assert_eq!((plan.id.as_str(), plan.label.as_str()), ("pro", "Pro"));
        assert!(from_profile(&Profile::default()).is_none());
    }

    #[test]
    fn has_no_badge_without_a_plan() {
        assert_eq!(label(None, Some("x")), None);
        assert_eq!(label(Some("  "), None), None);
    }
}
