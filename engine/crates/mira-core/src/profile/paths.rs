//! Emplacements des fichiers d'un profil et création du dossier avec droits restreints.

use std::fs::{self, DirBuilder, Permissions};
use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
use std::path::{Path, PathBuf};

use mira_vocabulary::error::codes::{invalid_input, local_storage};
use mira_vocabulary::{ErrorCategory, MiraError, ProfileId};

use super::environment::ProfileEnvironment;
use super::errors::io_error;

/// Chemins d'un profil : racine, base, cache des pièces jointes, dossier temporaire.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfilePaths {
    pub root: PathBuf,
    pub database: PathBuf,
    pub attachments_cache: PathBuf,
    pub temp: PathBuf,
}

impl ProfilePaths {
    /// Calcule les chemins : `MIRA_HOME` d'abord, puis `XDG_DATA_HOME`, puis `~/.local/share`
    pub fn resolve(profile: &ProfileId, env: &ProfileEnvironment) -> Result<Self, MiraError> {
        validate_profile_name(profile.as_str())?;
        let profiles = match (&env.mira_home, &env.xdg_data_home, &env.home) {
            (Some(home), _, _) if !home.as_os_str().is_empty() => home.join("profiles"),
            (_, Some(xdg), _) if !xdg.as_os_str().is_empty() => xdg.join("mira/profiles"),
            (_, _, Some(home)) if !home.as_os_str().is_empty() => {
                home.join(".local/share/mira/profiles")
            }
            _ => return Err(missing_base_directory()),
        };
        let root = profiles.join(profile.as_str());
        Ok(Self {
            database: root.join("mira.sqlite3"),
            attachments_cache: root.join("attachments"),
            temp: root.join("tmp"),
            root,
        })
    }

    /// Crée les dossiers du profil avec les droits 0o700
    pub fn ensure_created(&self) -> Result<(), MiraError> {
        for directory in [&self.root, &self.attachments_cache, &self.temp] {
            create_private_directory(directory)?;
        }
        Ok(())
    }

    /// Donne le chemin du fichier de verrou du profil
    pub fn lock_file(&self) -> PathBuf {
        self.root.join("engine.lock")
    }
}

/// Crée un dossier (et ses parents) en n'autorisant que le propriétaire
fn create_private_directory(directory: &Path) -> Result<(), MiraError> {
    let created = DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(directory);
    created
        .and_then(|()| fs::set_permissions(directory, Permissions::from_mode(0o700)))
        .map_err(|error| io_error("création du dossier du profil impossible", &error))
}

/// Refuse un nom de profil qui sortirait du dossier des profils
fn validate_profile_name(name: &str) -> Result<(), MiraError> {
    let unsafe_name =
        name.is_empty() || name == "." || name == ".." || name.contains(['/', '\\', '\0']);
    if unsafe_name {
        return Err(MiraError::new(
            ErrorCategory::InvalidInput,
            invalid_input::PARAMETER,
            "nom de profil invalide",
        ));
    }
    Ok(())
}

/// Erreur quand aucune variable ne permet de situer les données
fn missing_base_directory() -> MiraError {
    MiraError::new(
        ErrorCategory::LocalStorage,
        local_storage::IO,
        "aucun dossier de données : définir MIRA_HOME, XDG_DATA_HOME ou HOME",
    )
}
