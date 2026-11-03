//! The notification area icon and its menu.

use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{App, AppHandle, Manager, Wry};

use crate::AppState;
use crate::flyout::{self, Anchor};

pub const ID: &str = "main";
const REFRESH: &str = "refresh";
const QUIT: &str = "quit";

/// Menu items kept so the UI can translate them.
pub struct TrayMenu {
    refresh: MenuItem<Wry>,
    quit: MenuItem<Wry>,
}

impl std::fmt::Debug for TrayMenu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TrayMenu").finish_non_exhaustive()
    }
}

pub fn build(app: &App) -> tauri::Result<TrayMenu> {
    let refresh = MenuItem::with_id(app, REFRESH, "Refresh", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, QUIT, "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&refresh, &quit])?;

    let mut builder = TrayIconBuilder::with_id(ID)
        .tooltip("Headroom")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            REFRESH => app.state::<AppState>().poller.refresh_now(),
            QUIT => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                rect,
                ..
            } = event
            {
                // The notification area reports physical pixels on Windows.
                let position = rect.position.to_physical::<f64>(1.0);
                let size = rect.size.to_physical::<f64>(1.0);
                let anchor = Anchor {
                    x: position.x + size.width / 2.0,
                    y: position.y,
                };
                flyout::toggle(tray.app_handle(), anchor);
            }
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;

    Ok(TrayMenu { refresh, quit })
}

/// Applies texts translated by the UI.
pub fn localize(app: &AppHandle, menu: &TrayMenu, tooltip: &str, refresh: &str, quit: &str) {
    let _ = menu.refresh.set_text(refresh);
    let _ = menu.quit.set_text(quit);
    if let Some(tray) = app.tray_by_id(ID) {
        let _ = tray.set_tooltip(Some(tooltip));
    }
}
