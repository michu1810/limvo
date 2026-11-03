//! The Claude plan a sign-in belongs to, as the apps record it next to the token.

use crate::snapshot::PlanBadge;

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
    fn has_no_badge_without_a_plan() {
        assert_eq!(label(None, Some("x")), None);
        assert_eq!(label(Some("  "), None), None);
    }
}
