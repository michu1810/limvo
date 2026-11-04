//! Keeps a strip window on every taskbar, next to the notification area.
//!
//! The strips stay top-level windows instead of being re-parented into
//! `Shell_TrayWnd`: a `WebView2` child of the taskbar loses its transparency,
//! while a top-level tool window composes cleanly over the taskbar material.
//! The cost is that Explorer can raise a taskbar above them, so a light
//! watcher re-asserts position and z-order a few times a second, and steps
//! aside for full-screen apps and an auto-hidden taskbar.
//!
//! The main taskbar gets the `strip` window declared in `tauri.conf.json`.
//! Each taskbar on another monitor gets a copy of it (`strip-1`, `strip-2`,
//! ...), created the first time that taskbar appears.

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager, WebviewWindowBuilder};
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, HMONITOR, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromWindow,
};
use windows::Win32::UI::HiDpi::GetDpiForWindow;
use windows::Win32::UI::WindowsAndMessaging::{
    FindWindowExW, FindWindowW, GWL_EXSTYLE, GetClassNameW, GetForegroundWindow, GetWindowLongPtrW,
    GetWindowRect, HWND_TOPMOST, IsIconic, IsWindowVisible, SW_HIDE, SW_SHOWNOACTIVATE,
    SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW, SetWindowLongPtrW, SetWindowPos,
    ShowWindow, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
};
use windows::core::w;

use crate::events;
use crate::system;

pub const LABEL: &str = "strip";
const TICK: Duration = Duration::from_millis(400);
/// Space between the strip and the notification area, in logical pixels.
const GAP: f64 = 6.0;
/// What a secondary taskbar keeps at its right end for the clock and the
/// _Show desktop_ sliver, in logical pixels. Explorer draws them in XAML with
/// no window to measure, so this follows the Windows 11 layout.
const SECONDARY_TRAY: f64 = 120.0;

/// Width of the strip content in logical pixels, reported by the UI.
static CONTENT_WIDTH: AtomicU32 = AtomicU32::new(150);
/// The user can turn the strip off in settings and keep only the tray icon.
static ENABLED: AtomicBool = AtomicBool::new(true);
/// Whether the taskbars of other monitors get a strip too.
static ALL_TASKBARS: AtomicBool = AtomicBool::new(true);

pub fn set_enabled(enabled: bool) {
    ENABLED.store(enabled, Ordering::Relaxed);
}

pub fn set_all_taskbars(enabled: bool) {
    ALL_TASKBARS.store(enabled, Ordering::Relaxed);
}

pub fn set_content_width(width: f64) {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    CONTENT_WIDTH.store(width.clamp(40.0, 600.0).ceil() as u32, Ordering::Relaxed);
}

pub fn start(app: AppHandle) {
    let spawned = std::thread::Builder::new()
        .name("taskbar-dock".into())
        .spawn(move || watch(&app));
    if let Err(error) = spawned {
        log::error!("could not start the taskbar watcher: {error}");
    }
}

/// Where a strip should be, in screen pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Placement {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

/// One taskbar Explorer shows.
#[derive(Debug, Clone, Copy)]
struct Taskbar {
    hwnd: HWND,
    primary: bool,
}

/// A strip window and where it was last put.
struct Dock {
    label: String,
    styled: bool,
    current: Option<Placement>,
}

impl Dock {
    fn new(label: String) -> Self {
        Self {
            label,
            styled: false,
            current: None,
        }
    }

    fn apply(&mut self, app: &AppHandle, target: Option<Placement>) {
        let Some(hwnd) = app
            .get_webview_window(&self.label)
            .and_then(|window| window.hwnd().ok())
            .map(|raw| HWND(raw.0))
        else {
            return;
        };
        if !self.styled {
            make_tool_window(hwnd);
            self.styled = true;
        }
        // "Minimise all" (Win+M) and screen tools can minimise or hide the
        // strip behind the watcher's back; put it back like a moved taskbar.
        // SAFETY: state queries on a window this process owns.
        let displaced = unsafe { IsIconic(hwnd).as_bool() || !IsWindowVisible(hwnd).as_bool() };
        match (target, self.current) {
            (Some(target), previous) if previous != Some(target) || displaced => {
                // SAFETY: plain window positioning on a window this process owns.
                let _ = unsafe {
                    if displaced {
                        let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
                    }
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
        self.current = target;
    }
}

fn watch(app: &AppHandle) {
    let mut docks = vec![Dock::new(LABEL.to_owned())];
    let mut theme = system::taskbar_theme();

    loop {
        std::thread::sleep(TICK);
        let bars = if ENABLED.load(Ordering::Relaxed) {
            taskbars(ALL_TASKBARS.load(Ordering::Relaxed))
        } else {
            Vec::new()
        };
        // A taskbar seen for the first time gets its own strip window; it is
        // kept, hidden, when the monitor goes away.
        while docks.len() < bars.len() {
            let label = format!("{LABEL}-{}", docks.len());
            if let Err(error) = create_strip(app, &label) {
                log::warn!("could not create {label}: {error}");
                break;
            }
            docks.push(Dock::new(label));
        }
        for (index, dock) in docks.iter_mut().enumerate() {
            let target = bars
                .get(index)
                .filter(|bar| !full_screen_app_active(bar.hwnd))
                .and_then(|bar| placement(*bar));
            dock.apply(app, target);
        }

        let next_theme = system::taskbar_theme();
        if next_theme != theme {
            theme = next_theme;
            let _ = app.emit(events::TASKBAR_THEME, theme);
        }
    }
}

/// A copy of the main strip, built from its own declaration so both look the same.
fn create_strip(app: &AppHandle, label: &str) -> tauri::Result<()> {
    let Some(mut config) = app
        .config()
        .app
        .windows
        .iter()
        .find(|window| window.label == LABEL)
        .cloned()
    else {
        return Ok(());
    };
    label.clone_into(&mut config.label);
    WebviewWindowBuilder::from_config(app, &config)?.build()?;
    Ok(())
}

/// The main taskbar first, then the taskbars of other monitors when wanted.
fn taskbars(all: bool) -> Vec<Taskbar> {
    let mut bars = Vec::new();
    // SAFETY: read-only window lookups.
    unsafe {
        if let Ok(hwnd) = FindWindowW(w!("Shell_TrayWnd"), None) {
            bars.push(Taskbar {
                hwnd,
                primary: true,
            });
        }
        if all {
            let mut previous = None;
            while let Ok(hwnd) = FindWindowExW(None, previous, w!("Shell_SecondaryTrayWnd"), None) {
                bars.push(Taskbar {
                    hwnd,
                    primary: false,
                });
                previous = Some(hwnd);
            }
        }
    }
    bars
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

/// Explorer brings a taskbar to the front of the topmost band when it is clicked.
fn raise_above_taskbar(hwnd: HWND) {
    // SAFETY: reading the foreground window and re-ordering a window this process owns.
    unsafe {
        if is_taskbar(GetForegroundWindow()) {
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

fn monitor_info(monitor: HMONITOR) -> Option<MONITORINFO> {
    let mut info = MONITORINFO {
        cbSize: u32::try_from(std::mem::size_of::<MONITORINFO>()).ok()?,
        ..MONITORINFO::default()
    };
    // SAFETY: the out parameter is a local of the size it declares.
    unsafe { GetMonitorInfoW(monitor, &raw mut info) }
        .as_bool()
        .then_some(info)
}

fn placement(bar: Taskbar) -> Option<Placement> {
    // SAFETY: read-only queries on Explorer's windows; every out parameter is a local.
    unsafe {
        let mut rect = RECT::default();
        GetWindowRect(bar.hwnd, &raw mut rect).ok()?;

        // Vertical taskbars (Windows 10) have no room for a horizontal strip.
        if rect.bottom - rect.top > rect.right - rect.left {
            return None;
        }
        // An auto-hidden taskbar slides below the monitor edge.
        let info = monitor_info(MonitorFromWindow(bar.hwnd, MONITOR_DEFAULTTONEAREST))?;
        if rect.bottom > info.rcMonitor.bottom + 2 {
            return None;
        }

        let scale = f64::from(GetDpiForWindow(bar.hwnd)) / 96.0;
        let logical = |value: f64| {
            #[allow(clippy::cast_possible_truncation)]
            let pixels = (value * scale).round() as i32;
            pixels
        };
        let width = logical(f64::from(CONTENT_WIDTH.load(Ordering::Relaxed)));

        let tray_left = if bar.primary {
            FindWindowExW(Some(bar.hwnd), None, w!("TrayNotifyWnd"), None)
                .ok()
                .and_then(|tray| {
                    let mut tray_rect = RECT::default();
                    GetWindowRect(tray, &raw mut tray_rect)
                        .ok()
                        .map(|()| tray_rect.left)
                })
                .unwrap_or(rect.right - width * 2)
        } else {
            rect.right - logical(SECONDARY_TRAY)
        };

        Some(Placement {
            x: tray_left - width - logical(GAP),
            y: rect.top,
            width,
            height: rect.bottom - rect.top,
        })
    }
}

/// A game, a video or a slideshow covers the taskbar's monitor edge to edge.
///
/// `SHQueryUserNotificationState` is not used: it reports "busy" for a
/// full-screen window on any monitor, and for live wallpapers, which would
/// hide the strip on a perfectly visible taskbar.
fn full_screen_app_active(taskbar: HWND) -> bool {
    // SAFETY: read-only queries; every out parameter is a local.
    unsafe {
        let foreground = GetForegroundWindow();
        if foreground.is_invalid() || is_shell_window(foreground) {
            return false;
        }
        let monitor = MonitorFromWindow(taskbar, MONITOR_DEFAULTTONEAREST);
        if MonitorFromWindow(foreground, MONITOR_DEFAULTTONEAREST) != monitor {
            return false;
        }
        let Some(info) = monitor_info(monitor) else {
            return false;
        };
        let mut window = RECT::default();
        GetWindowRect(foreground, &raw mut window).is_ok()
            && window.left <= info.rcMonitor.left
            && window.top <= info.rcMonitor.top
            && window.right >= info.rcMonitor.right
            && window.bottom >= info.rcMonitor.bottom
    }
}

fn class_name(hwnd: HWND) -> String {
    let mut class = [0u16; 32];
    // SAFETY: the buffer length is passed alongside the buffer.
    let length = unsafe { GetClassNameW(hwnd, &mut class) };
    String::from_utf16_lossy(&class[..usize::try_from(length).unwrap_or(0)])
}

fn is_taskbar(hwnd: HWND) -> bool {
    matches!(
        class_name(hwnd).as_str(),
        "Shell_TrayWnd" | "Shell_SecondaryTrayWnd"
    )
}

/// The desktop and the taskbars span the monitor too, but they are not apps.
fn is_shell_window(hwnd: HWND) -> bool {
    is_taskbar(hwnd) || matches!(class_name(hwnd).as_str(), "Progman" | "WorkerW")
}
