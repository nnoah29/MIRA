//! Requêtes OAuth construites pour les tests.

use mira_core::api::{AccountDraft, AccountRequest, AuthRequest, CredentialInput, Request};
use mira_vocabulary::{AuthMethodKind, IdempotencyKey};

use super::oauth::OAUTH_USER;
use super::requests::draft;

/// Brouillon d'un compte OAuth2 pour l'utilisateur de test
pub fn oauth_draft() -> AccountDraft {
    draft(OAUTH_USER, AuthMethodKind::OAuth2)
}

/// Requête d'ajout d'un compte OAuth2 à partir d'un flux terminé
pub fn add_oauth_request(flow: &str, key: &str) -> Request {
    Request::Account(AccountRequest::AddAccount {
        draft: oauth_draft(),
        credential: CredentialInput::OAuthFlow {
            flow: flow.to_string(),
        },
        idempotency_key: IdempotencyKey(key.to_string()),
    })
}

/// Requête de fin de flux OAuth
pub fn complete_request(flow: &str) -> Request {
    Request::Auth(AuthRequest::CompleteOAuth {
        flow: flow.to_string(),
    })
}
