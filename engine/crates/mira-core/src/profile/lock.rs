//! Verrou exclusif d'un profil : un seul moteur à la fois (D-27).

use std::fs::{File, OpenOptions, TryLockError};

use mira_vocabulary::error::codes::incompatible_state;
use mira_vocabulary::{ErrorCategory, MiraError};

use super::errors::io_error;
use super::paths::ProfilePaths;

/// Verrou tenu tant que la valeur existe ; libéré à sa destruction.
#[derive(Debug)]
pub struct ProfileLock {
    _file: File,
}

impl ProfileLock {
    /// Prend le verrou du profil, ou échoue si un autre moteur le tient déjà
    pub fn acquire(paths: &ProfilePaths) -> Result<Self, MiraError> {
        paths.ensure_created()?;
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(paths.lock_file())
            .map_err(|error| io_error("ouverture du verrou du profil impossible", &error))?;
        match file.try_lock() {
            Ok(()) => Ok(Self { _file: file }),
            Err(TryLockError::WouldBlock) => Err(MiraError::new(
                ErrorCategory::IncompatibleState,
                incompatible_state::PROFILE_LOCKED,
                "ce profil est déjà utilisé par un autre moteur",
            )),
            Err(TryLockError::Error(error)) => {
                Err(io_error("verrouillage du profil impossible", &error))
            }
        }
    }
}
