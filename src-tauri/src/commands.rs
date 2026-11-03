//! Commands the UI may invoke. Names match `src/bridge/tauri.ts`.

// Tauri hands command arguments over by value.
#![allow(clippy::needless_pass_by_value)]

use std::sync::PoisonError;

use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_autostart::ManagerExt as _;
use tauri_plugin_opener::OpenerExt;

use crate::flyout;
use crate::settings::{Settings, SettingsPatch};
use crate::snapshot::ProviderReport;
use crate::system::{self, HostInfo};
use crate::{AppState, card, events, tray};

#[tauri::command]
pub fn get_report(state: State<'_, AppState>) -> ProviderReport {
    state.poller.report()
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

/// Opens the provider's own usage page. Only the URL from the provider is
/// accepted, so the UI cannot be used to open arbitrary links.
#[tauri::command]
pub fn open_usage_page(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let Some(url) = state
        .poller
        .report()
        .snapshot
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

/// Opens or closes the flyout above the taskbar strip.
#[tauri::command]
pub fn toggle_flyout(app: AppHandle) {
    flyout::toggle_from_strip(&app);
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
