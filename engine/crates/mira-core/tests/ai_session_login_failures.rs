#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Connexion d'IA par navigateur qui rend une session : refus, échec de l'échange, annulation, fournisseur sans connexion.

mod ai_models_support;
mod session_login_support;
mod session_support;
mod support_ai_api;
mod support_api;

use mira_core::api::AiRequest;
use mira_core::ports::{SecretStore, SecretValue};
use mira_vocabulary::{AiConnectionId, AiSecretRef, ProfileId};
use session_login_support::{PLAIN_PROVIDER, finish, start_login, started};
use session_support::browser::query_of;
use session_support::connect::grant;
use session_support::flow::ISSUER;
use support_ai_api::calls::{add, call};

/// Emplacement du secret de la connexion
fn location(id: &AiConnectionId) -> AiSecretRef {
    AiSecretRef {
        profile: ProfileId::from_raw("default"),
        connection: id.clone(),
    }
}

/// Une redirection d'inscription sans client émis échoue et ne range rien
#[tokio::test]
async fn callback_without_client_id_fails_and_stores_nothing() {
    let f = started().await;
    let (flow, url) = start_login(&f.fixture.engine, &f.connection).await.unwrap();
    let error = finish(&f, &flow, &url, "code=C1").await.unwrap_err();
    assert_eq!(error, "authentication.rejected");
    assert!(
        f.fixture
            .secrets
            .get(&location(&f.connection.id))
            .unwrap()
            .is_none()
    );
    assert!(f.token.received().is_empty());
}

/// Le refus de l'utilisateur est une annulation
#[tokio::test]
async fn access_denied_is_cancelled() {
    let f = started().await;
    let (flow, url) = start_login(&f.fixture.engine, &f.connection).await.unwrap();
    let error = finish(&f, &flow, &url, "error=access_denied")
        .await
        .unwrap_err();
    assert_eq!(error, "authentication.oauth_cancelled");
}

/// Un échange refusé échoue sans toucher la session déjà rangée
#[tokio::test]
async fn rejected_exchange_keeps_the_previous_secret() {
    let f = started().await;
    let place = location(&f.connection.id);
    f.fixture
        .secrets
        .put(&place, &SecretValue::new("ANCIEN-SECRET"))
        .unwrap();
    f.token.push(403, r#"{"error":"invalid_grant"}"#);
    let (flow, url) = start_login(&f.fixture.engine, &f.connection).await.unwrap();
    let error = finish(&f, &flow, &url, "code=C1&client_id=oaiapp_new")
        .await
        .unwrap_err();
    assert!(error.starts_with("authentication."), "{error}");
    assert_eq!(
        f.fixture.secrets.get(&place).unwrap().unwrap().expose(),
        "ANCIEN-SECRET"
    );
}

/// Annuler avant la redirection interrompt l'attente ; un fournisseur sans connexion est refusé
#[tokio::test]
async fn cancel_interrupts_and_plain_provider_is_refused() {
    let f = started().await;
    let (flow, _) = start_login(&f.fixture.engine, &f.connection).await.unwrap();
    let engine = &f.fixture.engine;
    let waiting = call(engine, AiRequest::CompleteAiKeyLogin { flow: flow.clone() });
    let cancelling = async {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        call(engine, AiRequest::CancelAiKeyLogin { flow: flow.clone() }).await
    };
    let (waited, cancelled) = tokio::join!(waiting, cancelling);
    assert_eq!(waited.unwrap_err(), "authentication.oauth_cancelled");
    cancelled.unwrap();
    let address = f.fixture.server.base_url();
    let plain = add(engine, PLAIN_PROVIDER, "Autre", Some(&address))
        .await
        .unwrap();
    let error = start_login(engine, &plain).await.unwrap_err();
    assert_eq!(error, "incompatible_state.ai_key_login_unsupported");
}

/// Ni jeton, ni code, ni vérificateur dans les erreurs et le `Debug` du moteur
#[tokio::test]
async fn no_secret_in_errors_or_debug() {
    let f = started().await;
    f.token.push(403, r#"{"access_token":"AT-FUITE-9"}"#);
    let (flow, url) = start_login(&f.fixture.engine, &f.connection).await.unwrap();
    let nonce = query_of(&url)["nonce"].clone();
    let _ = grant(&nonce, "oaiapp_new", ISSUER, "x");
    let error = finish(&f, &flow, &url, "code=CODE-SECRET-7&client_id=oaiapp_new")
        .await
        .unwrap_err();
    let form = f.token.received().remove(0);
    let verifier = form
        .split('&')
        .find_map(|p| p.strip_prefix("code_verifier="))
        .unwrap();
    let seen = format!("{error}{:?}", f.fixture.engine);
    for secret in [verifier, "CODE-SECRET-7", "AT-FUITE-9"] {
        assert!(!seen.contains(secret), "fuite de {secret}");
    }
}
