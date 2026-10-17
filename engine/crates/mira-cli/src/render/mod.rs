//! Rendu humain d'une réponse du contrat ; la version JSON est la réponse elle-même.

pub mod accounts;
pub mod addresses;
pub mod attachments;
pub mod drafts;
pub mod engine_info;
pub mod events;
pub mod flows;
pub mod generic;
pub mod identities;
pub mod instant;
pub mod label;
pub mod mailboxes;
pub mod maintenance;
pub mod message_detail;
pub mod messages;
pub mod pagination;
pub mod providers;
pub mod queue;
pub mod reading;
pub mod search;
pub mod sync_status;
pub mod table;
pub mod threads;
pub mod write_responses;

use mira_assembly::api::Response;

/// Texte lisible d'une réponse, un `match` exhaustif sur toutes ses formes.
#[rustfmt::skip]
pub fn human_text(response: &Response) -> String {
    match response {
        Response::Done => "ok".to_string(),
        Response::ContractInfo(info) => engine_info::contract_text(info),
        Response::Status(status) => engine_info::status_text(status),
        Response::Providers(list) => providers::providers_table(list),
        Response::Provider(one) => providers::providers_table(std::slice::from_ref(one)),
        Response::AccountPreparation(prep) => providers::preparation_text(prep),
        Response::Accounts(list) => accounts::accounts_table(list),
        Response::Account(one) => accounts::accounts_table(std::slice::from_ref(one)),
        Response::Identities(list) => identities::identities_table(list),
        Response::Identity(one) => identities::identities_table(std::slice::from_ref(one)),
        Response::OAuthStarted(start) => flows::oauth_start_text(start),
        Response::Operation(info) => flows::operation_text(info),
        Response::Events(list) => events::events_table(list),
        Response::Mailboxes(_) | Response::Messages(_) | Response::Message(_)
        | Response::MessageBody(_) | Response::RawSource(_) | Response::Attachments(_)
        | Response::SafeHtml(_) | Response::Threads(_) | Response::Thread(_)
        | Response::ServerSearch(_) => reading::reading_text(response),
        Response::StorageStats(_) | Response::Integrity(_) | Response::Exported(_) => {
            maintenance::maintenance_text(response)
        }
        Response::Write(write) => write_responses::write_text(write),
        Response::Sync(sync) => sync_status::sync_text(sync),
    }
}
