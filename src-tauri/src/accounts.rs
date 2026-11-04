//! Extra accounts of a service, each signed in from its own folder.
//!
//! Claude Code and Codex keep their sign-in in a folder the user can choose
//! (`CLAUDE_CONFIG_DIR`, `CODEX_HOME`), so a second account is a second
//! folder. The default account keeps its id (`claude`, `codex`); an extra one
//! gets `claude~2`, `codex~3` and so on, which never contains the `:` that
//! separates a pin's provider from its window.
//!
//! Folders are only ever chosen in the system's folder dialog, opened by the
//! core. The interface names a service and a label, never a path.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// At most this many extra accounts, across all services.
pub const MAX_ACCOUNTS: usize = 4;
/// Labels are short enough for a tab and the taskbar tooltip.
pub const MAX_LABEL: usize = 24;

/// A service that signs in from a folder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Service {
    Claude,
    Codex,
}

impl Service {
    pub fn id(self) -> &'static str {
        match self {
            Self::Claude => crate::providers::claude::PROVIDER_ID,
            Self::Codex => crate::providers::codex::PROVIDER_ID,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Claude => "Claude",
            Self::Codex => "Codex",
        }
    }

    /// The file that holds the sign-in inside the folder.
    pub fn sign_in_file(self) -> &'static str {
        match self {
            Self::Claude => ".credentials.json",
            Self::Codex => "auth.json",
        }
    }

    /// The folder the service's own tools use when nothing is set.
    pub fn default_folder(self) -> Option<PathBuf> {
        match self {
            Self::Claude => crate::providers::claude::default_folder(),
            Self::Codex => crate::providers::codex::default_folder(),
        }
    }
}

/// One extra account, as stored in the settings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    /// `<service>~<n>`, stable for pins, tabs and history.
    pub id: String,
    pub service: Service,
    pub folder: PathBuf,
    /// The user's name for it, such as "Work".
    pub label: String,
}

impl Account {
    /// The name the interface shows: the service and the label.
    pub fn display_name(&self) -> String {
        format!("{} · {}", self.service.name(), self.label)
    }
}

/// Why a folder cannot become an account. The interface translates the code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum AccountError {
    #[error("too_many")]
    TooMany,
    #[error("default_folder")]
    DefaultFolder,
    #[error("duplicate")]
    Duplicate,
    #[error("no_sign_in")]
    NoSignIn,
}

/// Checks a chosen folder against the accounts there already are.
///
/// `folder` and `default_folder` are compared as given, so callers pass
/// canonical paths; `has_sign_in` says whether the sign-in file exists.
pub fn check_new(
    existing: &[Account],
    service: Service,
    folder: &Path,
    default_folder: Option<&Path>,
    has_sign_in: bool,
) -> Result<(), AccountError> {
    if existing.len() >= MAX_ACCOUNTS {
        return Err(AccountError::TooMany);
    }
    if default_folder.is_some_and(|default| same_folder(default, folder)) {
        return Err(AccountError::DefaultFolder);
    }
    if existing
        .iter()
        .any(|account| account.service == service && same_folder(&account.folder, folder))
    {
        return Err(AccountError::Duplicate);
    }
    if !has_sign_in {
        return Err(AccountError::NoSignIn);
    }
    Ok(())
}

/// Windows paths ignore case.
fn same_folder(a: &Path, b: &Path) -> bool {
    a.as_os_str().eq_ignore_ascii_case(b.as_os_str())
}

/// The first free `<service>~<n>`, starting at 2: the default account is 1.
pub fn next_id(existing: &[Account], service: Service) -> String {
    // With at most MAX_ACCOUNTS taken, one of the first MAX_ACCOUNTS + 1 is free.
    (2..=MAX_ACCOUNTS + 2)
        .map(|n| format!("{}~{n}", service.id()))
        .find(|id| existing.iter().all(|account| &account.id != id))
        .unwrap_or_default()
}

/// A label the interface can show: trimmed, single-line and short.
/// An empty one falls back to the account's number.
pub fn clean_label(label: &str, id: &str) -> String {
    let single_line: String = label
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect();
    let trimmed: String = single_line.trim().chars().take(MAX_LABEL).collect();
    if trimmed.trim().is_empty() {
        id.rsplit('~').next().unwrap_or(id).to_owned()
    } else {
        trimmed.trim_end().to_owned()
    }
}

/// A canonical folder without the `\\?\` prefix Windows adds, so it reads
/// like the path the user picked and compares equal to other spellings.
pub fn canonical(folder: &Path) -> Option<PathBuf> {
    let canonical = std::fs::canonicalize(folder).ok()?;
    let text = canonical.to_string_lossy();
    Some(match text.strip_prefix(r"\\?\") {
        Some(rest) if !rest.starts_with("UNC") => PathBuf::from(rest),
        _ => canonical,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn account(id: &str, service: Service, folder: &str) -> Account {
        Account {
            id: id.into(),
            service,
            folder: folder.into(),
            label: "Work".into(),
        }
    }

    #[test]
    fn numbers_extra_accounts_from_two_and_reuses_free_numbers() {
        assert_eq!(next_id(&[], Service::Claude), "claude~2");
        let existing = [
            account("claude~2", Service::Claude, "a"),
            account("claude~4", Service::Claude, "b"),
            account("codex~2", Service::Codex, "c"),
        ];
        assert_eq!(next_id(&existing, Service::Claude), "claude~3");
        assert_eq!(next_id(&existing, Service::Codex), "codex~3");
    }

    #[test]
    fn ids_never_contain_the_pin_separator() {
        assert!(!next_id(&[], Service::Codex).contains(':'));
    }

    #[test]
    fn rejects_the_default_folder_in_any_case() {
        let result = check_new(
            &[],
            Service::Claude,
            Path::new(r"C:\Users\me\.CLAUDE"),
            Some(Path::new(r"C:\Users\me\.claude")),
            true,
        );
        assert_eq!(result, Err(AccountError::DefaultFolder));
    }

    #[test]
    fn rejects_a_folder_already_added_for_the_same_service() {
        let existing = [account("claude~2", Service::Claude, r"D:\work")];
        let check = |service| check_new(&existing, service, Path::new(r"d:\WORK"), None, true);
        assert_eq!(check(Service::Claude), Err(AccountError::Duplicate));
        // The same folder may hold another service's sign-in.
        assert_eq!(check(Service::Codex), Ok(()));
    }

    #[test]
    fn needs_a_sign_in_and_room() {
        let none = check_new(&[], Service::Codex, Path::new("x"), None, false);
        assert_eq!(none, Err(AccountError::NoSignIn));
        let full: Vec<_> = (0..MAX_ACCOUNTS)
            .map(|n| {
                account(
                    &format!("claude~{}", n + 2),
                    Service::Claude,
                    &n.to_string(),
                )
            })
            .collect();
        let crowded = check_new(&full, Service::Claude, Path::new("new"), None, true);
        assert_eq!(crowded, Err(AccountError::TooMany));
    }

    #[test]
    fn cleans_labels_for_display() {
        assert_eq!(clean_label("  Praca\n2 ", "claude~2"), "Praca 2");
        assert_eq!(clean_label("   ", "claude~3"), "3");
        let long = "x".repeat(40);
        assert_eq!(clean_label(&long, "codex~2").chars().count(), MAX_LABEL);
    }

    #[test]
    fn shows_the_service_and_the_label() {
        let work = account("claude~2", Service::Claude, "a");
        assert_eq!(work.display_name(), "Claude · Work");
    }

    #[test]
    fn canonical_folders_drop_the_verbatim_prefix_and_ignore_spelling() {
        let dir = std::env::temp_dir().join(format!("limvo-accounts-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("Work")).unwrap();
        let picked = canonical(&dir.join("Work")).unwrap();
        assert!(!picked.to_string_lossy().starts_with(r"\\?\"));
        let respelled = canonical(&dir.join("work").join("..").join("WORK")).unwrap();
        assert!(same_folder(&picked, &respelled));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn stored_accounts_round_trip() {
        let work = account("codex~2", Service::Codex, r"D:\codex-work");
        let json = serde_json::to_value(&work).unwrap();
        assert_eq!(json["service"], "codex");
        assert_eq!(serde_json::from_value::<Account>(json).unwrap(), work);
    }
}
