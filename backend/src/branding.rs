//! One-way migration of local settings and credential namespaces after the rename.
use crate::core::Res;
use std::path::{Path, PathBuf};

pub fn credential(account: &str) -> Res<keyring::Entry> {
    let profile = std::env::var("GVPAK_PROFILE").unwrap_or("dev".into());
    let entry = keyring::Entry::new(&format!("GVPAK-STUDIO-{profile}"), account)
        .map_err(|_| "KEYRING_UNAVAILABLE")?;
    if matches!(entry.get_password(), Err(keyring::Error::NoEntry)) {
        // This legacy identifier is read only for migration; secrets never enter files or logs.
        let legacy = keyring::Entry::new(&format!("EDGE-STUDIO-{profile}"), account)
            .map_err(|_| "KEYRING_UNAVAILABLE")?;
        match legacy.get_password() {
            Ok(value) => {
                entry.set_password(&value).map_err(|_| "KEYRING_MIGRATION_FAILED")?;
                legacy.delete_credential().map_err(|_| "KEYRING_MIGRATION_FAILED")?;
            }
            Err(keyring::Error::NoEntry) => {}
            Err(_) => return Err("KEYRING_UNAVAILABLE".into()),
        }
    }
    Ok(entry)
}

pub fn settings_path(directory: &Path, profile: &str) -> Res<PathBuf> {
    let current = directory.join(format!("gvpak-studio-settings-{profile}.json"));
    let legacy = directory.join(format!("edge-settings-{profile}.json"));
    if !current.exists() && legacy.exists() {
        std::fs::copy(&legacy, &current).map_err(|_| "SETTINGS_MIGRATION_FAILED")?;
    }
    Ok(current)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_migrate_without_overwriting_newer_preferences() {
        let temp = tempfile::tempdir().unwrap();
        let legacy = temp.path().join("edge-settings-test.json");
        std::fs::write(&legacy, r#"{"dataRoot":"fixture-projects"}"#).unwrap();
        let current = settings_path(temp.path(), "test").unwrap();
        assert_eq!(std::fs::read(&current).unwrap(), std::fs::read(&legacy).unwrap());
        std::fs::write(&current, r#"{"dataRoot":"newer-fixture"}"#).unwrap();
        settings_path(temp.path(), "test").unwrap();
        assert!(std::fs::read_to_string(current).unwrap().contains("newer-fixture"));
    }
}
