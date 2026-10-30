//! Niveau de risque de chaque requête du contrat : un `match` exhaustif, sans cas par défaut.
//! Une nouvelle requête ne compile pas tant qu'on ne lui a pas donné de niveau.

mod assistant;
mod labels;
mod reading;
mod system;
mod writing;

use mira_vocabulary::ToolRisk;

use super::Request;

/// Niveau de risque d'une requête quand un modèle peut la demander ; seul `Read` est exposable.
pub fn risk_of(request: &Request) -> ToolRisk {
    match request {
        Request::Engine(request) => system::engine_risk(request),
        Request::Provider(request) => system::provider_risk(request),
        Request::Account(request) => reading::account_risk(request),
        Request::Identity(request) => system::identity_risk(request),
        Request::Mailbox(request) => reading::mailbox_risk(request),
        Request::Message(request) => reading::message_risk(request),
        Request::Thread(request) => reading::thread_risk(request),
        Request::Search(request) => reading::search_risk(request),
        Request::Maintenance(request) => reading::maintenance_risk(request),
        Request::Auth(request) => system::auth_risk(request),
        Request::Operation(request) => system::operation_risk(request),
        Request::Event(request) => system::event_risk(request),
        Request::Action(request) => writing::action_risk(request),
        Request::Compose(request) => writing::compose_risk(request),
        Request::Queue(request) => writing::queue_risk(request),
        Request::Sync(request) => writing::sync_risk(request),
        Request::Rule(request) => labels::rule_risk(request),
        Request::LocalLabel(request) => labels::local_label_risk(request),
        Request::SystemLabel(request) => labels::system_label_risk(request),
        Request::LabelSettings(request) => labels::label_settings_risk(request),
        Request::Settings(request) => labels::settings_risk(request),
        Request::Template(request) => labels::template_risk(request),
        Request::Ai(request) => assistant::ai_risk(request),
    }
}
