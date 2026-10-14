//! Attente interrompue par un compte à réauthentifier : les actions restent en file, rien n'est perdu.

use mira_assembly::api::AccountStatus;
use mira_assembly::mira_vocabulary::error::codes::authentication;
use mira_assembly::mira_vocabulary::{
    AccountId, AccountState, ErrorCategory, MiraError, PendingAction, SuggestedAction,
};

/// Comptes dont l'état demande une nouvelle authentification.
pub fn accounts_needing_authentication(accounts: &[AccountStatus]) -> Vec<AccountId> {
    accounts
        .iter()
        .filter(|entry| entry.account.state == AccountState::AuthRequired)
        .map(|entry| entry.account.id.clone())
        .collect()
}

/// Dit si une action encore à jouer appartient à un compte à réauthentifier.
pub fn is_stuck_behind_authentication(unsettled: &[&PendingAction], blocked: &[AccountId]) -> bool {
    unsettled
        .iter()
        .any(|action| blocked.contains(&action.account))
}

/// Erreur rendue quand l'attente est vaine : l'utilisateur doit réauthentifier le compte.
pub fn authentication_required_error() -> MiraError {
    MiraError::new(
        ErrorCategory::Authentication,
        authentication::REJECTED,
        "le compte demande une nouvelle authentification ; l'action reste en file",
    )
    .with_action(SuggestedAction::Reauthenticate)
}
