#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Connexion d'IA par navigateur : échecs de l'échange, annulation, flux inconnu, fournisseur sans connexion.

mod ai_models_support;
mod key_login_support;
mod support_ai_api;
mod support_api;

use key_login_support::fixture::{PLAIN_PROVIDER, finish, start_login, started};
use mira_core::api::AiRequest;
use mira_core::ports::{SecretStore, SecretValue};
use mira_vocabulary::{AiSecretRef, ProfileId};
use support_ai_api::calls::{add, call};

const OLD_KEY: &str = "sk-or-ANCIENNE-CLE-1111";

/// Emplacement de la clé de la connexion dans le trousseau
fn location(id: &mira_vocabulary::AiConnectionId) -> AiSecretRef {
    AiSecretRef {
        profile: ProfileId::from_raw("default"),
        connection: id.clone(),
    }
}

/// Un échange refusé (403) échoue sans toucher la clé déjà rangée
#[tokio::test]
async fn rejected_exchange_keeps_the_previous_key() {
    let f = started(403, r#"{"error":"Invalid code or code_verifier"}"#).await;
    let place = location(&f.connection.id);
    f.fixture
        .secrets
        .put(&place, &SecretValue::new(OLD_KEY))
        .unwrap();
    let (flow, url) = start_login(&f.fixture.engine, &f.connection).await.unwrap();
    let error = finish(&f, &flow, &url, "C").await.unwrap_err();
    assert_eq!(error, "authentication.rejected");
    assert_eq!(
        f.fixture.secrets.get(&place).unwrap().unwrap().expose(),
        OLD_KEY
    );
}

/// Une réponse sans clé, vide ou illisible est une réponse de serveur invalide
#[tokio::test]
async fn bad_answers_are_protocol_errors() {
    for body in [r#"{"ok":true}"#, r#"{"key":""}"#, "pas du json"] {
        let f = started(200, body).await;
        let (flow, url) = start_login(&f.fixture.engine, &f.connection).await.unwrap();
        let error = finish(&f, &flow, &url, "C").await.unwrap_err();
        assert_eq!(error, "server.protocol", "{body}");
        assert!(
            f.fixture
                .secrets
                .get(&location(&f.connection.id))
                .unwrap()
                .is_none()
        );
    }
}

/// Annuler avant la redirection interrompt l'attente ; un flux annulé ou inconnu ne se termine plus
#[tokio::test]
async fn cancel_interrupts_and_unknown_flows_are_refused() {
    let f = started(200, "{}").await;
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
    let again = call(engine, AiRequest::CompleteAiKeyLogin { flow }).await;
    assert_eq!(again.unwrap_err(), "invalid_input.parameter");
    call(
        engine,
        AiRequest::CancelAiKeyLogin {
            flow: "inconnu".into(),
        },
    )
    .await
    .unwrap();
}

/// Un fournisseur sans connexion par navigateur refuse le démarrage
#[tokio::test]
async fn provider_without_key_login_is_refused() {
    let f = started(200, "{}").await;
    let address = f.fixture.server.base_url();
    let plain = add(&f.fixture.engine, PLAIN_PROVIDER, "Autre", Some(&address))
        .await
        .unwrap();
    let error = start_login(&f.fixture.engine, &plain).await.unwrap_err();
    assert_eq!(error, "incompatible_state.ai_key_login_unsupported");
}

/// Le vérificateur et la clé n'apparaissent ni dans les erreurs ni dans le `Debug` du moteur
#[tokio::test]
async fn no_secret_leaks_in_errors_or_debug() {
    let f = started(403, r#"{"key":"sk-or-FUITE-9"}"#).await;
    let (flow, url) = start_login(&f.fixture.engine, &f.connection).await.unwrap();
    let error = finish(&f, &flow, &url, "CODE-SECRET-7").await.unwrap_err();
    let body: serde_json::Value = serde_json::from_str(&f.exchange.received()[0]).unwrap();
    let verifier = body["code_verifier"].as_str().unwrap();
    let seen = format!("{error}{:?}", f.fixture.engine);
    for secret in [verifier, "CODE-SECRET-7", "sk-or-FUITE-9"] {
        assert!(!seen.contains(secret), "fuite de {secret}");
    }
}
