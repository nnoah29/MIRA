#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Connexion d'IA par navigateur qui rend une session : de l'adresse d'autorisation à la session au trousseau.

mod ai_models_support;
mod session_login_support;
mod session_support;
mod support_ai_api;
mod support_api;

use mira_core::api::AiResponse;
use mira_core::auth::session::AiSession;
use mira_core::ports::SecretStore;
use mira_vocabulary::{AiSecretRef, ProfileId};
use session_login_support::{SessionFixture, finish, start_login, started};
use session_support::browser::query_of;
use session_support::connect::grant;
use session_support::flow::ISSUER;

/// Met en file la réponse de jeton liée au nonce de l'adresse d'autorisation
fn reply_for(f: &SessionFixture, url: &str) {
    let nonce = query_of(url)["nonce"].clone();
    f.token.push(
        200,
        &grant(&nonce, "oaiapp_new", ISSUER, "openid scope.required"),
    );
}

/// Session rangée pour la connexion de test
fn stored(f: &SessionFixture) -> AiSession {
    let place = AiSecretRef {
        profile: ProfileId::from_raw("default"),
        connection: f.connection.id.clone(),
    };
    AiSession::from_secret(&f.fixture.secrets.get(&place).unwrap().unwrap()).unwrap()
}

/// Une connexion réussie range la session au trousseau, rend `has_key` et ne renvoie aucun jeton
#[tokio::test]
async fn session_login_stores_session_and_sets_has_key() {
    let f = started().await;
    let (flow, url) = start_login(&f.fixture.engine, &f.connection).await.unwrap();
    assert_eq!(query_of(&url)["client_id"], "dynamic_agent_client");
    reply_for(&f, &url);
    let AiResponse::Connection(done) = finish(&f, &flow, &url, "code=C1&client_id=oaiapp_new")
        .await
        .unwrap()
    else {
        panic!("connexion attendue");
    };
    assert!(done.has_key);
    assert!(!serde_json::to_string(&done).unwrap().contains("AT-1"));
    let session = stored(&f);
    assert_eq!(
        (session.client_id.as_str(), session.access_token.as_str()),
        ("oaiapp_new", "AT-1")
    );
    assert!(session.host_id.starts_with("urn:uuid:"));
}

/// La reconnexion reprend le client émis et le même identifiant d'hôte
#[tokio::test]
async fn reconnect_reuses_issued_client_id_and_host_id() {
    let f = started().await;
    let (flow, url) = start_login(&f.fixture.engine, &f.connection).await.unwrap();
    reply_for(&f, &url);
    finish(&f, &flow, &url, "code=C1&client_id=oaiapp_new")
        .await
        .unwrap();
    let first_host = stored(&f).host_id;
    let (flow, url) = start_login(&f.fixture.engine, &f.connection).await.unwrap();
    let query = query_of(&url);
    assert_eq!(query["client_id"], "oaiapp_new");
    assert_eq!(query["ext_agent_host_id"], first_host);
    reply_for(&f, &url);
    finish(&f, &flow, &url, "code=C2").await.unwrap();
    assert_eq!(stored(&f).host_id, first_host);
}
