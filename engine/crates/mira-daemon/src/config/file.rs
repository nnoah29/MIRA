//! Fichier `daemon.toml` du dossier MIRA : paramètres de démarrage seulement, lus avant d'ouvrir quoi que ce soit.

use std::path::{Path, PathBuf};

use mira_assembly::mira_vocabulary::error::codes::invalid_input;
use mira_assembly::mira_vocabulary::{ErrorCategory, MiraError, ProfileId};
use mira_assembly::profile::{ProfileEnvironment, ProfilePaths};
use serde::Deserialize;

const MAX_BYTES: u64 = 64 * 1024;

/// Valeurs du fichier ; une option de la ligne de commande l'emporte toujours.
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct FileConfig {
    pub profile: Option<String>,
    pub home: Option<PathBuf>,
    pub socket: Option<PathBuf>,
    pub token_file: Option<PathBuf>,
}

/// Erreur fixe : ni le chemin ni le contenu du fichier n'apparaissent.
fn invalid() -> MiraError {
    MiraError::new(
        ErrorCategory::InvalidInput,
        invalid_input::PARAMETER,
        "fichier de configuration du démon invalide",
    )
}

/// Emplacement du fichier : à la racine du dossier MIRA, à côté de `profiles/`.
pub fn config_file_path(environment: &ProfileEnvironment) -> Result<PathBuf, MiraError> {
    let root = ProfilePaths::resolve(&ProfileId::from_raw("default"), environment)?.root;
    root.parent()
        .and_then(Path::parent)
        .map(|dir| dir.join("daemon.toml"))
        .ok_or_else(invalid)
}

/// Lit le fichier ; absent, il donne la configuration vide ; trop gros, illisible ou aux clés inconnues, il est refusé.
pub fn read_file_config(path: &Path) -> Result<FileConfig, MiraError> {
    let metadata = match std::fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(FileConfig::default());
        }
        Err(_) => return Err(invalid()),
    };
    if metadata.len() > MAX_BYTES {
        return Err(invalid());
    }
    let text = std::fs::read_to_string(path).map_err(|_| invalid())?;
    toml::from_str(&text).map_err(|_| invalid())
}
