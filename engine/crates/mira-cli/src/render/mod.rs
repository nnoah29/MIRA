//! Rendu humain d'une réponse du contrat ; la version JSON est la réponse elle-même.

pub mod accounts;
pub mod engine_info;
pub mod events;
pub mod flows;
pub mod identities;
pub mod label;
pub mod providers;
pub mod table;

use mira_assembly::api::Response;

/// Texte lisible d'une réponse, un `match` exhaustif sur toutes ses formes.
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
        Response::Write(written) => serde_json::to_string_pretty(written).unwrap_or_default(),
    }
}
