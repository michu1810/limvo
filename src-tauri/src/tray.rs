//! The notification area icon and its menu.

use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

use tauri::image::Image;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{App, AppHandle, Manager, Wry};

use crate::AppState;
use crate::flyout::{self, Anchor};

pub const ID: &str = "main";
const REFRESH: &str = "refresh";
const QUIT: &str = "quit";
/// Level the icon was last drawn at, in whole percent; -1 means "no reading".
static DRAWN_LEVEL: AtomicI32 = AtomicI32::new(i32::MIN);
/// The provider's own mark is in use, so readings no longer redraw the icon.
static BRANDED: AtomicBool = AtomicBool::new(false);

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
            REFRESH => {
                if let Some(state) = app.try_state::<AppState>() {
                    state.poller.refresh_now();
                }
            }
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
    builder = builder.icon(level_icon(None));
    builder.build(app)?;

    Ok(TrayMenu { refresh, quit })
}

/// Redraws the icon so its crossbar shows percent. Cheap to call often: it only redraws on change.
pub fn show_level(app: &AppHandle, percent: Option<f64>) {
    if BRANDED.load(Ordering::Relaxed) {
        return;
    }
    #[allow(clippy::cast_possible_truncation)]
    let level = percent.map_or(-1, |value| value.round() as i32);
    if DRAWN_LEVEL.swap(level, Ordering::Relaxed) == level {
        return;
    }
    if let Some(tray) = app.tray_by_id(ID) {
        let _ = tray.set_icon(Some(level_icon(percent)));
    }
}

/// Uses the provider's own mark for the icon when the user bundled one.
pub fn use_brand(app: &AppHandle, provider: &str) {
    let Some(icon) = crate::brand::svg(app, provider).and_then(|svg| crate::brand::icon(&svg, 32))
    else {
        return;
    };
    if let Some(tray) = app.tray_by_id(ID)
        && tray.set_icon(Some(icon)).is_ok()
    {
        BRANDED.store(true, Ordering::Relaxed);
    }
}

fn level_icon(percent: Option<f64>) -> Image<'static> {
    Image::new_owned(
        crate::mark::rgba(percent),
        crate::mark::SIZE,
        crate::mark::SIZE,
    )
}

/// Applies texts translated by the UI.
pub fn localize(app: &AppHandle, menu: &TrayMenu, tooltip: &str, refresh: &str, quit: &str) {
    let _ = menu.refresh.set_text(refresh);
    let _ = menu.quit.set_text(quit);
    if let Some(tray) = app.tray_by_id(ID) {
        let _ = tray.set_tooltip(Some(tooltip));
    }
}
