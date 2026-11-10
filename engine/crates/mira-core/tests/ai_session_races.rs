#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Session d'abonné : se déconnecter ou supprimer la connexion pendant un rafraîchissement ou une attente ne laisse aucun secret.

mod ai_chat_support;
mod ai_models_support;
mod session_chat_support;
mod session_login_support;
mod session_support;
mod support_ai_api;
mod support_api;

use std::time::Duration;

use ai_chat_support::{chat_start, collect};
use ai_models_support::ModelsFixture;
use mira_core::api::AiRequest;
use mira_core::ports::SecretStore;
use mira_vocabulary::{AiConnectionId, AiSecretRef, ProfileId};
use session_chat_support::{NOW, provider, with_session};
use session_support::browser::query_of;
use session_support::connect::grant;
use session_support::flow::ISSUER;
use session_support::token_server::TokenServer;
use support_ai_api::calls::call;

/// Emplacement du secret de `conn-1`
fn place() -> AiSecretRef {
    AiSecretRef {
        profile: ProfileId::from_raw("default"),
        connection: AiConnectionId::from_raw("conn-1"),
    }
}

/// Se déconnecter pendant un rafraîchissement en vol attend la fin de celui-ci puis efface : la session ne ressuscite pas
#[tokio::test]
async fn disconnect_during_refresh_does_not_resurrect_the_session() {
    let token = TokenServer::start().await;
    token.push(
        200,
        r#"{"access_token":"AT-NEW","refresh_token":"RT-NEW","expires_in":3600}"#,
    );
    token.set_delay(400);
    let fixture = ModelsFixture::start(provider(&token)).await;
    let connection = with_session(&fixture, NOW - 10);
    fixture
        .server
        .queue(session_chat_support::responses::responses_stream());
    let engine = &fixture.engine;
    let chatting = async {
        let mut stream = engine
            .chat_stream(chat_start(&connection, "Salut"))
            .await
            .unwrap();
        collect(&mut stream).await;
    };
    let clearing = async {
        tokio::time::sleep(Duration::from_millis(150)).await;
        let clear = AiRequest::ClearAiApiKey {
            connection: connection.id.clone(),
        };
        call(engine, clear).await.unwrap();
    };
    tokio::join!(chatting, clearing);
    assert!(fixture.secrets.get(&place()).unwrap().is_none());
}

/// Le navigateur rappelle la redirection, même si plus rien n'écoute
async fn redirect_back(query: &std::collections::HashMap<String, String>) {
    let client = reqwest::Client::builder().no_proxy().build().unwrap();
    let state = &query["state"];
    let back = format!(
        "{}?state={state}&code=C1&client_id=oaiapp_new",
        query["redirect_uri"]
    );
    let _ = client.get(back).send().await;
}

/// Supprime la connexion, puis le navigateur rappelle la redirection
async fn remove_then_redirect(
    f: &session_login_support::SessionFixture,
    query: &std::collections::HashMap<String, String>,
) {
    let removal = AiRequest::RemoveAiConnection {
        connection: f.connection.id.clone(),
    };
    tokio::time::sleep(Duration::from_millis(100)).await;
    call(&f.fixture.engine, removal).await.unwrap();
    redirect_back(query).await;
}

/// Supprimer la connexion pendant que le navigateur attend annule le flux : aucune session n'est rangée pour une connexion disparue
#[tokio::test]
async fn removing_the_connection_during_login_stores_nothing() {
    let f = session_login_support::started().await;
    let started = session_login_support::start_login(&f.fixture.engine, &f.connection).await;
    let (flow, url) = started.unwrap();
    let query = query_of(&url);
    let reply = grant(
        &query["nonce"],
        "oaiapp_new",
        ISSUER,
        "openid scope.required",
    );
    f.token.push(200, &reply);
    let waiting = call(&f.fixture.engine, AiRequest::CompleteAiKeyLogin { flow });
    let (waited, ()) = tokio::join!(waiting, remove_then_redirect(&f, &query));
    assert!(waited.is_err());
    let location = AiSecretRef {
        profile: ProfileId::from_raw("default"),
        connection: f.connection.id.clone(),
    };
    assert!(f.fixture.secrets.get(&location).unwrap().is_none());
}
