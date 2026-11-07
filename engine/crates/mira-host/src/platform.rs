//! Adaptation de la configuration aux systèmes sans dossiers XDG ni service de secrets (Android).

use std::path::PathBuf;
use std::sync::Arc;

use mira_assembly::api::EngineConfig;
use mira_assembly::mira_vocabulary::error::codes::invalid_input;
use mira_assembly::mira_vocabulary::{ErrorCategory, MiraError};

use crate::file_secrets::FileSecretStore;

/// Nom du fichier du trousseau chiffré, à la racine du dossier de données.
const SECRETS_FILE: &str = "secrets.bin";

/// Applique le dossier de données et le trousseau chiffré demandés par l'application.
pub fn adapt(
    config: &mut EngineConfig,
    home: Option<String>,
    secrets_key: Option<Vec<u8>>,
) -> Result<(), MiraError> {
    let home = home.filter(|h| !h.is_empty()).map(PathBuf::from);
    if let Some(dir) = &home {
        config.environment.mira_home = Some(dir.clone());
    }
    let Some(key) = secrets_key else {
        return Ok(());
    };
    let Some(dir) = home else {
        return Err(MiraError::new(
            ErrorCategory::InvalidInput,
            invalid_input::PARAMETER,
            "le trousseau chiffré demande un dossier de données",
        ));
    };
    std::fs::create_dir_all(&dir).map_err(|_| {
        MiraError::new(
            ErrorCategory::InvalidInput,
            invalid_input::PARAMETER,
            "dossier de données inaccessible",
        )
    })?;
    config.secrets = Arc::new(FileSecretStore::new(dir.join(SECRETS_FILE), &key)?);
    Ok(())
}
