//! Where a Claude sign-in can be found on this machine.
//!
//! Everything here is read-only. Tokens are never written, refreshed, logged
//! or sent anywhere except the usage endpoint. Refreshing would rotate the
//! refresh token and sign the user out of Claude Code, so an expired token is
//! reported as "sign in again" instead.

use std::fmt;
use std::path::PathBuf;

use serde::Deserialize;

/// The scope the usage endpoint requires.
pub const PROFILE_SCOPE: &str = "user:profile";

pub struct Token {
    secret: String,
    /// Unix milliseconds, if the source records an expiry.
    pub expires_at_ms: Option<i64>,
    /// Whether the token carries [`PROFILE_SCOPE`]; `None` when the source does not say.
    pub has_profile_scope: Option<bool>,
    /// Human-readable origin, for diagnostics.
    pub source: &'static str,
}

impl Token {
    pub(super) fn from_desktop(
        secret: String,
        expires_at_ms: Option<i64>,
        has_profile_scope: bool,
    ) -> Self {
        Self {
            secret,
            expires_at_ms,
            has_profile_scope: Some(has_profile_scope),
            source: "Claude desktop app",
        }
    }

    pub fn secret(&self) -> &str {
        &self.secret
    }

    fn is_expired(&self, now_ms: i64) -> bool {
        self.expires_at_ms.is_some_and(|expiry| expiry <= now_ms)
    }
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Token")
            .field("secret", &"<redacted>")
            .field("expires_at_ms", &self.expires_at_ms)
            .field("has_profile_scope", &self.has_profile_scope)
            .field("source", &self.source)
            .finish()
    }
}

#[derive(Debug, Default)]
pub struct Candidates {
    /// Tokens worth trying, best first.
    pub usable: Vec<Token>,
    /// At least one sign-in exists, even if none is usable right now.
    pub found_any: bool,
}

/// Every sign-in on this machine, ordered so the most likely to work comes first.
pub fn discover(now_ms: i64) -> Candidates {
    let mut all = cli_tokens();
    all.extend(super::desktop::tokens());
    rank(all, now_ms)
}

fn rank(all: Vec<Token>, now_ms: i64) -> Candidates {
    let found_any = !all.is_empty();
    let mut usable: Vec<Token> = all
        .into_iter()
        .filter(|token| !token.secret.is_empty())
        .filter(|token| !token.is_expired(now_ms))
        .filter(|token| token.has_profile_scope != Some(false))
        .collect();
    // Known-good scope first, then the token that lives longest.
    usable.sort_by_key(|token| {
        (
            std::cmp::Reverse(token.has_profile_scope == Some(true)),
            std::cmp::Reverse(token.expires_at_ms.unwrap_or(i64::MIN)),
        )
    });
    Candidates { usable, found_any }
}

/// Token written by the Claude Code CLI login, in `CLAUDE_CONFIG_DIR` or `~/.claude`.
fn cli_tokens() -> Vec<Token> {
    let Some(path) = cli_credentials_path() else {
        return Vec::new();
    };
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    parse_cli_credentials(&text).into_iter().collect()
}

fn cli_credentials_path() -> Option<PathBuf> {
    Some(claude_config_dir()?.join(".credentials.json"))
}

/// Claude Code's own folder: `CLAUDE_CONFIG_DIR` when set, otherwise `~/.claude`.
pub fn claude_config_dir() -> Option<PathBuf> {
    std::env::var_os("CLAUDE_CONFIG_DIR")
        .map(PathBuf::from)
        .or_else(|| std::env::home_dir().map(|home| home.join(".claude")))
}

#[derive(Deserialize)]
struct CliFile {
    #[serde(rename = "claudeAiOauth")]
    oauth: Option<CliOauth>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CliOauth {
    access_token: Option<String>,
    expires_at: Option<i64>,
    scopes: Option<Vec<String>>,
}

fn parse_cli_credentials(text: &str) -> Option<Token> {
    let oauth = serde_json::from_str::<CliFile>(text).ok()?.oauth?;
    Some(Token {
        secret: oauth.access_token?,
        // Zero means the CLI never recorded an expiry.
        expires_at_ms: oauth.expires_at.filter(|expiry| *expiry > 0),
        has_profile_scope: oauth
            .scopes
            .map(|scopes| scopes.iter().any(|scope| scope == PROFILE_SCOPE)),
        source: "Claude Code CLI",
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn token(expires_at_ms: Option<i64>, scope: Option<bool>, source: &'static str) -> Token {
        Token {
            secret: "x".into(),
            expires_at_ms,
            has_profile_scope: scope,
            source,
        }
    }

    #[test]
    fn drops_expired_and_unscoped_tokens_but_remembers_they_existed() {
        let ranked = rank(
            vec![
                token(Some(10), Some(true), "expired"),
                token(Some(2_000), Some(false), "no scope"),
            ],
            1_000,
        );
        assert!(ranked.usable.is_empty());
        assert!(ranked.found_any);
    }

    #[test]
    fn prefers_a_known_scope_then_the_longest_lived_token() {
        let ranked = rank(
            vec![
                token(Some(9_000), None, "unknown scope"),
                token(Some(5_000), Some(true), "short"),
                token(Some(8_000), Some(true), "long"),
            ],
            1_000,
        );
        let order: Vec<_> = ranked.usable.iter().map(|token| token.source).collect();
        assert_eq!(order, ["long", "short", "unknown scope"]);
    }

    #[test]
    fn reads_the_cli_file_and_treats_a_zero_expiry_as_unknown() {
        let token = parse_cli_credentials(
            r#"{"claudeAiOauth": {"accessToken": "abc", "expiresAt": 0, "scopes": ["user:inference", "user:profile"]}}"#,
        )
        .unwrap();
        assert_eq!(token.secret(), "abc");
        assert_eq!(token.expires_at_ms, None);
        assert_eq!(token.has_profile_scope, Some(true));
    }

    #[test]
    fn never_prints_the_secret() {
        let printed = format!("{:?}", token(None, None, "test"));
        assert!(printed.contains("<redacted>"));
        assert!(!printed.contains("\"x\""));
    }
}
