#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Chemin du fichier de configuration lu depuis l'environnement, y compris le repli Windows.

use std::ffi::OsString;
use std::path::PathBuf;

use mira_assembly::oauth_file_path::oauth_config_path_from_lookup;

/// Sous Windows, sans `XDG_CONFIG_HOME` ni `HOME`, `APPDATA` désigne le dossier de configuration
#[test]
fn appdata_is_the_windows_fallback() {
    let lookup = |name: &str| match name {
        "APPDATA" => Some(OsString::from("/appdata")),
        _ => None,
    };
    assert_eq!(
        oauth_config_path_from_lookup(&lookup),
        Some(PathBuf::from("/appdata/mira/config.toml"))
    );
}

/// `MIRA_CONFIG` l'emporte sur toute autre variable
#[test]
fn mira_config_wins() {
    let lookup = |name: &str| match name {
        "MIRA_CONFIG" => Some(OsString::from("/c.toml")),
        "APPDATA" => Some(OsString::from("/appdata")),
        _ => None,
    };
    assert_eq!(
        oauth_config_path_from_lookup(&lookup),
        Some(PathBuf::from("/c.toml"))
    );
}
