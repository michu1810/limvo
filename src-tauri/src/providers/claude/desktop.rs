//! Sign-ins kept by the Claude desktop app.
//!
//! The desktop app is an Electron application. It stores its OAuth token cache
//! in `config.json`, encrypted with Chromium's `OSCrypt` scheme: an AES-256-GCM
//! key wrapped with DPAPI in the neighbouring `Local State` file, and each value
//! stored as `"v10" || nonce || ciphertext || tag`. DPAPI only unwraps the key
//! for the signed-in Windows user, so this works for the owner of the account
//! and nobody else.

use super::credentials::Token;

#[cfg(windows)]
pub fn tokens() -> Vec<Token> {
    windows_impl::tokens()
}

/// The macOS app keeps its cache in the Keychain; support comes with the macOS port.
#[cfg(not(windows))]
pub fn tokens() -> Vec<Token> {
    Vec::new()
}

#[cfg(windows)]
mod windows_impl {
    use std::path::{Path, PathBuf};

    use aes_gcm::aead::{Aead, KeyInit};
    use aes_gcm::{Aes256Gcm, Nonce};
    use base64::Engine;
    use base64::engine::general_purpose::STANDARD;
    use windows::Win32::Foundation::{HLOCAL, LocalFree};
    use windows::Win32::Security::Cryptography::{CRYPT_INTEGER_BLOB, CryptUnprotectData};

    use super::Token;
    use crate::providers::claude::credentials::PROFILE_SCOPE;

    /// Newest layout first; the app keeps both and does not always refresh both.
    const CACHE_KEYS: [&str; 2] = ["oauth:tokenCacheV2", "oauth:tokenCache"];
    const NONCE_LEN: usize = 12;
    const TAG_LEN: usize = 16;

    pub fn tokens() -> Vec<Token> {
        config_files()
            .iter()
            .flat_map(|config| read_config(config).unwrap_or_default())
            .collect()
    }

    /// The classic installer, the Microsoft Store package and its virtualised folder.
    fn config_files() -> Vec<PathBuf> {
        let mut files = Vec::new();
        if let Some(roaming) = std::env::var_os("APPDATA") {
            files.push(PathBuf::from(roaming).join("Claude").join("config.json"));
        }
        if let Some(local) = std::env::var_os("LOCALAPPDATA").map(PathBuf::from) {
            files.extend(children(&local.join("Claude-Data")).map(|dir| dir.join("config.json")));
            files.extend(
                children(&local.join("Packages"))
                    .filter(|dir| {
                        dir.file_name()
                            .is_some_and(|name| name.to_string_lossy().starts_with("Claude_"))
                    })
                    .map(|dir| {
                        dir.join("LocalCache")
                            .join("Roaming")
                            .join("Claude")
                            .join("config.json")
                    }),
            );
        }
        files.retain(|file| file.is_file());
        files
    }

    fn children(dir: &Path) -> impl Iterator<Item = PathBuf> {
        std::fs::read_dir(dir)
            .into_iter()
            .flatten()
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path.is_dir())
    }

    fn read_config(config_path: &Path) -> Option<Vec<Token>> {
        let config: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(config_path).ok()?).ok()?;
        let key = os_crypt_key(&config_path.with_file_name("Local State"))?;

        let mut found = Vec::new();
        for cache_key in CACHE_KEYS {
            let Some(encrypted) = config.get(cache_key).and_then(serde_json::Value::as_str) else {
                continue;
            };
            let Some(plaintext) = decrypt(encrypted, &key) else {
                log::warn!("could not decrypt the Claude desktop {cache_key}");
                continue;
            };
            found.extend(parse_cache(&plaintext));
        }
        Some(found)
    }

    /// Cache entries are keyed `"<install>:<user>:<base url>:<scopes>"`.
    pub(super) fn parse_cache(plaintext: &[u8]) -> Vec<Token> {
        let Ok(serde_json::Value::Object(entries)) = serde_json::from_slice(plaintext) else {
            return Vec::new();
        };
        entries
            .iter()
            .filter_map(|(key, entry)| {
                let secret = entry.get("token")?.as_str()?.to_owned();
                Some(Token::from_desktop(
                    secret,
                    entry.get("expiresAt").and_then(serde_json::Value::as_i64),
                    key.contains(PROFILE_SCOPE),
                ))
            })
            .collect()
    }

    fn os_crypt_key(local_state: &Path) -> Option<Vec<u8>> {
        let state: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(local_state).ok()?).ok()?;
        let wrapped = STANDARD
            .decode(state.get("os_crypt")?.get("encrypted_key")?.as_str()?)
            .ok()?;
        dpapi_unprotect(wrapped.strip_prefix(b"DPAPI")?)
    }

    fn decrypt(value: &str, key: &[u8]) -> Option<Vec<u8>> {
        let blob = STANDARD.decode(value).ok()?;
        let body = blob.strip_prefix(b"v10")?;
        if body.len() < NONCE_LEN + TAG_LEN {
            return None;
        }
        let (nonce, sealed) = body.split_at(NONCE_LEN);
        let cipher = Aes256Gcm::new_from_slice(key).ok()?;
        let nonce = Nonce::try_from(nonce).ok()?;
        cipher.decrypt(&nonce, sealed).ok()
    }

    fn dpapi_unprotect(data: &[u8]) -> Option<Vec<u8>> {
        let input = CRYPT_INTEGER_BLOB {
            cbData: u32::try_from(data.len()).ok()?,
            pbData: data.as_ptr().cast_mut(),
        };
        let mut output = CRYPT_INTEGER_BLOB::default();
        // SAFETY: `input` borrows `data` for the duration of the call, and
        // `output` is allocated by the system and released below with LocalFree.
        unsafe {
            CryptUnprotectData(&raw const input, None, None, None, None, 0, &raw mut output)
                .ok()?;
            if output.pbData.is_null() {
                return None;
            }
            let key = std::slice::from_raw_parts(output.pbData, output.cbData as usize).to_vec();
            let _ = LocalFree(Some(HLOCAL(output.pbData.cast())));
            Some(key)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn reads_profile_scope_from_the_entry_key() {
            let tokens = parse_cache(
                br#"{
                    "a:b:https://api.anthropic.com:user:inference user:profile": {"token": "t1", "expiresAt": 5},
                    "a:b:https://api.anthropic.com:user:inference": {"token": "t2"},
                    "broken": {"expiresAt": 1}
                }"#,
            );
            assert_eq!(tokens.len(), 2);
            let scoped: Vec<_> = tokens.iter().map(|token| token.has_profile_scope).collect();
            assert!(scoped.contains(&Some(true)) && scoped.contains(&Some(false)));
        }

        #[test]
        fn rejects_values_that_are_not_os_crypt_blobs() {
            assert!(decrypt("bm90IGEgYmxvYg==", &[0; 32]).is_none());
            assert!(decrypt("%%%", &[0; 32]).is_none());
        }
    }
}
