#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Connexion d'IA par navigateur (OAuth PKCE) : de l'adresse d'autorisation à la clé au trousseau.

mod ai_models_support;
mod key_login_support;
mod support_ai_api;
mod support_api;

use key_login_support::browser::{query_of, redirect};
use key_login_support::fixture::{finish, start_login, started};
use mira_core::api::{AiRequest, AiResponse};
use mira_core::ports::SecretStore;
use mira_vocabulary::{AiSecretRef, ProfileId};
use support_ai_api::calls::call;

const KEY: &str = "sk-or-v1-CLE-SECRETE-4242";

/// L'adresse d'autorisation porte le défi S256, l'état, le libellé et une redirection en boucle locale
#[tokio::test]
async fn start_builds_the_authorization_url() {
    let f = started(200, "{}").await;
    let (_, url) = start_login(&f.fixture.engine, &f.connection).await.unwrap();
    let query = query_of(&url);
    assert!(url.starts_with("https://auth.provider.test/auth?"));
    assert_eq!(query["code_challenge_method"], "S256");
    assert_eq!(query["key_label"], "MIRA");
    assert!(query["callback_url"].starts_with("http://127.0.0.1:"));
    assert!(query["callback_url"].ends_with("/callback"));
    assert!(query["code_challenge"].len() >= 43 && !query["state"].is_empty());
}

/// Une connexion réussie range la clé au trousseau et rend `has_key`, sans jamais la renvoyer
#[tokio::test]
async fn complete_stores_the_key() {
    let f = started(200, &format!(r#"{{"key":"{KEY}"}}"#)).await;
    let (flow, url) = start_login(&f.fixture.engine, &f.connection).await.unwrap();
    let AiResponse::Connection(done) = finish(&f, &flow, &url, "CODE-1").await.unwrap() else {
        panic!("connexion attendue");
    };
    assert!(done.has_key);
    assert!(!serde_json::to_string(&done).unwrap().contains(KEY));
    let location = AiSecretRef {
        profile: ProfileId::from_raw("default"),
        connection: f.connection.id.clone(),
    };
    assert_eq!(
        f.fixture.secrets.get(&location).unwrap().unwrap().expose(),
        KEY
    );
}

/// L'échange présente le code, le vérificateur qui correspond au défi, et la méthode S256
#[tokio::test]
async fn exchange_body_matches_the_challenge() {
    let f = started(200, &format!(r#"{{"key":"{KEY}"}}"#)).await;
    let (flow, url) = start_login(&f.fixture.engine, &f.connection).await.unwrap();
    finish(&f, &flow, &url, "CODE-2").await.unwrap();
    let body: serde_json::Value = serde_json::from_str(&f.exchange.received()[0]).unwrap();
    assert_eq!(body["code"], "CODE-2");
    assert_eq!(body["code_challenge_method"], "S256");
    let verifier = body["code_verifier"].as_str().unwrap();
    assert_eq!(
        mira_core::auth::pkce::challenge_for(verifier),
        query_of(&url)["code_challenge"]
    );
}

/// Un `state` inattendu est refusé sans échange ni clé
#[tokio::test]
async fn wrong_state_stores_nothing() {
    let f = started(200, &format!(r#"{{"key":"{KEY}"}}"#)).await;
    let (flow, url) = start_login(&f.fixture.engine, &f.connection).await.unwrap();
    let callback = query_of(&url)["callback_url"].clone();
    let complete = call(&f.fixture.engine, AiRequest::CompleteAiKeyLogin { flow });
    let (status, result) = tokio::join!(redirect(&callback, "code=X&state=faux"), async {
        tokio::time::timeout(std::time::Duration::from_millis(500), complete).await
    });
    assert_eq!(status, 400);
    assert!(
        result.is_err(),
        "le flux doit rester en attente du bon état"
    );
    assert!(f.exchange.received().is_empty());
}

/// Le refus de l'utilisateur annule la connexion sans clé
#[tokio::test]
async fn user_denial_is_cancelled() {
    let f = started(200, "{}").await;
    let (flow, url) = start_login(&f.fixture.engine, &f.connection).await.unwrap();
    let query = query_of(&url);
    let denial = format!("error=access_denied&state={}", query["state"]);
    let complete = call(&f.fixture.engine, AiRequest::CompleteAiKeyLogin { flow });
    let (_, result) = tokio::join!(redirect(&query["callback_url"], &denial), complete);
    assert_eq!(result.unwrap_err(), "authentication.oauth_cancelled");
}
