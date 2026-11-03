//! A system-wide shortcut that opens the flyout from anywhere.
//!
//! Win+Shift+H is tried first. Windows reserves many Win combinations and
//! other apps may hold this one too, so Ctrl+Alt+H is the fallback. The
//! shortcut that actually registered is reported to the settings panel.

use std::sync::Mutex;

use tauri::plugin::TauriPlugin;
use tauri::{AppHandle, Wry};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

/// Shortcuts in order of preference, with the label the settings panel shows.
const CANDIDATES: [(Modifiers, Code, &str); 2] = [
    (
        Modifiers::SUPER.union(Modifiers::SHIFT),
        Code::KeyH,
        "Win+Shift+H",
    ),
    (
        Modifiers::CONTROL.union(Modifiers::ALT),
        Code::KeyH,
        "Ctrl+Alt+H",
    ),
];

/// The label of the registered shortcut, if any.
static ACTIVE: Mutex<Option<&'static str>> = Mutex::new(None);

pub fn plugin() -> TauriPlugin<Wry> {
    tauri_plugin_global_shortcut::Builder::new()
        .with_handler(|app, _shortcut, event| {
            if event.state() == ShortcutState::Pressed {
                crate::flyout::toggle_from_strip(app);
            }
        })
        .build()
}

/// Registers the first free shortcut, or removes it when `enabled` is false.
pub fn apply(app: &AppHandle, enabled: bool) {
    let shortcuts = app.global_shortcut();
    let _ = shortcuts.unregister_all();
    let mut active = ACTIVE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    *active = None;
    if !enabled {
        return;
    }
    for (modifiers, code, label) in CANDIDATES {
        match shortcuts.register(Shortcut::new(Some(modifiers), code)) {
            Ok(()) => {
                log::info!("{label} opens the flyout");
                *active = Some(label);
                return;
            }
            Err(error) => log::info!("{label} is not available: {error}"),
        }
    }
}

pub fn active_label() -> Option<&'static str> {
    *ACTIVE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
