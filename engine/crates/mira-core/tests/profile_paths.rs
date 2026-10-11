#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Emplacements d'un profil : priorité des variables, noms refusés, droits des dossiers.

use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use mira_core::profile::{ProfileEnvironment, ProfilePaths};
use mira_vocabulary::ProfileId;

/// Environnement de test avec les trois variables renseignées
fn full_environment() -> ProfileEnvironment {
    ProfileEnvironment {
        mira_home: Some(PathBuf::from("/data/mira-home")),
        xdg_data_home: Some(PathBuf::from("/data/xdg")),
        home: Some(PathBuf::from("/home/alice")),
    }
}

/// MIRA_HOME est prioritaire sur XDG_DATA_HOME et HOME
#[test]
fn mira_home_takes_priority() {
    let paths = ProfilePaths::resolve(&ProfileId::from_raw("work"), &full_environment()).unwrap();
    assert_eq!(paths.root, PathBuf::from("/data/mira-home/profiles/work"));
    assert_eq!(paths.database, paths.root.join("mira.sqlite3"));
    assert_eq!(paths.attachments_cache, paths.root.join("attachments"));
    assert_eq!(paths.temp, paths.root.join("tmp"));
}

/// Sans MIRA_HOME, XDG_DATA_HOME puis HOME donnent la racine
#[test]
fn xdg_then_home_are_fallbacks() {
    let mut env = full_environment();
    env.mira_home = None;
    let profile = ProfileId::from_raw("default");
    let xdg = ProfilePaths::resolve(&profile, &env).unwrap();
    assert_eq!(xdg.root, PathBuf::from("/data/xdg/mira/profiles/default"));
    env.xdg_data_home = None;
    let home = ProfilePaths::resolve(&profile, &env).unwrap();
    assert_eq!(
        home.root,
        PathBuf::from("/home/alice/.local/share/mira/profiles/default")
    );
}

/// Sans aucune variable, ou avec un nom de profil dangereux, la résolution échoue
#[test]
fn missing_base_or_unsafe_name_is_refused() {
    let profile = ProfileId::from_raw("default");
    assert!(ProfilePaths::resolve(&profile, &ProfileEnvironment::default()).is_err());
    for name in ["", "..", "a/b", "../x"] {
        let result = ProfilePaths::resolve(&ProfileId::from_raw(name), &full_environment());
        assert_eq!(result.unwrap_err().code, "invalid_input.parameter");
    }
}

/// Les dossiers du profil sont créés avec les droits 0o700, de façon répétable
#[test]
fn ensure_created_uses_private_permissions() {
    let dir = tempfile::tempdir().unwrap();
    let env = ProfileEnvironment {
        mira_home: Some(dir.path().to_path_buf()),
        ..ProfileEnvironment::default()
    };
    let paths = ProfilePaths::resolve(&ProfileId::from_raw("p1"), &env).unwrap();
    paths.ensure_created().unwrap();
    paths.ensure_created().unwrap();
    for directory in [&paths.root, &paths.attachments_cache, &paths.temp] {
        let mode = std::fs::metadata(directory).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o700);
    }
}
