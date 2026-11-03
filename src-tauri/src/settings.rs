//! User preferences, stored as JSON in the app's config folder.
//!
//! Unknown fields are kept when saving, so a newer version's settings survive
//! a run of an older one. Changes arrive as a [`SettingsPatch`]: only the
//! fields a request names are touched, so a stale window can never reset
//! options it does not show.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Deserializer, Serialize};

/// How often the provider is read.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum RefreshMode {
    /// About once a minute while Claude Code is in use, every few minutes otherwise.
    #[default]
    #[serde(rename = "adaptive")]
    Adaptive,
    #[serde(rename = "2m")]
    Every2Minutes,
    #[serde(rename = "5m")]
    Every5Minutes,
    #[serde(rename = "15m")]
    Every15Minutes,
}

/// Top-left corner of the desktop card, in physical screen pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CardPosition {
    pub x: i32,
    pub y: i32,
}

// Each switch in the settings panel is one independent on/off preference.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    /// BCP 47 code, or `None` to follow the system language.
    pub locale: Option<String>,
    /// The reading docked in the taskbar.
    pub show_strip: bool,
    /// Windows notifications when a limit gets close, is reached or resets.
    pub notifications: bool,
    pub refresh: RefreshMode,
    /// The card pinned to the desktop.
    pub desktop_card: bool,
    pub card_position: Option<CardPosition>,
    /// The system-wide shortcut that opens the flyout.
    pub shortcut: bool,
    /// Tint the interface with the colour of the account's plan.
    pub plan_colors: bool,
    #[serde(flatten)]
    other: serde_json::Map<String, serde_json::Value>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            locale: None,
            show_strip: true,
            notifications: true,
            refresh: RefreshMode::Adaptive,
            desktop_card: false,
            card_position: None,
            shortcut: true,
            plan_colors: true,
            other: serde_json::Map::new(),
        }
    }
}

/// A partial update. A missing field means "leave as is"; for nullable
/// fields an explicit `null` clears the value, hence the nested options.
#[allow(clippy::option_option)]
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SettingsPatch {
    #[serde(default, deserialize_with = "present")]
    pub locale: Option<Option<String>>,
    pub show_strip: Option<bool>,
    pub notifications: Option<bool>,
    pub refresh: Option<RefreshMode>,
    pub desktop_card: Option<bool>,
    #[serde(default, deserialize_with = "present")]
    pub card_position: Option<Option<CardPosition>>,
    pub shortcut: Option<bool>,
    pub plan_colors: Option<bool>,
}

/// Turns a field that is present into `Some`, even when its value is `null`.
#[allow(clippy::option_option)]
fn present<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer).map(Some)
}

impl Settings {
    pub fn apply(&mut self, patch: SettingsPatch) {
        if let Some(locale) = patch.locale {
            self.locale = locale;
        }
        if let Some(show_strip) = patch.show_strip {
            self.show_strip = show_strip;
        }
        if let Some(notifications) = patch.notifications {
            self.notifications = notifications;
        }
        if let Some(refresh) = patch.refresh {
            self.refresh = refresh;
        }
        if let Some(desktop_card) = patch.desktop_card {
            self.desktop_card = desktop_card;
        }
        if let Some(card_position) = patch.card_position {
            self.card_position = card_position;
        }
        if let Some(shortcut) = patch.shortcut {
            self.shortcut = shortcut;
        }
        if let Some(plan_colors) = patch.plan_colors {
            self.plan_colors = plan_colors;
        }
    }
}

#[derive(Debug)]
pub struct SettingsStore {
    path: PathBuf,
}

impl SettingsStore {
    pub fn new(config_dir: &Path) -> Self {
        Self {
            path: config_dir.join("settings.json"),
        }
    }

    /// A missing or unreadable file yields the defaults; a corrupt one is logged and kept on disk.
    pub fn load(&self) -> Settings {
        match std::fs::read_to_string(&self.path) {
            Ok(text) => serde_json::from_str(&text).unwrap_or_else(|error| {
                log::warn!(
                    "ignoring unreadable settings at {}: {error}",
                    self.path.display()
                );
                Settings::default()
            }),
            Err(_) => Settings::default(),
        }
    }

    /// Writes to a temporary file first, so a crash never leaves half a file behind.
    pub fn save(&self, settings: &Settings) -> std::io::Result<()> {
        if let Some(dir) = self.path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let temporary = self.path.with_extension("json.tmp");
        std::fs::write(&temporary, serde_json::to_vec_pretty(settings)?)?;
        std::fs::rename(&temporary, &self.path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_fields_it_does_not_know() {
        let settings: Settings =
            serde_json::from_str(r#"{"locale": "pl", "futureOption": true}"#).unwrap();
        let saved = serde_json::to_value(&settings).unwrap();
        assert_eq!(saved["locale"], "pl");
        assert_eq!(saved["futureOption"], true);
    }

    #[test]
    fn fills_new_fields_with_defaults_for_older_files() {
        let settings: Settings = serde_json::from_str(r#"{"locale": "pl"}"#).unwrap();
        assert!(settings.show_strip && settings.notifications);
        assert_eq!(settings.refresh, RefreshMode::Adaptive);
    }

    #[test]
    fn a_patch_touches_only_the_fields_it_names() {
        let mut settings = Settings {
            locale: Some("he".into()),
            refresh: RefreshMode::Every15Minutes,
            ..Settings::default()
        };
        let patch: SettingsPatch = serde_json::from_str(r#"{"notifications": false}"#).unwrap();
        settings.apply(patch);
        assert_eq!(settings.locale.as_deref(), Some("he"));
        assert_eq!(settings.refresh, RefreshMode::Every15Minutes);
        assert!(!settings.notifications);
    }

    #[test]
    fn an_explicit_null_clears_a_nullable_field() {
        let mut settings = Settings {
            locale: Some("he".into()),
            ..Settings::default()
        };
        settings.apply(serde_json::from_str(r#"{"locale": null}"#).unwrap());
        assert_eq!(settings.locale, None);
    }

    #[test]
    fn rejects_unknown_patch_fields() {
        assert!(serde_json::from_str::<SettingsPatch>(r#"{"lcoale": "pl"}"#).is_err());
    }

    #[test]
    fn round_trips_through_disk() {
        let dir = std::env::temp_dir().join(format!("limvo-settings-{}", std::process::id()));
        let store = SettingsStore::new(&dir);
        let settings = Settings {
            locale: Some("he".into()),
            ..Settings::default()
        };
        store.save(&settings).unwrap();
        assert_eq!(store.load(), settings);
        let _ = std::fs::remove_dir_all(dir);
    }
}
