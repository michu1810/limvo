//! In-app updates from GitHub Releases.
//!
//! A while after start and then twice a day, the core asks the release feed
//! (`latest.json` of the newest release) whether a newer version exists. The
//! feed and the installer it points to are checked against the public key in
//! `tauri.conf.json`, so only a build signed with the release key installs.
//! Nothing downloads until the user chooses to update in the flyout.

use std::sync::{Mutex, PoisonError};
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_updater::{Update, UpdaterExt};

use crate::{AppState, events};

/// The first check waits until start-up work and the first readings are done.
const FIRST_CHECK: Duration = Duration::from_mins(2);
const BETWEEN_CHECKS: Duration = Duration::from_hours(12);

/// The update found by the last check, waiting for the user.
#[derive(Default)]
pub struct PendingUpdate(Mutex<Option<Update>>);

// `Update` has no `Debug`; the version is all a log needs.
impl std::fmt::Debug for PendingUpdate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let pending = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        f.debug_tuple("PendingUpdate")
            .field(&pending.as_ref().map(|update| update.version.as_str()))
            .finish()
    }
}

/// What the interface learns about an update: never a URL or a signature.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AvailableUpdate {
    pub version: String,
    pub current_version: String,
}

impl From<&Update> for AvailableUpdate {
    fn from(update: &Update) -> Self {
        Self {
            version: update.version.clone(),
            current_version: update.current_version.clone(),
        }
    }
}

/// Starts the background checks. They skip a round while the setting is off.
///
/// [`PendingUpdate`] is managed on the builder, not here: the windows load
/// before `setup` ends and may ask for the update straight away.
///
/// The Store build never checks: the Store delivers its updates, and with no
/// check nothing is ever pending, so [`available`] and [`install`] stay inert.
pub fn start(app: AppHandle) {
    if crate::package::is_packaged() {
        log::info!("running from the Store package; updates come from the Store");
        return;
    }
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(FIRST_CHECK).await;
        loop {
            if checks_enabled(&app) {
                check(&app).await;
            }
            tokio::time::sleep(BETWEEN_CHECKS).await;
        }
    });
}

fn checks_enabled(app: &AppHandle) -> bool {
    app.state::<AppState>()
        .settings
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .check_updates
}

/// One check. Failures are only logged: being offline is not worth a message.
async fn check(app: &AppHandle) {
    let found = match app.updater() {
        Ok(updater) => updater.check().await,
        Err(error) => Err(error),
    };
    match found {
        Ok(Some(update)) => {
            let available = AvailableUpdate::from(&update);
            log::info!("Limvo {} is available", available.version);
            *app.state::<PendingUpdate>()
                .0
                .lock()
                .unwrap_or_else(PoisonError::into_inner) = Some(update);
            let _ = app.emit(events::UPDATE_AVAILABLE, &available);
        }
        Ok(None) => {}
        Err(error) => log::warn!("update check failed: {error}"),
    }
}

/// The update waiting for the user, if any.
pub fn available(app: &AppHandle) -> Option<AvailableUpdate> {
    app.state::<PendingUpdate>()
        .0
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .as_ref()
        .map(AvailableUpdate::from)
}

/// Downloads, verifies and runs the installer. On Windows the installer closes
/// Limvo and starts the new version itself, so this only returns on failure.
pub async fn install(app: &AppHandle) -> Result<(), String> {
    let update = app
        .state::<PendingUpdate>()
        .0
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clone()
        .ok_or("no update is waiting")?;
    update
        .download_and_install(|_, _| {}, || {})
        .await
        .map_err(|error| error.to_string())?;
    app.restart();
}
