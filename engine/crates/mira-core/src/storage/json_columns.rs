//! Colonnes JSON : sérialisation des valeurs composées du domaine stockées en texte.

use mira_vocabulary::error::codes::local_storage;
use mira_vocabulary::{ErrorCategory, MiraError};
use serde::Serialize;
use serde::de::DeserializeOwned;

/// Sérialise une valeur du domaine pour une colonne JSON
pub(crate) fn to_json<T: Serialize>(value: &T) -> Result<String, MiraError> {
    serde_json::to_string(value).map_err(|error| json_error(&error))
}

/// Relit une valeur du domaine depuis une colonne JSON
pub(crate) fn from_json<T: DeserializeOwned>(text: &str) -> Result<T, MiraError> {
    serde_json::from_str(text).map_err(|error| json_error(&error))
}

/// Erreur de colonne JSON illisible (base corrompue ou migration manquante)
fn json_error(error: &serde_json::Error) -> MiraError {
    MiraError::new(
        ErrorCategory::LocalStorage,
        local_storage::CORRUPTED,
        format!("colonne JSON illisible : {error}"),
    )
}
