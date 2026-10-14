//! Exemples des domaines comptes, identités et authentification.

use mira_core::api::{AccountRequest, AuthRequest, IdentityRequest};

pub const NAMES: &[&str] = &[
    "account.prepare_account",
    "account.test_connection",
    "account.add_account",
    "account.update_account",
    "account.pause_account",
    "account.resume_account",
    "account.delete_account",
    "account.list_accounts",
    "identity.list_identities",
    "identity.add_identity",
    "identity.update_identity",
    "identity.set_default_identity",
    "auth.start_oauth",
    "auth.complete_oauth",
    "auth.cancel_oauth",
    "auth.store_app_password",
    "auth.reauthenticate",
];

pub const EXAMPLES: &[&str] = &[
    r#"{"domain":"account","request":{"op":"prepare_account","address":"a@b.c","provider":null}}"#,
    r#"{"domain":"account","request":{"op":"test_connection","draft":{"address":"a@b.c","display_name":"n","provider":null,"servers":null,"auth":"password"},"credential":{"type":"password","value":"x"}}}"#,
    r#"{"domain":"account","request":{"op":"add_account","draft":{"address":"a@b.c","display_name":"n","provider":null,"servers":null,"auth":"password"},"credential":{"type":"password","value":"x"},"idempotency_key":"k"}}"#,
    r#"{"domain":"account","request":{"op":"update_account","account":"a","changes":{"display_name":null,"servers":null,"window_days":null,"synced_mailboxes":null}}}"#,
    r#"{"domain":"account","request":{"op":"pause_account","account":"a"}}"#,
    r#"{"domain":"account","request":{"op":"resume_account","account":"a"}}"#,
    r#"{"domain":"account","request":{"op":"delete_account","account":"a"}}"#,
    r#"{"domain":"account","request":{"op":"list_accounts"}}"#,
    r#"{"domain":"identity","request":{"op":"list_identities","account":"a"}}"#,
    r#"{"domain":"identity","request":{"op":"add_identity","account":"a","address":"a@b.c","display_name":"n","signature":null,"is_default":false}}"#,
    r#"{"domain":"identity","request":{"op":"update_identity","identity":"i","display_name":null,"signature":null}}"#,
    r#"{"domain":"identity","request":{"op":"set_default_identity","identity":"i"}}"#,
    r#"{"domain":"auth","request":{"op":"start_oauth","address":"a@b.c","provider":null}}"#,
    r#"{"domain":"auth","request":{"op":"complete_oauth","flow":"f"}}"#,
    r#"{"domain":"auth","request":{"op":"cancel_oauth","flow":"f"}}"#,
    r#"{"domain":"auth","request":{"op":"store_app_password","account":"a","password":"x"}}"#,
    r#"{"domain":"auth","request":{"op":"reauthenticate","account":"a"}}"#,
];

/// Nom `domaine.opération` d'une requête du domaine comptes
pub fn account_name(request: &AccountRequest) -> &'static str {
    match request {
        AccountRequest::PrepareAccount { .. } => "account.prepare_account",
        AccountRequest::TestConnection { .. } => "account.test_connection",
        AccountRequest::AddAccount { .. } => "account.add_account",
        AccountRequest::UpdateAccount { .. } => "account.update_account",
        AccountRequest::PauseAccount { .. } => "account.pause_account",
        AccountRequest::ResumeAccount { .. } => "account.resume_account",
        AccountRequest::DeleteAccount { .. } => "account.delete_account",
        AccountRequest::ListAccounts => "account.list_accounts",
    }
}

/// Nom `domaine.opération` d'une requête du domaine identités
pub fn identity_name(request: &IdentityRequest) -> &'static str {
    match request {
        IdentityRequest::ListIdentities { .. } => "identity.list_identities",
        IdentityRequest::AddIdentity(_) => "identity.add_identity",
        IdentityRequest::UpdateIdentity { .. } => "identity.update_identity",
        IdentityRequest::SetDefaultIdentity { .. } => "identity.set_default_identity",
    }
}

/// Nom `domaine.opération` d'une requête du domaine authentification
pub fn auth_name(request: &AuthRequest) -> &'static str {
    match request {
        AuthRequest::StartOAuth { .. } => "auth.start_oauth",
        AuthRequest::CompleteOAuth { .. } => "auth.complete_oauth",
        AuthRequest::CancelOAuth { .. } => "auth.cancel_oauth",
        AuthRequest::StoreAppPassword { .. } => "auth.store_app_password",
        AuthRequest::Reauthenticate { .. } => "auth.reauthenticate",
    }
}
