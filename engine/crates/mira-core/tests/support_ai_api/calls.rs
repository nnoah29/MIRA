//! Appels du domaine « Ai » par la façade et lecture de leurs réponses.

use mira_core::api::{AiRequest, AiResponse, Engine, Request, Response};
use mira_vocabulary::{AiConnection, AiProviderId};

/// Exécute une requête du domaine et rend la réponse du domaine, ou le code d'erreur
pub async fn call(engine: &Engine, request: AiRequest) -> Result<AiResponse, String> {
    match engine.execute(Request::Ai(request)).await {
        Ok(Response::Ai(answer)) => Ok(answer),
        Ok(other) => panic!("réponse inattendue : {other:?}"),
        Err(error) => Err(error.code),
    }
}

/// Exécute une requête dont la réponse est une connexion
pub async fn call_connection(engine: &Engine, request: AiRequest) -> Result<AiConnection, String> {
    match call(engine, request).await? {
        AiResponse::Connection(connection) => Ok(connection),
        other => panic!("réponse inattendue : {other:?}"),
    }
}

/// Ajoute une connexion de ce fournisseur avec cette adresse éventuelle
pub async fn add(
    engine: &Engine,
    provider: &str,
    label: &str,
    base_url: Option<&str>,
) -> Result<AiConnection, String> {
    let request = AiRequest::AddAiConnection {
        provider: AiProviderId(provider.to_string()),
        label: label.to_string(),
        base_url: base_url.map(str::to_string),
    };
    call_connection(engine, request).await
}

/// Connexions listées par le moteur
pub async fn list(engine: &Engine) -> Vec<AiConnection> {
    match call(engine, AiRequest::ListAiConnections).await.unwrap() {
        AiResponse::Connections(found) => found,
        other => panic!("réponse inattendue : {other:?}"),
    }
}
