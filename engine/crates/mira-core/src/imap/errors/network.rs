//! Erreurs de connexion : injoignable, coupée, délai dépassé.

use std::io::{Error, ErrorKind};

use mira_vocabulary::error::codes::network;
use mira_vocabulary::{ErrorCategory, MiraError, SuggestedAction};

/// Le serveur n'a pas pu être joint.
pub(crate) fn unreachable(host: &str, port: u16) -> MiraError {
    MiraError::new(
        ErrorCategory::Network,
        network::UNREACHABLE,
        format!("serveur injoignable : {host}:{port}"),
    )
    .with_action(SuggestedAction::CheckServerSettings)
}

/// La connexion a été coupée en cours d'échange.
pub(crate) fn connection_lost() -> MiraError {
    MiraError::new(
        ErrorCategory::Network,
        network::CONNECTION_LOST,
        "connexion au serveur interrompue",
    )
    .with_action(SuggestedAction::RetryLater)
}

/// Le serveur n'a pas répondu dans le délai prévu.
pub(crate) fn timeout() -> MiraError {
    MiraError::new(
        ErrorCategory::Network,
        network::TIMEOUT,
        "le serveur n'a pas répondu à temps",
    )
    .with_action(SuggestedAction::RetryLater)
}

/// Convertit une erreur d'entrée-sortie d'une connexion établie.
pub(crate) fn from_io(error: &Error) -> MiraError {
    match error.kind() {
        ErrorKind::TimedOut => timeout(),
        _ => connection_lost(),
    }
}
