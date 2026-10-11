//! Erreurs de chiffrement : clair refusé, certificat invalide, négociation échouée.

use std::io::Error;

use mira_vocabulary::error::codes::tls;
use mira_vocabulary::{ErrorCategory, MiraError, SuggestedAction};

/// Le clair est refusé hors de la boucle locale (AG-03).
pub(crate) fn plaintext_refused(host: &str) -> MiraError {
    MiraError::new(
        ErrorCategory::Tls,
        tls::PLAINTEXT_REFUSED,
        format!("connexion en clair refusée vers {host} (réservée à la boucle locale)"),
    )
    .with_action(SuggestedAction::CheckServerSettings)
}

/// Convertit un échec de négociation TLS en erreur de certificat ou de négociation.
pub(crate) fn from_handshake(error: &Error) -> MiraError {
    let invalid = error
        .get_ref()
        .and_then(|inner| inner.downcast_ref::<rustls::Error>())
        .is_some_and(|inner| matches!(inner, rustls::Error::InvalidCertificate(_)));
    if invalid {
        return MiraError::new(
            ErrorCategory::Tls,
            tls::INVALID_CERTIFICATE,
            "certificat du serveur invalide ou non approuvé",
        )
        .with_action(SuggestedAction::AcceptCertificate);
    }
    MiraError::new(
        ErrorCategory::Tls,
        tls::HANDSHAKE,
        "échec de la négociation TLS avec le serveur",
    )
    .with_action(SuggestedAction::CheckServerSettings)
}
