#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Lecture des variables d'emplacement : priorité de `XDG_DATA_HOME` et repli Windows.

use std::ffi::OsString;
use std::path::PathBuf;

use mira_core::profile::ProfileEnvironment;

/// Sous Windows, sans `XDG_DATA_HOME`, le dossier de données local de l'utilisateur sert de base
#[test]
fn local_app_data_replaces_the_missing_xdg_variable() {
    let lookup = |name: &str| match name {
        "LOCALAPPDATA" => Some(OsString::from("C:\\Users\\a\\AppData\\Local")),
        _ => None,
    };
    assert_eq!(
        ProfileEnvironment::from_lookup(&lookup).xdg_data_home,
        Some(PathBuf::from("C:\\Users\\a\\AppData\\Local"))
    );
}

/// `XDG_DATA_HOME` garde la priorité sur `LOCALAPPDATA`
#[test]
fn xdg_data_home_wins_over_local_app_data() {
    let lookup = |name: &str| match name {
        "XDG_DATA_HOME" => Some(OsString::from("/x")),
        "LOCALAPPDATA" => Some(OsString::from("/y")),
        _ => None,
    };
    assert_eq!(
        ProfileEnvironment::from_lookup(&lookup).xdg_data_home,
        Some(PathBuf::from("/x"))
    );
}
