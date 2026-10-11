#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Verrou exclusif d'un profil (D-27).

use mira_core::profile::{ProfileEnvironment, ProfileLock, ProfilePaths};
use mira_vocabulary::ProfileId;

/// Chemins d'un profil dans un dossier temporaire
fn temporary_paths(dir: &tempfile::TempDir, name: &str) -> ProfilePaths {
    let env = ProfileEnvironment {
        mira_home: Some(dir.path().to_path_buf()),
        ..ProfileEnvironment::default()
    };
    ProfilePaths::resolve(&ProfileId::from_raw(name), &env).unwrap()
}

/// Un second moteur sur le même profil est refusé avec le bon code
#[test]
fn second_acquire_is_refused_while_first_is_held() {
    let dir = tempfile::tempdir().unwrap();
    let paths = temporary_paths(&dir, "shared");
    let _first = ProfileLock::acquire(&paths).unwrap();
    let error = ProfileLock::acquire(&paths).unwrap_err();
    assert_eq!(error.code, "incompatible_state.profile_locked");
}

/// Le verrou est libéré quand sa valeur est détruite
#[test]
fn lock_is_released_on_drop() {
    let dir = tempfile::tempdir().unwrap();
    let paths = temporary_paths(&dir, "shared");
    let first = ProfileLock::acquire(&paths).unwrap();
    drop(first);
    assert!(ProfileLock::acquire(&paths).is_ok());
}

/// Deux profils différents se verrouillent indépendamment
#[test]
fn different_profiles_do_not_conflict() {
    let dir = tempfile::tempdir().unwrap();
    let _one = ProfileLock::acquire(&temporary_paths(&dir, "one")).unwrap();
    assert!(ProfileLock::acquire(&temporary_paths(&dir, "two")).is_ok());
}
