//! Correspondance entre réponses SMTP, échecs TLS ou d'authentification et codes du contrat.

use mira_vocabulary::error::codes::{authentication, server, tls};
use mira_vocabulary::{ErrorCategory, ErrorCode, MiraError, SuggestedAction};

/// Convertit un code de réponse SMTP (4xx transitoire, 5xx définitif)
pub(super) fn map_status(code: u16, xoauth: bool) -> MiraError {
    let message = format!("réponse SMTP {code}");
    match code {
        535 | 534 | 530 => map_auth_failure(xoauth),
        552 => fail(
            ErrorCategory::Server,
            server::MESSAGE_TOO_LARGE,
            &message,
            false,
        ),
        421 | 450 | 451 => fail(ErrorCategory::Server, server::RATE_LIMITED, &message, true),
        400..=499 => fail(ErrorCategory::Server, server::REJECTED, &message, true),
        _ => fail(ErrorCategory::Server, server::REJECTED, &message, false),
    }
}

/// Échec TLS : négociation impossible ou STARTTLS non proposé
pub(super) fn map_tls_failure() -> MiraError {
    let message = "négociation TLS SMTP échouée";
    fail(ErrorCategory::Tls, tls::HANDSHAKE, message, false)
}

/// Erreur d'authentification : jeton expiré pour XOAUTH2, identifiants refusés sinon
pub(super) fn map_auth_failure(xoauth: bool) -> MiraError {
    let code = if xoauth {
        authentication::TOKEN_EXPIRED
    } else {
        authentication::REJECTED
    };
    let message = "authentification SMTP refusée";
    fail(ErrorCategory::Authentication, code, message, false)
        .with_action(SuggestedAction::Reauthenticate)
}

/// Fabrique une erreur dont le message ne contient ni secret ni texte du serveur
pub(super) fn fail(
    category: ErrorCategory,
    code: ErrorCode,
    message: &str,
    retryable: bool,
) -> MiraError {
    MiraError::new(category, code, message).retryable(retryable)
}
