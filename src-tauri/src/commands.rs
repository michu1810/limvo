//! Commands the UI may invoke. Names match `src/bridge/tauri.ts`.

// Tauri hands command arguments over by value.
#![allow(clippy::needless_pass_by_value)]

use std::sync::PoisonError;

use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_autostart::ManagerExt as _;
use tauri_plugin_opener::OpenerExt;

use crate::accounts::{self, Account, AccountError, Service};
use crate::flyout;
use crate::settings::{Settings, SettingsPatch};
use crate::snapshot::ProviderReport;
use crate::system::{self, HostInfo};
use crate::updater::{self, AvailableUpdate};
use crate::{AppState, card, events, tray};

/// One report per provider, in the order the interface lists them.
#[tauri::command]
pub fn get_reports(state: State<'_, AppState>) -> Vec<ProviderReport> {
    state.poller.reports()
}

#[tauri::command]
pub fn refresh(state: State<'_, AppState>) {
    state.poller.refresh_now();
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Settings {
    state
        .settings
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clone()
}

#[tauri::command]
pub fn update_settings(app: AppHandle, patch: SettingsPatch) -> Result<Settings, String> {
    apply_patch(&app, patch)
}

/// Applies a partial update, saves it, tells every window and puts it into effect.
pub fn apply_patch(app: &AppHandle, mut patch: SettingsPatch) -> Result<Settings, String> {
    if let Some(Some(code)) = &patch.locale
        && !is_locale_tag(code)
    {
        patch.locale = None;
    }
    let state = app.state::<AppState>();
    let (before, after) = {
        let mut settings = state
            .settings
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let before = settings.clone();
        settings.apply(patch);
        (before, settings.clone())
    };
    state
        .store
        .save(&after)
        .map_err(|error| error.to_string())?;

    if before.refresh != after.refresh {
        state.poller.set_mode(after.refresh);
    }
    #[cfg(windows)]
    crate::taskbar::set_enabled(after.show_strip);
    #[cfg(windows)]
    crate::taskbar::set_all_taskbars(after.strip_all_monitors);
    if before.shortcut != after.shortcut {
        crate::hotkey::apply(app, after.shortcut);
    }
    if before.desktop_card != after.desktop_card {
        card::set_visible(app, after.desktop_card, after.card_position);
    }
    app.emit(events::SETTINGS_UPDATED, &after)
        .map_err(|error| error.to_string())?;
    Ok(after)
}

#[tauri::command]
pub fn get_launch_at_login(app: AppHandle) -> bool {
    app.autolaunch().is_enabled().unwrap_or(false)
}

#[tauri::command]
pub fn set_launch_at_login(app: AppHandle, enabled: bool) -> Result<bool, String> {
    let autolaunch = app.autolaunch();
    let result = if enabled {
        autolaunch.enable()
    } else {
        autolaunch.disable()
    };
    result.map_err(|error| error.to_string())?;
    Ok(autolaunch.is_enabled().unwrap_or(enabled))
}

#[tauri::command]
pub fn system_locales() -> Vec<String> {
    system::preferred_languages()
}

#[tauri::command]
pub fn host_info() -> HostInfo {
    system::host_info()
}

/// Opens a provider's own usage page. Only the URL from that provider's
/// reading is accepted, so the UI cannot be used to open arbitrary links.
#[tauri::command]
pub fn open_usage_page(
    app: AppHandle,
    state: State<'_, AppState>,
    provider_id: String,
) -> Result<(), String> {
    let Some(url) = state
        .poller
        .report(&provider_id)
        .and_then(|report| report.snapshot)
        .and_then(|snapshot| snapshot.usage_page_url)
    else {
        return Ok(());
    };
    if !url.starts_with("https://") {
        return Err("refusing a non-HTTPS usage page".into());
    }
    flyout::hide(&app);
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn fit_flyout(app: AppHandle, height: f64) {
    flyout::fit(&app, height);
}

#[tauri::command]
pub fn fit_strip(width: f64) {
    #[cfg(windows)]
    crate::taskbar::set_content_width(width);
    #[cfg(not(windows))]
    let _ = width;
}

/// Opens or closes the flyout above the taskbar strip that was clicked.
#[tauri::command]
pub fn toggle_flyout(app: AppHandle, window: tauri::WebviewWindow) {
    flyout::toggle_from_strip(&app, window.label());
}

/// The system-wide shortcut that is active, for the settings panel.
#[tauri::command]
pub fn shortcut_label() -> Option<&'static str> {
    crate::hotkey::active_label()
}

#[tauri::command]
pub fn hide_flyout(app: AppHandle) {
    flyout::hide(&app);
}

#[tauri::command]
pub fn localize_tray(
    app: AppHandle,
    state: State<'_, AppState>,
    tooltip: String,
    refresh: String,
    quit: String,
) {
    tray::localize(&app, &state.tray, &tooltip, &refresh, &quit);
}

/// Adds an account of `service` signed in from a folder the user picks in
/// the system dialog. The interface never sees or sends the path: it gets
/// the saved settings, or an error code (`cancelled`, `too_many`,
/// `default_folder`, `duplicate`, `no_sign_in`) to explain.
#[tauri::command]
pub async fn add_account(
    app: AppHandle,
    service: Service,
    label: String,
    title: String,
) -> Result<Settings, String> {
    let title: String = title.chars().take(80).collect();
    let Some(folder) = pick_folder(&app, title).await else {
        return Err("cancelled".into());
    };
    let folder = accounts::canonical(&folder).ok_or_else(|| AccountError::NoSignIn.to_string())?;
    let default_folder = service
        .default_folder()
        .and_then(|dir| accounts::canonical(&dir));
    let has_sign_in = folder.join(service.sign_in_file()).is_file();
    let state = app.state::<AppState>();
    let existing = state
        .settings
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .accounts
        .clone();
    accounts::check_new(
        &existing,
        service,
        &folder,
        default_folder.as_deref(),
        has_sign_in,
    )
    .map_err(|error| error.to_string())?;

    let id = accounts::next_id(&existing, service);
    let account = Account {
        label: accounts::clean_label(&label, &id),
        id,
        service,
        folder,
    };
    log::info!("adding {} as {}", account.service.name(), account.id);
    change_accounts(&app, |accounts| accounts.push(account))
}

/// Forgets an extra account. Its folder and sign-in stay as they are.
#[tauri::command]
pub fn remove_account(app: AppHandle, id: String) -> Result<Settings, String> {
    change_accounts(&app, |accounts| accounts.retain(|account| account.id != id))
}

#[tauri::command]
pub fn rename_account(app: AppHandle, id: String, label: String) -> Result<Settings, String> {
    change_accounts(&app, |accounts| {
        if let Some(account) = accounts.iter_mut().find(|account| account.id == id) {
            account.label = accounts::clean_label(&label, &id);
        }
    })
}

/// Saves a change to the accounts and puts it into effect: the poller reads
/// the new list, the activity watch follows its folders, every window hears.
fn change_accounts(
    app: &AppHandle,
    change: impl FnOnce(&mut Vec<Account>),
) -> Result<Settings, String> {
    let state = app.state::<AppState>();
    let after = {
        let mut settings = state
            .settings
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        change(&mut settings.accounts);
        settings.clone()
    };
    state
        .store
        .save(&after)
        .map_err(|error| error.to_string())?;
    crate::apply_accounts(app, &after.accounts);
    app.emit(events::SETTINGS_UPDATED, &after)
        .map_err(|error| error.to_string())?;
    Ok(after)
}

async fn pick_folder(app: &AppHandle, title: String) -> Option<std::path::PathBuf> {
    use tauri_plugin_dialog::DialogExt;

    let (sender, receiver) = tokio::sync::oneshot::channel();
    // The dialog belongs to the flyout, which stays open behind it, so the
    // new account or the reason it was refused shows where the user asked.
    flyout::hold_open(true);
    let mut dialog = app.dialog().file().set_title(title);
    if let Some(window) = app.get_webview_window(flyout::LABEL) {
        dialog = dialog.set_parent(&window);
    }
    dialog.pick_folder(move |folder| {
        let _ = sender.send(folder);
    });
    let picked = receiver.await.ok().flatten();
    flyout::hold_open(false);
    if let Some(window) = app.get_webview_window(flyout::LABEL) {
        let _ = window.set_focus();
    }
    picked?.into_path().ok()
}

/// The newer version found by the last check, if any.
#[tauri::command]
pub fn get_update(app: AppHandle) -> Option<AvailableUpdate> {
    updater::available(&app)
}

/// Installs the update that was found. Only the one from the verified feed
/// can be installed: the interface names no version, URL or file.
#[tauri::command]
pub async fn install_update(app: AppHandle) -> Result<(), String> {
    updater::install(&app).await
}

#[tauri::command]
pub fn quit(app: AppHandle) {
    app.exit(0);
}

/// A loose BCP 47 check: letters, digits and hyphens, as `Intl` accepts.
fn is_locale_tag(code: &str) -> bool {
    !code.is_empty()
        && code.len() <= 35
        && code.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

#[cfg(test)]
mod tests {
    use super::is_locale_tag;

    #[test]
    fn accepts_locale_tags_and_nothing_else() {
        assert!(is_locale_tag("pt-BR"));
        assert!(is_locale_tag("he"));
        assert!(!is_locale_tag(""));
        assert!(!is_locale_tag("../../etc"));
    }
}
