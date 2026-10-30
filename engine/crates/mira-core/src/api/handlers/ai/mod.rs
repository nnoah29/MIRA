//! Domaine « assistant d'IA » : un fichier par opération, clés au trousseau, consentement par destination.

mod add_connection;
mod clear_key;
mod connection_view;
mod consent;
mod list_connections;
mod list_models;
mod list_providers;
mod remove_connection;
mod set_default;
mod set_key;
mod test_connection;
mod update_connection;
mod validate_key;
mod validate_label;
mod validate_url;

use mira_vocabulary::MiraError;

use crate::api::contract::{AiRequest, AiRequest as R, Response};
use crate::api::engine::Engine;
use add_connection::handle_add_ai_connection as add;
use clear_key::handle_clear_ai_api_key as clear_key;
use consent::handle_acknowledge_ai_consent as acknowledge;
use list_connections::handle_list_ai_connections as connections;
use list_models::handle_list_ai_models as models;
use list_providers::handle_list_ai_providers as providers;
use remove_connection::handle_remove_ai_connection as remove;
use set_default::handle_set_default_ai_connection as make_default;
use set_key::handle_set_ai_api_key as set_key;
use test_connection::handle_test_ai_connection as test;
use update_connection::handle_update_ai_connection as update;

/// Résultat d'une requête du domaine
type Answer = Result<Response, MiraError>;

/// Route une requête du domaine assistant d'IA
pub(crate) async fn route(engine: &Engine, request: AiRequest) -> Answer {
    match request {
        R::ListAiProviders => providers(engine),
        R::ListAiConnections => connections(engine),
        R::AddAiConnection {
            provider,
            label,
            base_url,
        } => add(engine, &provider, &label, base_url.as_deref()),
        R::UpdateAiConnection {
            connection,
            label,
            base_url,
            default_model,
        } => update(engine, &connection, &label, base_url, default_model),
        R::RemoveAiConnection { connection } => remove(engine, &connection),
        R::SetDefaultAiConnection { connection } => make_default(engine, &connection),
        R::SetAiApiKey { connection, key } => set_key(engine, &connection, key),
        R::ClearAiApiKey { connection } => clear_key(engine, &connection),
        R::TestAiConnection { connection } => test(engine, &connection).await,
        R::ListAiModels {
            connection,
            refresh,
        } => models(engine, &connection, refresh).await,
        R::AcknowledgeAiConsent { connection } => acknowledge(engine, &connection),
    }
}
