//! The flyout window: where it opens, when it hides, how tall it is.

use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

use tauri::{AppHandle, LogicalSize, Manager, PhysicalPosition, WebviewWindow};

use crate::AppState;

pub const LABEL: &str = "flyout";
const WIDTH: f64 = 380.0;
const MIN_HEIGHT: f64 = 120.0;
const MAX_HEIGHT: f64 = 760.0;
/// Gap between the flyout and the taskbar, in logical pixels, as Windows flyouts use.
const MARGIN: f64 = 12.0;
/// Clicking the tray icon while the flyout is open first blurs it, which hides
/// it, and then delivers the click. A toggle this soon after a blur is that
/// same click, meant to close the flyout, so it must not reopen it.
const REOPEN_GUARD: Duration = Duration::from_millis(300);

/// A point on the taskbar the flyout grows from: the horizontal centre of the
/// element that opened it and the edge of that element facing the desktop.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Anchor {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Default)]
pub struct FlyoutState {
    anchor: Option<Anchor>,
    hidden_at: Option<Instant>,
}

pub type SharedFlyoutState = Mutex<FlyoutState>;

/// Opens or closes the flyout above the taskbar strip, or above the notification area when the strip is hidden.
/// The strip on the main taskbar.
pub const MAIN_STRIP: &str = "strip";

/// `strip` is the label of the window that asked: each monitor has its own
/// strip, and the flyout opens above the one that was clicked. Other windows
/// (the desktop card) anchor to the main strip.
pub fn toggle_from_strip(app: &AppHandle, strip: &str) {
    let label = if strip.starts_with(MAIN_STRIP) {
        strip
    } else {
        MAIN_STRIP
    };
    let anchor = app
        .get_webview_window(label)
        .filter(|strip| strip.is_visible().unwrap_or(false))
        .and_then(|strip| Some((strip.outer_position().ok()?, strip.outer_size().ok()?)))
        .map(|(position, size)| Anchor {
            x: f64::from(position.x) + f64::from(size.width) / 2.0,
            y: f64::from(position.y),
        });
    match anchor {
        Some(anchor) => toggle(app, anchor),
        None if is_visible(app) => hide(app),
        None => show(app, None),
    }
}

fn is_visible(app: &AppHandle) -> bool {
    app.get_webview_window(LABEL)
        .is_some_and(|window| window.is_visible().unwrap_or(false))
}

pub fn toggle(app: &AppHandle, anchor: Anchor) {
    let Some(window) = app.get_webview_window(LABEL) else {
        return;
    };
    if window.is_visible().unwrap_or(false) {
        hide(app);
        return;
    }
    let recently_hidden = with_state(app, |state| {
        state
            .hidden_at
            .is_some_and(|at| at.elapsed() < REOPEN_GUARD)
    });
    if !recently_hidden {
        show(app, Some(anchor));
    }
}

/// Opens the flyout at `anchor`, or above the notification area when there is none.
pub fn show(app: &AppHandle, anchor: Option<Anchor>) {
    let Some(window) = app.get_webview_window(LABEL) else {
        return;
    };
    let anchor = anchor.or_else(|| default_anchor(&window));
    with_state(app, |state| state.anchor = anchor);
    if let Some(anchor) = anchor {
        place(&window, anchor);
    }
    let _ = window.show();
    let _ = window.set_focus();
    // Looking at the flyout asks for a fresh reading; the poller skips it if the last one is recent.
    if let Some(state) = app.try_state::<AppState>() {
        state.poller.refresh_now();
    }
}

pub fn hide(app: &AppHandle) {
    let Some(window) = app.get_webview_window(LABEL) else {
        return;
    };
    let _ = window.hide();
    with_state(app, |state| state.hidden_at = Some(Instant::now()));
}

/// Resizes the window to the content height reported by the UI, keeping it anchored.
pub fn fit(app: &AppHandle, height: f64) {
    let Some(window) = app.get_webview_window(LABEL) else {
        return;
    };
    let height = height.clamp(MIN_HEIGHT, MAX_HEIGHT);
    let _ = window.set_size(LogicalSize::new(WIDTH, height));
    if let Some(anchor) = with_state(app, |state| state.anchor) {
        place(&window, anchor);
    }
}

fn place(window: &WebviewWindow, anchor: Anchor) {
    let Ok(Some(monitor)) = window
        .monitor_from_point(anchor.x, anchor.y)
        .map(|found| found.or_else(|| window.primary_monitor().ok().flatten()))
    else {
        return;
    };
    let Ok(size) = window.outer_size() else {
        return;
    };
    let scale = monitor.scale_factor();
    let margin = MARGIN * scale;
    let work = monitor.work_area();

    let left = f64::from(work.position.x);
    let top = f64::from(work.position.y);
    let right = left + f64::from(work.size.width);
    let bottom = top + f64::from(work.size.height);
    let width = f64::from(size.width);
    let height = f64::from(size.height);

    let x =
        (anchor.x - width / 2.0).clamp(left + margin, (right - width - margin).max(left + margin));
    // The taskbar sits outside the work area; open away from whichever edge it is on.
    let y = if anchor.y <= top {
        top + margin
    } else {
        bottom - height - margin
    };

    #[allow(clippy::cast_possible_truncation)]
    let _ = window.set_position(PhysicalPosition::new(x.round() as i32, y.round() as i32));
}

/// Bottom right of the primary work area, where the notification area usually is.
fn default_anchor(window: &WebviewWindow) -> Option<Anchor> {
    let monitor = window.primary_monitor().ok().flatten()?;
    let work = monitor.work_area();
    Some(Anchor {
        x: f64::from(work.position.x) + f64::from(work.size.width)
            - WIDTH * monitor.scale_factor() / 2.0,
        y: f64::from(work.position.y) + f64::from(work.size.height),
    })
}

/// Runs  on the flyout state. Before setup has registered it (early window
/// events), there is nothing to remember yet and the default is returned.
fn with_state<T: Default>(app: &AppHandle, f: impl FnOnce(&mut FlyoutState) -> T) -> T {
    let Some(state) = app.try_state::<AppState>() else {
        return T::default();
    };
    let mut guard = state.flyout.lock().unwrap_or_else(PoisonError::into_inner);
    f(&mut guard)
}
