//! The card pinned to the desktop.
//!
//! It behaves like a desktop gadget rather than an app window: no taskbar
//! button, no focus stealing, and it sinks back below other windows as soon
//! as the user is done with it. Its position is remembered across runs.

use std::sync::{Mutex, PoisonError};

use tauri::{AppHandle, Manager, PhysicalPosition, WebviewWindow};

use crate::AppState;
use crate::settings::{CardPosition, SettingsPatch};

pub const LABEL: &str = "card";
/// Distance from the screen corner when the card has never been moved, in logical pixels.
const DEFAULT_INSET: f64 = 24.0;

/// Where the card was last dragged to, written to settings when the drag ends.
#[derive(Debug, Default)]
pub struct CardState {
    pending_position: Mutex<Option<CardPosition>>,
}

pub fn set_visible(app: &AppHandle, visible: bool, position: Option<CardPosition>) {
    let Some(window) = app.get_webview_window(LABEL) else {
        return;
    };
    if !visible {
        let _ = window.hide();
        return;
    }
    match position.filter(|position| is_on_screen(&window, *position)) {
        Some(position) => {
            let _ = window.set_position(PhysicalPosition::new(position.x, position.y));
        }
        None => place_in_default_corner(&window),
    }
    show_without_focus(&window);
    send_to_back(&window);
}

/// Appearing on the desktop must not pull focus away from what the user is doing.
#[cfg(windows)]
fn show_without_focus(window: &WebviewWindow) {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{SW_SHOWNOACTIVATE, ShowWindow};

    match window.hwnd() {
        // SAFETY: showing a window this process owns.
        Ok(raw) => {
            let _ = unsafe { ShowWindow(HWND(raw.0), SW_SHOWNOACTIVATE) };
        }
        Err(_) => {
            let _ = window.show();
        }
    }
}

#[cfg(not(windows))]
fn show_without_focus(window: &WebviewWindow) {
    let _ = window.show();
}

pub fn on_moved(app: &AppHandle, position: PhysicalPosition<i32>) {
    // Windows from the config exist before setup has registered the state.
    let Some(state) = app.try_state::<AppState>() else {
        return;
    };
    *state
        .card
        .pending_position
        .lock()
        .unwrap_or_else(PoisonError::into_inner) = Some(CardPosition {
        x: position.x,
        y: position.y,
    });
}

/// The user let go of the card: remember where it is and return it to the desktop layer.
pub fn on_blur(app: &AppHandle) {
    let Some(window) = app.get_webview_window(LABEL) else {
        return;
    };
    send_to_back(&window);
    let Some(state) = app.try_state::<AppState>() else {
        return;
    };
    let moved = state
        .card
        .pending_position
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .take();
    if let Some(position) = moved {
        let patch = SettingsPatch {
            card_position: Some(Some(position)),
            ..SettingsPatch::default()
        };
        if let Err(error) = crate::commands::apply_patch(app, patch) {
            log::warn!("could not remember the card position: {error}");
        }
    }
}

fn place_in_default_corner(window: &WebviewWindow) {
    let Ok(Some(monitor)) = window.primary_monitor() else {
        return;
    };
    let Ok(size) = window.outer_size() else {
        return;
    };
    let work = monitor.work_area();
    let inset = DEFAULT_INSET * monitor.scale_factor();
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    let x = work.position.x + work.size.width as i32 - size.width as i32 - inset.round() as i32;
    #[allow(clippy::cast_possible_truncation)]
    let y = work.position.y + inset.round() as i32;
    let _ = window.set_position(PhysicalPosition::new(x, y));
}

/// A saved position can point at a monitor that is no longer connected.
fn is_on_screen(window: &WebviewWindow, position: CardPosition) -> bool {
    window
        .monitor_from_point(f64::from(position.x) + 20.0, f64::from(position.y) + 20.0)
        .ok()
        .flatten()
        .is_some()
}

#[cfg(windows)]
fn send_to_back(window: &WebviewWindow) {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{
        HWND_BOTTOM, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SetWindowPos,
    };

    let Ok(raw) = window.hwnd() else {
        return;
    };
    // SAFETY: re-ordering a window this process owns.
    let _ = unsafe {
        SetWindowPos(
            HWND(raw.0),
            Some(HWND_BOTTOM),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        )
    };
}

#[cfg(not(windows))]
fn send_to_back(_window: &WebviewWindow) {}
