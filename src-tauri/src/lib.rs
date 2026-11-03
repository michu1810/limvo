//! Headroom: plan limits in the Windows taskbar.
//!
//! The Rust core owns everything that touches the system: credentials, the
//! network, windows and the tray. The UI in `src/` only renders reports and
//! asks for actions through the commands in [`commands`].

mod commands;
mod events;
mod flyout;
mod poller;
mod providers;
mod settings;
mod snapshot;
mod system;
#[cfg(windows)]
mod taskbar;
mod tray;

use std::sync::Mutex;

use tauri::{Manager, WindowEvent};

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
    tray: TrayMenu,
}

pub fn run() {
    tauri::Builder::default()
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
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .invoke_handler(tauri::generate_handler![
            commands::get_report,
            commands::refresh,
            commands::get_settings,
            commands::set_locale,
            commands::system_locales,
            commands::host_info,
            commands::open_usage_page,
            commands::fit_flyout,
            commands::fit_strip,
            commands::toggle_flyout,
            commands::hide_flyout,
            commands::localize_tray,
            commands::quit,
        ])
        .setup(|app| {
            let store = SettingsStore::new(&app.path().app_config_dir()?);
            let settings = store.load();
            let tray = tray::build(app)?;
            let poller = poller::start(app.handle().clone(), providers::default_provider());
            app.manage(AppState {
                poller,
                settings: Mutex::new(settings),
                store,
                flyout: SharedFlyoutState::default(),
                tray,
            });
            #[cfg(windows)]
            taskbar::start(app.handle().clone());
            Ok(())
        })
        .on_window_event(|window, event| match event {
            // Like every Windows flyout, it closes when the user clicks elsewhere.
            WindowEvent::Focused(false) if window.label() == flyout::LABEL => {
                flyout::hide(window.app_handle());
            }
            // Windows are created once and reused; closing one only hides it.
            WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                let _ = window.hide();
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .unwrap_or_else(|error| log::error!("Headroom stopped: {error}"));
}
