//! Erreurs du dépôt des modèles de messages.

use mira_vocabulary::error::codes::{invalid_input, not_found};
use mira_vocabulary::{ErrorCategory, MiraError};

/// Erreur « modèle inconnu »
pub(super) fn template_not_found() -> MiraError {
    MiraError::new(
        ErrorCategory::NotFound,
        not_found::TEMPLATE,
        "modèle de message inconnu",
    )
}

/// Erreur « nom déjà pris »
pub(super) fn name_taken() -> MiraError {
    MiraError::new(
        ErrorCategory::InvalidInput,
        invalid_input::TEMPLATE_NAME_TAKEN,
        "un modèle porte déjà ce nom",
    )
}

/// Erreur « nom vide »
pub(super) fn name_empty() -> MiraError {
    MiraError::new(
        ErrorCategory::InvalidInput,
        invalid_input::TEMPLATE_NAME_EMPTY,
        "le nom du modèle est vide",
    )
}

/// Erreur « champ trop long »
pub(super) fn too_long() -> MiraError {
    MiraError::new(
        ErrorCategory::InvalidInput,
        invalid_input::TEMPLATE_TOO_LONG,
        "nom, objet ou corps du modèle trop long",
    )
}
