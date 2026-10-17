//! Noms des variantes des domaines comptes, identités et authentification (`match` sans `_`).

use mira_assembly::api::{AccountRequest, AuthRequest, IdentityRequest};

/// Nom d'une requête du domaine comptes.
pub fn account_name(request: &AccountRequest) -> &'static str {
    match request {
        AccountRequest::PrepareAccount { .. } => "account.prepare_account",
        AccountRequest::TestConnection { .. } => "account.test_connection",
        AccountRequest::AddAccount { .. } => "account.add_account",
        AccountRequest::ListAccounts => "account.list_accounts",
        AccountRequest::UpdateAccount { .. } => "account.update_account",
        AccountRequest::PauseAccount { .. } => "account.pause_account",
        AccountRequest::ResumeAccount { .. } => "account.resume_account",
        AccountRequest::DeleteAccount { .. } => "account.delete_account",
    }
}

/// Nom d'une requête du domaine identités.
pub fn identity_name(request: &IdentityRequest) -> &'static str {
    match request {
        IdentityRequest::ListIdentities { .. } => "identity.list_identities",
        IdentityRequest::AddIdentity(_) => "identity.add_identity",
        IdentityRequest::UpdateIdentity { .. } => "identity.update_identity",
        IdentityRequest::SetDefaultIdentity { .. } => "identity.set_default_identity",
    }
}

/// Nom d'une requête du domaine authentification.
pub fn auth_name(request: &AuthRequest) -> &'static str {
    match request {
        AuthRequest::StartOAuth { .. } => "auth.start_oauth",
        AuthRequest::CompleteOAuth { .. } => "auth.complete_oauth",
        AuthRequest::CancelOAuth { .. } => "auth.cancel_oauth",
        AuthRequest::StoreAppPassword { .. } => "auth.store_app_password",
        AuthRequest::Reauthenticate { .. } => "auth.reauthenticate",
    }
}
