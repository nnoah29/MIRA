//! Erreurs propres au transfert d'un fichier : connexion coupée, annulation.

use mira_vocabulary::error::codes::{cancelled, network};
use mira_vocabulary::{ErrorCategory, MiraError};

/// Erreur réseau : la connexion a été coupée ; le fichier partiel est gardé pour une reprise.
pub(super) fn connection_lost() -> MiraError {
    MiraError::new(
        ErrorCategory::Network,
        network::CONNECTION_LOST,
        "connexion interrompue pendant le téléchargement",
    )
}

/// Erreur d'annulation demandée par l'utilisateur.
pub(super) fn cancelled_error() -> MiraError {
    MiraError::new(
        ErrorCategory::Cancelled,
        cancelled::BY_USER,
        "téléchargement annulé",
    )
}
