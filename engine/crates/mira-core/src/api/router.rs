//! Routage du contrat : un `match` exhaustif par domaine, vers un handler par opération.

use mira_vocabulary::MiraError;

use super::contract::{Request, Response};
use super::engine::Engine;
use super::handlers::{engine, event, operation, provider};

/// Dirige une requête vers le routeur de son domaine
pub(crate) async fn route(engine_ref: &Engine, request: Request) -> Result<Response, MiraError> {
    match request {
        Request::Engine(request) => engine::route(engine_ref, request).await,
        Request::Provider(request) => provider::route(engine_ref, request).await,
        Request::Operation(request) => operation::route(engine_ref, request).await,
        Request::Event(request) => event::route(engine_ref, request).await,
    }
}
