//! Erreurs du rejeu : message disparu du serveur, boîte absente, envoi sans serveur SMTP.

use mira_vocabulary::error::codes::{invalid_input, not_found};
use mira_vocabulary::{ErrorCategory, MiraError};

/// Le message visé n'existe plus sur le serveur (échec définitif, D-23)
pub(crate) fn message_gone() -> MiraError {
    MiraError::new(
        ErrorCategory::NotFound,
        not_found::MESSAGE,
        "le message n'existe plus sur le serveur",
    )
}

/// La boîte visée n'existe pas sur le serveur
pub(crate) fn mailbox_gone() -> MiraError {
    MiraError::new(
        ErrorCategory::NotFound,
        not_found::MAILBOX,
        "la boîte n'existe pas sur le serveur",
    )
}

/// Aucun serveur SMTP n'est configuré pour envoyer ce message
pub(crate) fn smtp_not_configured() -> MiraError {
    MiraError::new(
        ErrorCategory::InvalidInput,
        invalid_input::PARAMETER,
        "aucun serveur SMTP n'est configuré pour ce compte",
    )
}
