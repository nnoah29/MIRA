#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, dead_code)]
//! Moteur prêt pour une connexion de session : fournisseur de test dont le point de jeton est le faux serveur.

use mira_core::api::{AiRequest, AiResponse, Engine};
use mira_vocabulary::{AiConnection, AiSessionLogin};

use crate::ai_models_support::{ModelsFixture, TestProvider};
use crate::session_support::browser::query_of;
use crate::session_support::flow::login;
use crate::session_support::token_server::TokenServer;
use crate::support_ai_api::calls::{add, call};

pub const SESSION_PROVIDER: &str = "session-provider";
pub const PLAIN_PROVIDER: &str = "plain-provider";

/// Moteur, faux point de jeton et connexion créée pour le fournisseur à session
pub struct SessionFixture {
    pub fixture: ModelsFixture,
    pub token: TokenServer,
    pub connection: AiConnection,
}

/// Démarre le tout ; le point de jeton répond ce que le test met en file
pub async fn started() -> SessionFixture {
    let token = TokenServer::start().await;
    let mut with_login = TestProvider::new(SESSION_PROVIDER);
    let session: AiSessionLogin = login(&token.url);
    with_login.session_login = Some(session);
    let fixture =
        ModelsFixture::start_many(vec![with_login, TestProvider::new(PLAIN_PROVIDER)]).await;
    let address = fixture.server.base_url();
    let connection = add(&fixture.engine, SESSION_PROVIDER, "Essai", Some(&address))
        .await
        .unwrap();
    SessionFixture {
        fixture,
        token,
        connection,
    }
}

/// Démarre la connexion et rend le flux et l'adresse d'autorisation
pub async fn start_login(
    engine: &Engine,
    connection: &AiConnection,
) -> Result<(String, String), String> {
    let request = AiRequest::StartAiKeyLogin {
        connection: connection.id.clone(),
    };
    match call(engine, request).await? {
        AiResponse::KeyLoginStarted {
            flow,
            authorize_url,
        } => Ok((flow, authorize_url)),
        other => panic!("réponse inattendue : {other:?}"),
    }
}

/// Termine le flux pendant que le navigateur simulé rappelle la redirection avec `extra` (le `state` est ajouté)
pub async fn finish(
    f: &SessionFixture,
    flow: &str,
    url: &str,
    extra: &str,
) -> Result<AiResponse, String> {
    let query = query_of(url);
    let request = AiRequest::CompleteAiKeyLogin {
        flow: flow.to_string(),
    };
    let complete = call(&f.fixture.engine, request);
    let uri = query["redirect_uri"].clone();
    let rest = format!("state={}&{extra}", query["state"]);
    let (_, result) = tokio::join!(
        crate::session_support::browser::redirect(&uri, &rest),
        complete
    );
    result
}
