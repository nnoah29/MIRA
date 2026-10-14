//! Routage du contrat : un `match` exhaustif par domaine, vers un handler par opération.

use mira_vocabulary::MiraError;

use super::contract::{Request, Response};
use super::engine::Engine;
use super::handlers::{
    account, action, auth, compose, engine, event, identity, operation, provider, queue,
};

/// Dirige une requête vers le routeur de son domaine
pub(crate) async fn route(engine_ref: &Engine, request: Request) -> Result<Response, MiraError> {
    match request {
        Request::Engine(request) => engine::route(engine_ref, request).await,
        Request::Provider(request) => provider::route(engine_ref, request).await,
        Request::Account(request) => account::route(engine_ref, request).await,
        Request::Identity(request) => identity::route(engine_ref, request).await,
        Request::Auth(request) => auth::route(engine_ref, request).await,
        Request::Operation(request) => operation::route(engine_ref, request).await,
        Request::Event(request) => event::route(engine_ref, request).await,
        Request::Action(request) => action::route(engine_ref, request).await,
        Request::Compose(request) => compose::route(engine_ref, request).await,
        Request::Queue(request) => queue::route(engine_ref, request).await,
    }
}
