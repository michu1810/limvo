//! User preferences, stored as JSON in the app's config folder.
//!
//! Unknown fields are kept when saving, so a newer version's settings survive
//! a run of an older one.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    /// BCP 47 code, or `None` to follow the system language.
    pub locale: Option<String>,
    #[serde(flatten)]
    other: serde_json::Map<String, serde_json::Value>,
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
    fn round_trips_through_disk() {
        let dir = std::env::temp_dir().join(format!("headroom-settings-{}", std::process::id()));
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
