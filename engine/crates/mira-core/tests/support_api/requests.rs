//! Requêtes du contrat construites pour les tests, et accès au résultat attendu.

use mira_core::api::contract::AccountPreparation;
use mira_core::api::{
    AccountDraft, AccountRequest, CredentialInput, Engine, Request, Response, SecretInput,
};
use mira_vocabulary::{Account, AuthMethodKind, IdempotencyKey, ProviderId, ServerSettings};

/// Préparation d'un compte : provider retenu et serveurs proposés
pub async fn prepare(engine: &Engine, address: &str) -> AccountPreparation {
    let request = Request::Account(AccountRequest::PrepareAccount {
        address: address.to_string(),
        provider: None,
    });
    match engine.execute(request).await.unwrap() {
        Response::AccountPreparation(preparation) => preparation,
        other => panic!("réponse inattendue : {other:?}"),
    }
}

/// Brouillon de compte pour une adresse et une méthode d'authentification
pub fn draft(address: &str, auth: AuthMethodKind) -> AccountDraft {
    AccountDraft {
        address: address.to_string(),
        display_name: "Test".to_string(),
        provider: None,
        servers: None,
        auth,
    }
}

/// Brouillon « Autre » avec ses serveurs saisis à la main
pub fn manual_draft(address: &str, servers: ServerSettings, provider: &str) -> AccountDraft {
    AccountDraft {
        provider: Some(ProviderId::new(provider)),
        servers: Some(servers),
        ..draft(address, AuthMethodKind::Password)
    }
}

/// Requête d'ajout de compte avec un mot de passe ou un mot de passe d'application
pub fn add_request(draft: AccountDraft, secret: &str, key: &str) -> Request {
    let input = SecretInput::new(secret);
    let credential = match draft.auth {
        AuthMethodKind::AppPassword => CredentialInput::AppPassword(input),
        _ => CredentialInput::Password(input),
    };
    Request::Account(AccountRequest::AddAccount {
        draft,
        credential,
        idempotency_key: IdempotencyKey(key.to_string()),
    })
}

/// Exécute une requête d'ajout et rend le compte créé
pub async fn add_account(engine: &Engine, draft: AccountDraft, secret: &str, key: &str) -> Account {
    match engine
        .execute(add_request(draft, secret, key))
        .await
        .unwrap()
    {
        Response::Account(account) => account,
        other => panic!("réponse inattendue : {other:?}"),
    }
}

/// Exécute une requête de compte et rend le compte de la réponse
pub async fn account_response(engine: &Engine, request: AccountRequest) -> Account {
    match engine.execute(Request::Account(request)).await.unwrap() {
        Response::Account(account) => account,
        other => panic!("réponse inattendue : {other:?}"),
    }
}
