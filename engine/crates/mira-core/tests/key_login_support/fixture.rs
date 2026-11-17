//! Moteur prêt pour une connexion par navigateur : fournisseur de test dont le point d'échange est le faux serveur.

use mira_core::api::{AiRequest, AiResponse, Engine};
use mira_vocabulary::{AiConnection, AiKeyLogin};

use super::browser::approve;
use super::exchange_server::ExchangeServer;
use crate::ai_models_support::{ModelsFixture, TestProvider};
use crate::support_ai_api::calls::{add, call};

pub const LOGIN_PROVIDER: &str = "login-provider";
pub const PLAIN_PROVIDER: &str = "plain-provider";

/// Moteur, faux point d'échange et connexion créée pour le fournisseur à connexion par navigateur
pub struct LoginFixture {
    pub fixture: ModelsFixture,
    pub exchange: ExchangeServer,
    pub connection: AiConnection,
}

/// Démarre le tout ; le point d'échange répond `status` avec `body`
pub async fn started(status: u16, body: &str) -> LoginFixture {
    let exchange = ExchangeServer::start(status, body).await;
    let mut with_login = TestProvider::new(LOGIN_PROVIDER);
    with_login.key_login = Some(AiKeyLogin {
        authorize_url: "https://auth.provider.test/auth".to_string(),
        exchange_url: exchange.url.clone(),
    });
    let fixture =
        ModelsFixture::start_many(vec![with_login, TestProvider::new(PLAIN_PROVIDER)]).await;
    let address = fixture.server.base_url();
    let connection = add(&fixture.engine, LOGIN_PROVIDER, "Essai", Some(&address))
        .await
        .unwrap();
    LoginFixture {
        fixture,
        exchange,
        connection,
    }
}

/// Démarre la connexion par navigateur et rend le flux et l'adresse d'autorisation
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

/// Termine le flux pendant que le navigateur simulé approuve avec ce code
pub async fn finish(
    f: &LoginFixture,
    flow: &str,
    url: &str,
    code: &str,
) -> Result<AiResponse, String> {
    let request = AiRequest::CompleteAiKeyLogin {
        flow: flow.to_string(),
    };
    let complete = call(&f.fixture.engine, request);
    let (_, result) = tokio::join!(approve(url, code), complete);
    result
}
