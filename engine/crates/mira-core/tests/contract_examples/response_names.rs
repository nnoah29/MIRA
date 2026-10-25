//! Noms des réponses du contrat : un `match` exhaustif sans `_` force à compléter les exemples.

use mira_core::api::Response;

pub const TYPES: &[&str] = &[
    "done",
    "contract_info",
    "status",
    "providers",
    "provider",
    "account_preparation",
    "accounts",
    "account",
    "identities",
    "identity",
    "oauth_started",
    "operation",
    "events",
    "mailboxes",
    "messages",
    "message",
    "message_body",
    "raw_source",
    "attachments",
    "safe_html",
    "threads",
    "thread",
    "server_search",
    "storage_stats",
    "integrity",
    "exported",
];

/// Rattache chaque réponse à son fichier d'exemples ; une nouvelle variante ne compile pas sans choix
#[rustfmt::skip]
pub fn group_of(response: &Response) -> &'static str {
    match response {
        Response::Done | Response::ContractInfo(_) | Response::Status(_) | Response::Operation(_)
        | Response::Events(_) | Response::Providers(_) | Response::Provider(_)
        | Response::AccountPreparation(_) | Response::Accounts(_) | Response::Account(_)
        | Response::Identities(_) | Response::Identity(_) | Response::OAuthStarted(_) => "responses_legacy",
        Response::Mailboxes(_) | Response::Messages(_) | Response::Message(_)
        | Response::MessageBody(_) | Response::RawSource(_) | Response::Attachments(_)
        | Response::SafeHtml(_) | Response::Threads(_) | Response::Thread(_)
        | Response::ServerSearch(_) | Response::StorageStats(_) | Response::Integrity(_)
        | Response::Exported(_) => "responses_read",
        Response::Write(_) => "actions",
        Response::Sync(_) => "sync",
        Response::Rule(_) => "rule",
        Response::LocalLabel(_) => "local_label",
        Response::LabelSettings(_) => "label_settings",
        Response::Settings(_) => "settings",
        Response::Template(_) => "template",
    }
}
