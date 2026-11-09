//! Domaine « assistant d'IA » : un fichier par opération, clés au trousseau, consentement par destination.

mod add_connection;
mod allowed_kinds;
mod clear_conversations;
mod clear_journal;
mod clear_key;
mod connection_view;
mod consent;
mod conversation_input;
mod default_masking;
mod delete_conversation;
mod get_conversation;
mod get_history_enabled;
mod key_login_cancel;
mod key_login_complete;
mod key_login_start;
mod list_connections;
mod list_conversations;
mod list_journal;
mod list_models;
mod list_providers;
mod remove_connection;
mod rename_conversation;
mod save_conversation;
mod session_login_complete;
mod session_login_start;
mod set_allowed_content;
mod set_default;
mod set_history_enabled;
mod set_key;
mod set_masking;
mod test_connection;
mod update_connection;
mod validate_key;
mod validate_label;
mod validate_url;

use mira_vocabulary::MiraError;

use crate::api::contract::{AiRequest, AiRequest as R, Response};
use crate::api::engine::Engine;
use add_connection::handle_add_ai_connection as add;
use clear_conversations::handle_clear_ai_conversations as clear_conversations;
use clear_journal::handle_clear_ai_journal as clear_journal;
use clear_key::handle_clear_ai_api_key as clear_key;
use consent::handle_acknowledge_ai_consent as acknowledge;
use delete_conversation::handle_delete_ai_conversation as delete_conversation;
use get_conversation::handle_get_ai_conversation as get_conversation;
use get_history_enabled::handle_get_ai_history_enabled as get_history;
use key_login_cancel::handle_cancel_ai_key_login as cancel_key_login;
use key_login_complete::handle_complete_ai_key_login as complete_key_login;
use key_login_start::handle_start_ai_key_login as start_key_login;
use list_connections::handle_list_ai_connections as connections;
use list_conversations::handle_list_ai_conversations as conversations;
use list_journal::handle_list_ai_journal as journal;
use list_models::handle_list_ai_models as models;
use list_providers::handle_list_ai_providers as providers;
use remove_connection::handle_remove_ai_connection as remove;
use rename_conversation::handle_rename_ai_conversation as rename_conversation;
use save_conversation::handle_save_ai_conversation as save_conversation;
use set_allowed_content::handle_set_ai_allowed_content as set_allowed;
use set_default::handle_set_default_ai_connection as make_default;
use set_history_enabled::handle_set_ai_history_enabled as set_history;
use set_key::handle_set_ai_api_key as set_key;
use set_masking::handle_set_ai_masking as set_mask;
use test_connection::handle_test_ai_connection as test;
use update_connection::handle_update_ai_connection as update;

/// Résultat d'une requête du domaine
type Answer = Result<Response, MiraError>;

/// Route une requête du domaine assistant d'IA
#[rustfmt::skip]
pub(crate) async fn route(engine: &Engine, request: AiRequest) -> Answer {
    match request {
        R::ListAiProviders => providers(engine),
        R::ListAiConnections => connections(engine),
        R::AddAiConnection { provider, label, base_url } => add(engine, &provider, &label, base_url.as_deref()),
        R::UpdateAiConnection { connection, label, base_url, default_model } => update(engine, &connection, &label, base_url, default_model),
        R::RemoveAiConnection { connection } => remove(engine, &connection),
        R::SetDefaultAiConnection { connection } => make_default(engine, &connection),
        R::SetAiApiKey { connection, key } => set_key(engine, &connection, key),
        R::ClearAiApiKey { connection } => clear_key(engine, &connection),
        R::StartAiKeyLogin { connection } => start_key_login(engine, &connection).await,
        R::CompleteAiKeyLogin { flow } => complete_key_login(engine, &flow).await,
        R::CancelAiKeyLogin { flow } => cancel_key_login(engine, &flow),
        R::TestAiConnection { connection } => test(engine, &connection).await,
        R::ListAiModels { connection, refresh } => models(engine, &connection, refresh).await,
        R::AcknowledgeAiConsent { connection, kinds } => acknowledge(engine, &connection, &kinds),
        R::SetAiAllowedContent { connection, kinds } => set_allowed(engine, &connection, &kinds),
        R::SetAiMasking { connection, masking } => set_mask(engine, &connection, masking),
        R::ListAiJournal { limit, before } => journal(engine, limit, before.as_deref()),
        R::ClearAiJournal => clear_journal(engine),
        R::SaveAiConversation { id, title, payload_version, payload } => save_conversation(engine, &id, &title, payload_version, &payload),
        R::ListAiConversations { limit, before } => conversations(engine, limit, before.as_deref()),
        R::GetAiConversation { id } => get_conversation(engine, &id),
        R::RenameAiConversation { id, title } => rename_conversation(engine, &id, &title),
        R::DeleteAiConversation { id } => delete_conversation(engine, &id),
        R::ClearAiConversations => clear_conversations(engine),
        R::SetAiHistoryEnabled { enabled } => set_history(engine, enabled),
        R::GetAiHistoryEnabled => get_history(engine),
    }
}
