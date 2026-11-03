//! Keeps the strip window sitting on the taskbar, next to the notification area.
//!
//! The strip stays a top-level window instead of being re-parented into
//! `Shell_TrayWnd`: a `WebView2` child of the taskbar loses its transparency,
//! while a top-level tool window composes cleanly over the taskbar material.
//! The cost is that Explorer can raise the taskbar above it, so a light
//! watcher re-asserts the position and z-order a few times a second, and
//! steps aside for full-screen apps and an auto-hidden taskbar.

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager};
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromWindow,
};
use windows::Win32::UI::HiDpi::GetDpiForWindow;
use windows::Win32::UI::WindowsAndMessaging::{
    FindWindowExW, FindWindowW, GWL_EXSTYLE, GetClassNameW, GetForegroundWindow, GetWindowLongPtrW,
    GetWindowRect, HWND_TOPMOST, SW_HIDE, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW,
    SetWindowLongPtrW, SetWindowPos, ShowWindow, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
};
use windows::core::w;

use crate::events;
use crate::system;

pub const LABEL: &str = "strip";
const TICK: Duration = Duration::from_millis(400);
/// Space between the strip and the notification area, in logical pixels.
const GAP: f64 = 6.0;

/// Width of the strip content in logical pixels, reported by the UI.
static CONTENT_WIDTH: AtomicU32 = AtomicU32::new(150);
/// The user can turn the strip off in settings and keep only the tray icon.
static ENABLED: AtomicBool = AtomicBool::new(true);

pub fn set_enabled(enabled: bool) {
    ENABLED.store(enabled, Ordering::Relaxed);
}

pub fn set_content_width(width: f64) {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    CONTENT_WIDTH.store(width.clamp(40.0, 400.0).ceil() as u32, Ordering::Relaxed);
}

pub fn start(app: AppHandle) {
    let spawned = std::thread::Builder::new()
        .name("taskbar-dock".into())
        .spawn(move || watch(&app));
    if let Err(error) = spawned {
        log::error!("could not start the taskbar watcher: {error}");
    }
}

/// Where the strip should be, in screen pixels, or `None` when it should be hidden.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Placement {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

fn watch(app: &AppHandle) {
    let mut styled = false;
    let mut current: Option<Placement> = None;
    let mut theme = system::taskbar_theme();

    loop {
        std::thread::sleep(TICK);
        let Some(hwnd) = app
            .get_webview_window(LABEL)
            .and_then(|window| window.hwnd().ok())
            .map(|raw| HWND(raw.0))
        else {
            continue;
        };
        if !styled {
            make_tool_window(hwnd);
            styled = true;
        }

        let target = if !ENABLED.load(Ordering::Relaxed) || full_screen_app_active() {
            None
        } else {
            placement()
        };
        match (target, current) {
            (Some(target), previous) if previous != Some(target) => {
                // SAFETY: plain window positioning on a window this process owns.
                let _ = unsafe {
                    SetWindowPos(
                        hwnd,
                        Some(HWND_TOPMOST),
                        target.x,
                        target.y,
                        target.width,
                        target.height,
                        SWP_NOACTIVATE | SWP_SHOWWINDOW,
                    )
                };
            }
            (Some(_), _) => raise_above_taskbar(hwnd),
            (None, Some(_)) => {
                // SAFETY: hiding a window this process owns.
                let _ = unsafe { ShowWindow(hwnd, SW_HIDE) };
            }
            (None, None) => {}
        }
        current = target;

        let next_theme = system::taskbar_theme();
        if next_theme != theme {
            theme = next_theme;
            let _ = app.emit(events::TASKBAR_THEME, theme);
        }
    }
}

/// Tool windows stay out of Alt+Tab, and no-activate windows never take focus from the user's app.
fn make_tool_window(hwnd: HWND) {
    // SAFETY: reading and extending the extended style of a window this process owns.
    unsafe {
        let style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        #[allow(clippy::cast_possible_wrap)]
        let extra = (WS_EX_TOOLWINDOW.0 | WS_EX_NOACTIVATE.0) as isize;
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, style | extra);
    }
}

/// Explorer brings the taskbar to the front of the topmost band when it is clicked.
fn raise_above_taskbar(hwnd: HWND) {
    // SAFETY: comparing window handles and re-ordering a window this process owns.
    unsafe {
        let Ok(taskbar) = FindWindowW(w!("Shell_TrayWnd"), None) else {
            return;
        };
        if GetForegroundWindow() == taskbar {
            let _ = SetWindowPos(
                hwnd,
                Some(HWND_TOPMOST),
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            );
        }
    }
}

fn placement() -> Option<Placement> {
    // SAFETY: read-only queries on Explorer's windows; every out parameter is a local.
    unsafe {
        let taskbar = FindWindowW(w!("Shell_TrayWnd"), None).ok()?;
        let mut bar = RECT::default();
        GetWindowRect(taskbar, &raw mut bar).ok()?;

        // Vertical taskbars (Windows 10) have no room for a horizontal strip.
        if bar.bottom - bar.top > bar.right - bar.left {
            return None;
        }
        // An auto-hidden taskbar slides below the monitor edge.
        let monitor = MonitorFromWindow(taskbar, MONITOR_DEFAULTTONEAREST);
        let mut info = MONITORINFO {
            cbSize: u32::try_from(std::mem::size_of::<MONITORINFO>()).ok()?,
            ..MONITORINFO::default()
        };
        if GetMonitorInfoW(monitor, &raw mut info).as_bool()
            && bar.bottom > info.rcMonitor.bottom + 2
        {
            return None;
        }

        let scale = f64::from(GetDpiForWindow(taskbar)) / 96.0;
        let content = f64::from(CONTENT_WIDTH.load(Ordering::Relaxed));
        #[allow(clippy::cast_possible_truncation)]
        let width = (content * scale).round() as i32;
        #[allow(clippy::cast_possible_truncation)]
        let gap = (GAP * scale).round() as i32;

        let tray_left = FindWindowExW(Some(taskbar), None, w!("TrayNotifyWnd"), None)
            .ok()
            .and_then(|tray| {
                let mut rect = RECT::default();
                GetWindowRect(tray, &raw mut rect).ok().map(|()| rect.left)
            })
            .unwrap_or(bar.right - width * 2);

        Some(Placement {
            x: tray_left - width - gap,
            y: bar.top,
            width,
            height: bar.bottom - bar.top,
        })
    }
}

/// A game, a video or a slideshow covers the taskbar's monitor edge to edge.
///
/// `SHQueryUserNotificationState` is not used: it reports "busy" for a
/// full-screen window on any monitor, and for live wallpapers, which would
/// hide the strip on a perfectly visible taskbar.
fn full_screen_app_active() -> bool {
    // SAFETY: read-only queries; every out parameter is a local.
    unsafe {
        let Ok(taskbar) = FindWindowW(w!("Shell_TrayWnd"), None) else {
            return false;
        };
        let foreground = GetForegroundWindow();
        if foreground.is_invalid() || is_shell_window(foreground) {
            return false;
        }
        let monitor = MonitorFromWindow(taskbar, MONITOR_DEFAULTTONEAREST);
        if MonitorFromWindow(foreground, MONITOR_DEFAULTTONEAREST) != monitor {
            return false;
        }
        let mut info = MONITORINFO {
            cbSize: u32::try_from(std::mem::size_of::<MONITORINFO>()).unwrap_or(40),
            ..MONITORINFO::default()
        };
        let mut window = RECT::default();
        GetMonitorInfoW(monitor, &raw mut info).as_bool()
            && GetWindowRect(foreground, &raw mut window).is_ok()
            && window.left <= info.rcMonitor.left
            && window.top <= info.rcMonitor.top
            && window.right >= info.rcMonitor.right
            && window.bottom >= info.rcMonitor.bottom
    }
}

/// The desktop and the taskbar span the monitor too, but they are not apps.
fn is_shell_window(hwnd: HWND) -> bool {
    let mut class = [0u16; 32];
    // SAFETY: the buffer length is passed alongside the buffer.
    let length = unsafe { GetClassNameW(hwnd, &mut class) };
    let name = String::from_utf16_lossy(&class[..usize::try_from(length).unwrap_or(0)]);
    matches!(
        name.as_str(),
        "Progman" | "WorkerW" | "Shell_TrayWnd" | "Shell_SecondaryTrayWnd"
    )
}
