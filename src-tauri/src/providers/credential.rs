//! Generic credentials of the Windows Credential Manager, read for this user only.
//!
//! Services that keep their sign-in there (Antigravity, the GitHub CLI) are
//! read through this module. It only reads: nothing is ever written, renewed
//! or deleted.

#[cfg(windows)]
mod platform {
    use windows::Win32::Security::Credentials::{
        CRED_TYPE_GENERIC, CREDENTIALW, CredFree, CredReadW,
    };
    use windows::core::HSTRING;

    pub fn read(target: &str) -> Option<Vec<u8>> {
        let mut credential: *mut CREDENTIALW = std::ptr::null_mut();
        // SAFETY: CredReadW fills `credential` with a buffer it owns; it is
        // copied out and released with CredFree before returning.
        unsafe {
            CredReadW(
                &HSTRING::from(target),
                CRED_TYPE_GENERIC,
                None,
                &raw mut credential,
            )
            .ok()?;
            let blob = (*credential).CredentialBlob;
            let size = usize::try_from((*credential).CredentialBlobSize).ok();
            let secret = match size {
                Some(size) if !blob.is_null() && size > 0 => {
                    Some(std::slice::from_raw_parts(blob, size).to_vec())
                }
                _ => None,
            };
            CredFree(credential.cast());
            secret
        }
    }
}

#[cfg(not(windows))]
mod platform {
    pub fn read(_target: &str) -> Option<Vec<u8>> {
        None
    }
}

/// The secret of the generic credential named `target`, if there is one.
pub fn read(target: &str) -> Option<Vec<u8>> {
    platform::read(target)
}
