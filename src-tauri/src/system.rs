//! Facts about the operating system the UI cannot read on its own.

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Material {
    /// Windows 11 Mica behind the flyout.
    Mica,
    /// No native material; the UI paints a solid surface.
    None,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HostInfo {
    pub material: Material,
    pub taskbar_theme: Theme,
    /// Installed from the Microsoft Store, which delivers the updates.
    pub store: bool,
}

pub fn host_info() -> HostInfo {
    HostInfo {
        material: if supports_mica() {
            Material::Mica
        } else {
            Material::None
        },
        taskbar_theme: taskbar_theme(),
        store: crate::package::is_packaged(),
    }
}

#[cfg(windows)]
pub use windows_impl::{preferred_languages, taskbar_theme};

#[cfg(windows)]
fn supports_mica() -> bool {
    // Mica arrived with Windows 11, build 22000.
    windows_impl::build_number().is_some_and(|build| build >= 22_000)
}

#[cfg(not(windows))]
fn supports_mica() -> bool {
    false
}

#[cfg(not(windows))]
pub fn preferred_languages() -> Vec<String> {
    Vec::new()
}

#[cfg(not(windows))]
pub fn taskbar_theme() -> Theme {
    Theme::Dark
}

#[cfg(windows)]
mod windows_impl {
    use windows::Win32::Globalization::{GetUserPreferredUILanguages, MUI_LANGUAGE_NAME};
    use windows::Win32::System::Registry::{
        HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, RRF_RT_REG_DWORD, RRF_RT_REG_SZ, RegGetValueW,
    };
    use windows::core::{PWSTR, w};

    use super::Theme;

    /// The taskbar follows the "Windows mode" setting, which can differ from the app mode.
    pub fn taskbar_theme() -> Theme {
        let mut value: u32 = 0;
        let mut size = u32::try_from(std::mem::size_of::<u32>()).unwrap_or(4);
        // SAFETY: the buffer is a valid, correctly sized u32 for a REG_DWORD read.
        let read = unsafe {
            RegGetValueW(
                HKEY_CURRENT_USER,
                w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize"),
                w!("SystemUsesLightTheme"),
                RRF_RT_REG_DWORD,
                None,
                Some((&raw mut value).cast()),
                Some(&raw mut size),
            )
        };
        if read.is_ok() && value == 1 {
            Theme::Light
        } else {
            Theme::Dark
        }
    }

    pub fn build_number() -> Option<u32> {
        let mut buffer = [0u16; 16];
        let mut size = u32::try_from(std::mem::size_of_val(&buffer)).ok()?;
        // SAFETY: the buffer and its byte size describe the same allocation.
        let read = unsafe {
            RegGetValueW(
                HKEY_LOCAL_MACHINE,
                w!("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion"),
                w!("CurrentBuildNumber"),
                RRF_RT_REG_SZ,
                None,
                Some(buffer.as_mut_ptr().cast()),
                Some(&raw mut size),
            )
        };
        if read.is_err() {
            return None;
        }
        let text = String::from_utf16_lossy(&buffer);
        text.trim_end_matches('\0').trim().parse().ok()
    }

    /// The display languages chosen in Windows settings, most preferred first.
    pub fn preferred_languages() -> Vec<String> {
        let mut count = 0u32;
        let mut length = 0u32;
        // SAFETY: the first call only asks for the buffer length; the second
        // passes a buffer of exactly that many UTF-16 units.
        unsafe {
            if GetUserPreferredUILanguages(MUI_LANGUAGE_NAME, &raw mut count, None, &raw mut length)
                .is_err()
            {
                return Vec::new();
            }
            let mut buffer = vec![0u16; length as usize];
            if GetUserPreferredUILanguages(
                MUI_LANGUAGE_NAME,
                &raw mut count,
                Some(PWSTR(buffer.as_mut_ptr())),
                &raw mut length,
            )
            .is_err()
            {
                return Vec::new();
            }
            // A list of NUL-terminated names, ended by an extra NUL.
            buffer
                .split(|unit| *unit == 0)
                .filter(|name| !name.is_empty())
                .map(String::from_utf16_lossy)
                .collect()
        }
    }
}
