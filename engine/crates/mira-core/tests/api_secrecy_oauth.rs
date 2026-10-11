#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Invariant 15 pour OAuth : jetons et code d'autorisation absents des sorties, événements et base.

mod support_api;

use mira_core::api::{CredentialInput, Response, SecretInput};
use mira_vocabulary::SecretKind;
use support_api::files::files_containing;
use support_api::fixture::{self, Fixture};
use support_api::oauth::{OAUTH_USER, authorize, start_oauth};
use support_api::oauth_requests::add_oauth_request;
use support_api::queries::{accounts, event_kinds};

/// Joue S1 et rend les sorties JSON vues par un client, avec les jetons rangés
async fn s1_output(f: &Fixture) -> (String, Vec<String>) {
    let started = start_oauth(&f.engine, OAUTH_USER).await;
    let mut seen = serde_json::to_string(&started).unwrap();
    let flow = authorize(f, OAUTH_USER).await;
    let added = f
        .engine
        .execute(add_oauth_request(&flow, "k"))
        .await
        .unwrap();
    seen.push_str(&serde_json::to_string(&added).unwrap());
    let Response::Account(account) = added else {
        panic!("compte attendu");
    };
    seen.push_str(&serde_json::to_string(&accounts(&f.engine).await).unwrap());
    seen.push_str(&serde_json::to_string(&event_kinds(&f.engine).await).unwrap());
    let kinds = [SecretKind::AccessToken, SecretKind::RefreshToken];
    let tokens = kinds.map(|kind| f.stored_secret(&account.id, kind).unwrap());
    (seen, tokens.to_vec())
}

/// Jetons d'accès et de rafraîchissement : introuvables dans les réponses, les événements et la base
#[tokio::test]
async fn invariant_15_no_oauth_token_in_responses_events_or_database() {
    let f = fixture::start().await;
    let (seen, tokens) = s1_output(&f).await;
    assert_eq!(tokens.len(), 2);
    for token in &tokens {
        assert!(
            !seen.contains(token.as_str()),
            "jeton visible dans les sorties"
        );
    }
    let Fixture { home, engine, .. } = f;
    engine.shutdown().unwrap();
    for token in &tokens {
        assert!(files_containing(home.path(), token).is_empty());
    }
}

/// La forme JSON du secret d'un compte est stable et son affichage de débogage est masqué
#[test]
fn credential_input_json_shape_is_stable() {
    let flow = CredentialInput::OAuthFlow {
        flow: "f1".to_string(),
    };
    let json = serde_json::to_value(&flow).unwrap();
    assert_eq!(
        json,
        serde_json::json!({"type": "oauth_flow", "value": {"flow": "f1"}})
    );
    let start = mira_core::api::AuthRequest::StartOAuth {
        address: "a@b.test".to_string(),
        provider: None,
    };
    assert_eq!(serde_json::to_value(&start).unwrap()["op"], "start_oauth");
    let password = CredentialInput::Password(SecretInput::new("sekret-123"));
    let json = serde_json::to_value(&password).unwrap();
    assert_eq!(
        json,
        serde_json::json!({"type": "password", "value": "sekret-123"})
    );
    assert!(!format!("{password:?}").contains("sekret-123"));
    let back: CredentialInput = serde_json::from_value(json).unwrap();
    assert_eq!(back, password);
}
