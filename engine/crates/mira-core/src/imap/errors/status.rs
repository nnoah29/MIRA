//! Réponses `NO` et `BAD` converties en erreurs du contrat (jamais de secret).

use mira_vocabulary::error::codes::{authentication, server};
use mira_vocabulary::{ErrorCategory, ErrorCode, MiraError, SuggestedAction};

use crate::imap::protocol::response::{Response, Status, StatusKind};

/// Contexte qui oriente l'interprétation d'un refus du serveur.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Operation {
    Plain,
    Extension,
    PasswordLogin,
    TokenLogin,
}

/// Texte du serveur nettoyé des caractères de contrôle et borné.
fn clean_text(text: &str) -> String {
    text.chars().filter(|c| !c.is_control()).take(120).collect()
}

/// Erreur de catégorie serveur avec le texte (nettoyé) de la réponse.
fn server_error(code: ErrorCode, label: &str, status: &Status) -> MiraError {
    let detail = clean_text(&status.text);
    MiraError::new(ErrorCategory::Server, code, format!("{label} : {detail}"))
}

/// Erreur d'authentification, sans reprendre le texte du serveur.
fn login_error(token: bool) -> MiraError {
    if token {
        return MiraError::new(
            ErrorCategory::Authentication,
            authentication::TOKEN_EXPIRED,
            "jeton d'accès refusé par le serveur",
        )
        .retryable(true)
        .with_action(SuggestedAction::Reauthenticate);
    }
    MiraError::new(
        ErrorCategory::Authentication,
        authentication::REJECTED,
        "identifiants refusés par le serveur",
    )
    .with_action(SuggestedAction::Reauthenticate)
}

/// Traduit un statut `NO` ou `BAD` selon l'opération et le code de réponse.
pub(crate) fn status_error(status: &Status, operation: Operation) -> MiraError {
    let code = status.code.as_ref().map_or("", |code| code.name.as_str());
    match (status.kind, operation, code) {
        (_, Operation::PasswordLogin, _) => login_error(false),
        (_, Operation::TokenLogin, _) => login_error(true),
        (StatusKind::No, _, "OVERQUOTA") => {
            server_error(server::QUOTA_EXCEEDED, "quota dépassé", status)
        }
        (StatusKind::No, _, "TOOBIG") => {
            server_error(server::MESSAGE_TOO_LARGE, "message trop gros", status)
        }
        (StatusKind::No, _, "NONEXISTENT" | "TRYCREATE") => {
            server_error(server::MAILBOX_MISSING, "boîte absente", status)
        }
        (StatusKind::Bad, Operation::Extension, _) => server_error(
            server::EXTENSION_MISSING,
            "extension non prise en charge",
            status,
        ),
        (StatusKind::Bad, _, _) => server_error(server::PROTOCOL, "commande invalide", status),
        _ => server_error(server::REJECTED, "commande refusée", status),
    }
}

impl Response {
    /// Rend la réponse si elle est `OK`, sinon l'erreur correspondante.
    pub(crate) fn require_ok(self, operation: Operation) -> Result<Self, MiraError> {
        match self.status.kind {
            StatusKind::Ok => Ok(self),
            _ => Err(status_error(&self.status, operation)),
        }
    }
}
