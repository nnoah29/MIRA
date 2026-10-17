//! Erreurs « introuvable » : compte, action, brouillon, identité.

use mira_vocabulary::error::codes::not_found;
use mira_vocabulary::{
    AccountId, ActionId, DraftId, ErrorCategory, ErrorContext, IdentityId, MiraError,
};

/// Erreur « compte inconnu »
pub(crate) fn account_not_found(id: &AccountId) -> MiraError {
    MiraError::new(
        ErrorCategory::NotFound,
        not_found::ACCOUNT,
        "compte inconnu",
    )
    .with_context(ErrorContext {
        account: Some(id.clone()),
        ..ErrorContext::default()
    })
}

/// Erreur « action inconnue »
pub(crate) fn action_not_found(id: &ActionId) -> MiraError {
    MiraError::new(
        ErrorCategory::NotFound,
        not_found::ACTION,
        "action inconnue",
    )
    .with_context(ErrorContext {
        action: Some(id.clone()),
        ..ErrorContext::default()
    })
}

/// Erreur « brouillon inconnu » (l'identifiant n'est pas un champ du contexte d'erreur)
pub(crate) fn draft_not_found(id: &DraftId) -> MiraError {
    MiraError::new(
        ErrorCategory::NotFound,
        not_found::DRAFT,
        format!("brouillon inconnu : {id}"),
    )
}

/// Erreur « aucune identité d'envoi pour ce compte »
pub(crate) fn no_identity_for_account(account: &AccountId) -> MiraError {
    MiraError::new(
        ErrorCategory::NotFound,
        not_found::IDENTITY,
        "aucune identité d'envoi pour ce compte",
    )
    .with_context(ErrorContext {
        account: Some(account.clone()),
        ..ErrorContext::default()
    })
}

/// Erreur « identité inconnue »
pub(crate) fn identity_not_found(id: &IdentityId) -> MiraError {
    MiraError::new(
        ErrorCategory::NotFound,
        not_found::IDENTITY,
        format!("identité inconnue : {id}"),
    )
}
