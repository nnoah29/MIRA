//! Erreurs « état incompatible » : boîte système, libellés multiples, action ou brouillon figés.

use mira_vocabulary::error::codes::incompatible_state;
use mira_vocabulary::{ActionId, ErrorCategory, ErrorContext, MailboxId, MiraError};

/// Opération refusée sur une boîte système
pub(crate) fn system_mailbox(id: &MailboxId) -> MiraError {
    MiraError::new(
        ErrorCategory::IncompatibleState,
        incompatible_state::SYSTEM_MAILBOX,
        "une boîte système ne se renomme ni ne se supprime",
    )
    .with_context(ErrorContext {
        mailbox: Some(id.clone()),
        ..ErrorContext::default()
    })
}

/// D-17 : second libellé refusé sur un compte à dossiers
pub(crate) fn multiple_labels_unsupported() -> MiraError {
    MiraError::new(
        ErrorCategory::IncompatibleState,
        incompatible_state::MULTIPLE_LABELS_UNSUPPORTED,
        "ce compte range un message dans un seul dossier : déplacez-le plutôt que d'ajouter un libellé",
    )
}

/// L'action n'est plus annulable
pub(crate) fn action_not_cancellable(id: &ActionId) -> MiraError {
    MiraError::new(
        ErrorCategory::IncompatibleState,
        incompatible_state::ACTION_NOT_CANCELLABLE,
        "l'action n'est plus en attente : elle ne peut plus être annulée",
    )
    .with_context(ErrorContext {
        action: Some(id.clone()),
        ..ErrorContext::default()
    })
}

/// L'action n'est pas échouée : elle ne peut pas être relancée
pub(crate) fn action_not_retryable(id: &ActionId) -> MiraError {
    MiraError::new(
        ErrorCategory::IncompatibleState,
        incompatible_state::ACTION_NOT_RETRYABLE,
        "seule une action échouée peut être relancée",
    )
    .with_context(ErrorContext {
        action: Some(id.clone()),
        ..ErrorContext::default()
    })
}

/// Aucun envoi en attente à annuler pour ce brouillon
pub(crate) fn no_send_to_cancel() -> MiraError {
    MiraError::new(
        ErrorCategory::IncompatibleState,
        incompatible_state::ACTION_NOT_CANCELLABLE,
        "aucun envoi en attente à annuler pour ce brouillon",
    )
}

/// Le brouillon n'est plus en cours d'édition
pub(crate) fn draft_not_editable() -> MiraError {
    MiraError::new(
        ErrorCategory::IncompatibleState,
        incompatible_state::DRAFT_NOT_EDITABLE,
        "le brouillon est planifié ou en cours d'envoi : annulez l'envoi avant de le modifier",
    )
}
