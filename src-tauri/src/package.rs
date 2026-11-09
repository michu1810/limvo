//! Whether Limvo runs from its MSIX package, the Microsoft Store build.
//!
//! The same `limvo.exe` ships in the installer and in the package. Packaged,
//! Windows owns two things the installer build does itself: the Store
//! delivers updates, so the in-app updater stays off, and start at sign-in
//! goes through the package's startup task, because the registry `Run` key
//! the installer build uses is virtualised inside a package and never seen.

use std::sync::OnceLock;

/// The `TaskId` of the `windows.startupTask` extension in `AppxManifest.xml`.
#[cfg(windows)]
const STARTUP_TASK_ID: &str = "LimvoStartup";

/// True when the process has package identity. Decided once: it cannot change
/// while the process runs.
pub fn is_packaged() -> bool {
    static PACKAGED: OnceLock<bool> = OnceLock::new();
    *PACKAGED.get_or_init(has_package_identity)
}

#[cfg(windows)]
fn has_package_identity() -> bool {
    use windows::Win32::Foundation::APPMODEL_ERROR_NO_PACKAGE;
    use windows::Win32::Storage::Packaging::Appx::GetCurrentPackageFullName;

    let mut length = 0u32;
    // With no buffer the call only reports the length the name needs; without
    // a package it says so instead.
    let status = unsafe { GetCurrentPackageFullName(&raw mut length, None) };
    status != APPMODEL_ERROR_NO_PACKAGE
}

#[cfg(not(windows))]
fn has_package_identity() -> bool {
    false
}

/// Whether the startup task will start Limvo at sign-in.
#[cfg(windows)]
pub async fn launch_at_login() -> Result<bool, String> {
    let task = startup_task().await?;
    let state = task.State().map_err(|error| error.to_string())?;
    Ok(starts_at_login(state))
}

/// Turns the startup task on or off and returns whether it is on afterwards.
///
/// Turning it on can fail without an error: once the user switched Limvo off
/// in Task Manager or Settings, or a policy forbids it, only they can switch
/// it back, and the task stays off.
#[cfg(windows)]
pub async fn set_launch_at_login(enabled: bool) -> Result<bool, String> {
    let task = startup_task().await?;
    let state = if enabled {
        task.RequestEnableAsync()
            .map_err(|error| error.to_string())?
            .await
            .map_err(|error| error.to_string())?
    } else {
        task.Disable().map_err(|error| error.to_string())?;
        task.State().map_err(|error| error.to_string())?
    };
    Ok(starts_at_login(state))
}

#[cfg(windows)]
async fn startup_task() -> Result<windows::ApplicationModel::StartupTask, String> {
    use windows::ApplicationModel::StartupTask;
    use windows::core::HSTRING;

    StartupTask::GetAsync(&HSTRING::from(STARTUP_TASK_ID))
        .map_err(|error| error.to_string())?
        .await
        .map_err(|error| error.to_string())
}

#[cfg(windows)]
fn starts_at_login(state: windows::ApplicationModel::StartupTaskState) -> bool {
    use windows::ApplicationModel::StartupTaskState;

    matches!(
        state,
        StartupTaskState::Enabled | StartupTaskState::EnabledByPolicy
    )
}
