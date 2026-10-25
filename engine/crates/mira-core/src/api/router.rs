//! Routage du contrat : un `match` exhaustif par domaine, vers un handler par opération.

use mira_vocabulary::MiraError;

use super::contract::{Request, Response};
use super::engine::Engine;
use super::handlers::{
    account, action, auth, compose, engine, event, identity, label_settings, local_label, mailbox,
    maintenance, message, operation, provider, queue, rule, search, settings, sync, template,
    thread,
};

/// Dirige une requête vers le routeur de son domaine
pub(crate) async fn route(engine_ref: &Engine, request: Request) -> Result<Response, MiraError> {
    match request {
        Request::Engine(request) => engine::route(engine_ref, request).await,
        Request::Provider(request) => provider::route(engine_ref, request).await,
        Request::Account(request) => account::route(engine_ref, request).await,
        Request::Identity(request) => identity::route(engine_ref, request).await,
        Request::Mailbox(request) => mailbox::route(engine_ref, request).await,
        Request::Message(request) => message::route(engine_ref, request).await,
        Request::Thread(request) => thread::route(engine_ref, request).await,
        Request::Search(request) => search::route(engine_ref, request).await,
        Request::Maintenance(request) => maintenance::route(engine_ref, request).await,
        Request::Auth(request) => auth::route(engine_ref, request).await,
        Request::Operation(request) => operation::route(engine_ref, request).await,
        Request::Event(request) => event::route(engine_ref, request).await,
        Request::Action(request) => action::route(engine_ref, request).await,
        Request::Compose(request) => compose::route(engine_ref, request).await,
        Request::Queue(request) => queue::route(engine_ref, request).await,
        Request::Sync(request) => sync::route(engine_ref, request).await,
        Request::Rule(request) => rule::route(engine_ref, request).await,
        Request::LocalLabel(request) => local_label::route(engine_ref, request).await,
        Request::LabelSettings(request) => label_settings::route(engine_ref, request).await,
        Request::Settings(request) => settings::route(engine_ref, request).await,
        Request::Template(request) => template::route(engine_ref, request).await,
    }
}
