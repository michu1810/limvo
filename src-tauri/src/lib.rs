//! Limvo: plan limits in the Windows taskbar.
//!
//! The Rust core owns everything that touches the system: credentials, the
//! network, windows and the tray. The UI in `src/` only renders reports and
//! asks for actions through the commands in [`commands`].

mod accounts;
mod activity;
mod brand;
mod card;
mod commands;
mod events;
mod flyout;
mod hotkey;
mod mark;
mod package;
mod poller;
mod providers;
mod settings;
mod snapshot;
mod system;
#[cfg(windows)]
mod taskbar;
mod tray;
mod updater;
mod widgets;

use std::sync::Mutex;

use tauri::{Emitter, Manager, WindowEvent};

use crate::activity::ActivityWatch;
use crate::card::CardState;
use crate::flyout::SharedFlyoutState;
use crate::poller::PollerHandle;
use crate::settings::{Settings, SettingsStore};
use crate::tray::TrayMenu;

/// Shared state, owned by Tauri and handed to commands.
#[derive(Debug)]
pub struct AppState {
    poller: PollerHandle,
    settings: Mutex<Settings>,
    store: SettingsStore,
    flyout: SharedFlyoutState,
    card: CardState,
    tray: TrayMenu,
    /// Replaced when accounts change, so it follows their folders.
    activity: Mutex<ActivityWatch>,
}

/// Puts a new list of extra accounts into effect while the app runs.
pub fn apply_accounts(app: &tauri::AppHandle, accounts: &[accounts::Account]) {
    let state = app.state::<AppState>();
    let providers = providers::all(accounts);
    let paths = providers
        .iter()
        .flat_map(|provider| provider.activity_paths())
        .collect();
    *state
        .activity
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = activity::watch(paths, &state.poller);
    state.poller.set_providers(providers);
}

pub fn run() {
    tauri::Builder::default()
        // Managed before any window exists, since the flyout asks for it on load.
        .manage(updater::PendingUpdate::default())
        // A second launch opens the running instance's flyout instead of a second tray icon.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            flyout::show(app, None);
        }))
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(hotkey::plugin())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .invoke_handler(tauri::generate_handler![
            commands::get_reports,
            commands::refresh,
            commands::get_settings,
            commands::update_settings,
            commands::get_launch_at_login,
            commands::set_launch_at_login,
            commands::system_locales,
            commands::host_info,
            commands::open_usage_page,
            commands::fit_flyout,
            commands::fit_strip,
            commands::toggle_flyout,
            commands::shortcut_label,
            commands::hide_flyout,
            commands::localize_tray,
            commands::update_widget,
            commands::add_account,
            commands::remove_account,
            commands::rename_account,
            commands::get_update,
            commands::install_update,
            commands::quit,
        ])
        .setup(|app| {
            let store = SettingsStore::new(&app.path().app_config_dir()?);
            let settings = store.load();
            let tray = tray::build(app)?;
            tray::use_brand(app.handle(), "claude");

            let providers = providers::all(&settings.accounts);
            let activity_paths = providers
                .iter()
                .flat_map(|provider| provider.activity_paths())
                .collect();
            let poller = poller::start(app.handle().clone(), providers, settings.refresh);
            let activity = activity::watch(activity_paths, &poller);

            #[cfg(windows)]
            taskbar::set_enabled(settings.show_strip);
            #[cfg(windows)]
            taskbar::set_all_taskbars(settings.strip_all_monitors);
            let (desktop_card, card_position) = (settings.desktop_card, settings.card_position);
            let shortcut = settings.shortcut;
            app.manage(AppState {
                poller,
                settings: Mutex::new(settings),
                store,
                flyout: SharedFlyoutState::default(),
                card: CardState::default(),
                tray,
                activity: Mutex::new(activity),
            });
            #[cfg(windows)]
            taskbar::start(app.handle().clone());
            card::set_visible(app.handle(), desktop_card, card_position);
            hotkey::apply(app.handle(), shortcut);
            updater::start(app.handle().clone());
            widgets::start(app.handle());
            #[cfg(windows)]
            package::start_at_login_by_default(app.path().app_config_dir()?);
            Ok(())
        })
        .on_window_event(|window, event| {
            let app = window.app_handle();
            match (window.label(), event) {
                // Like every Windows flyout, it closes when the user clicks elsewhere.
                (flyout::LABEL, WindowEvent::Focused(false)) => flyout::on_blur(app),
                // Tells the UI to play its entrance; hidden webviews throttle their own clocks.
                (flyout::LABEL, WindowEvent::Focused(true)) => {
                    let _ = app.emit_to(flyout::LABEL, events::FLYOUT_SHOWN, ());
                }
                (card::LABEL, WindowEvent::Moved(position)) => card::on_moved(app, *position),
                (card::LABEL, WindowEvent::Focused(false)) => card::on_blur(app),
                // Windows are created once and reused; closing one only hides it.
                (_, WindowEvent::CloseRequested { api, .. }) => {
                    api.prevent_close();
                    let _ = window.hide();
                }
                _ => {}
            }
        })
        .run(tauri::generate_context!())
        .unwrap_or_else(|error| log::error!("Limvo stopped: {error}"));
}
