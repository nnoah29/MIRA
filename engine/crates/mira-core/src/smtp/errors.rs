//! Conversion des erreurs de `lettre` en erreurs du contrat, sans secret ni texte du serveur.

use lettre::transport::smtp::Error as SmtpError;
use mira_vocabulary::error::codes::{network, server};
use mira_vocabulary::{ErrorCategory, MiraError};

use super::reply_codes::{fail, map_auth_failure, map_status, map_tls_failure};

/// Convertit une erreur de transport ; `xoauth` précise si le jeton d'accès était présenté
pub(super) fn map_error(error: &SmtpError, xoauth: bool) -> MiraError {
    if let Some(code) = error.status() {
        return map_status(u16::from(code), xoauth);
    }
    if error.is_tls() {
        return map_tls_failure();
    }
    if error.is_client() {
        return map_client_error(&error.to_string(), xoauth);
    }
    map_io_failure(error)
}

/// Erreur d'entrée-sortie : délai, hôte injoignable ou connexion coupée (issue incertaine)
fn map_io_failure(error: &SmtpError) -> MiraError {
    if error.is_timeout() {
        return fail(
            ErrorCategory::Network,
            network::TIMEOUT,
            "délai SMTP dépassé",
            false,
        );
    }
    if error.to_string().starts_with("Connection error") {
        let message = "serveur SMTP injoignable";
        return fail(ErrorCategory::Network, network::UNREACHABLE, message, true);
    }
    let message = "connexion SMTP perdue, l'issue de l'envoi est incertaine";
    fail(
        ErrorCategory::Network,
        network::CONNECTION_LOST,
        message,
        false,
    )
}

/// Erreur détectée par le client : TLS, extension, défi d'authentification refusé, sinon protocole
fn map_client_error(description: &str, xoauth: bool) -> MiraError {
    if description.contains("STARTTLS") || description.contains("not encrypted") {
        return map_tls_failure();
    }
    if description.contains("8BITMIME") || description.contains("SMTPUTF8") {
        let message = "extension SMTP absente chez le serveur";
        return fail(
            ErrorCategory::Server,
            server::EXTENSION_MISSING,
            message,
            false,
        );
    }
    if description.contains("challenge") {
        return map_auth_failure(xoauth);
    }
    let message = "dialogue SMTP inattendu";
    fail(ErrorCategory::Server, server::PROTOCOL, message, false)
}
