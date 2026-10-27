//! Exemples sérialisés du contrat, un fichier par domaine ; chaque nouvelle variante y ajoute son exemple.
#![allow(dead_code)]

pub mod account_domain;
pub mod actions;
pub mod compose;
pub mod compose_drafts;
pub mod compose_sending;
pub mod label_portability;
pub mod label_settings_domain;
pub mod local_label_domain;
pub mod mailbox_domain;
pub mod maintenance_domain;
pub mod message_domain;
pub mod queue;
pub mod response_names;
pub mod response_samples;
pub mod responses_legacy;
pub mod responses_read;
pub mod rule_domain;
pub mod rule_portability;
pub mod search_domain;
pub mod settings_domain;
pub mod small_domains;
pub mod snapshot;
pub mod sync_domain;
pub mod template_domain;
pub mod write_domain;

pub use write_domain::{numbered, write_requests, write_responses};

use mira_core::api::{Request, Response};

/// Un exemple nommé (`domaine.opération`, suivi d'un rang si l'opération a plusieurs exemples).
pub type Named<T> = (String, T);

/// Nom `domaine.opération` d'une requête, par `match` exhaustif sur tous les domaines
pub fn request_name(request: &Request) -> &'static str {
    match request {
        Request::Engine(inner) => small_domains::engine_name(inner),
        Request::Provider(inner) => small_domains::provider_name(inner),
        Request::Operation(inner) => small_domains::operation_name(inner),
        Request::Event(inner) => small_domains::event_name(inner),
        Request::Account(inner) => account_domain::account_name(inner),
        Request::Identity(inner) => account_domain::identity_name(inner),
        Request::Auth(inner) => account_domain::auth_name(inner),
        Request::Mailbox(inner) => mailbox_domain::mailbox_name(inner),
        Request::Message(inner) => message_domain::message_name(inner),
        Request::Thread(inner) => search_domain::thread_name(inner),
        Request::Search(inner) => search_domain::search_name(inner),
        Request::Maintenance(inner) => maintenance_domain::maintenance_name(inner),
        Request::Action(_) | Request::Compose(_) | Request::Queue(_) => "write",
        Request::Sync(_) => "sync",
        Request::Rule(_) => "rule",
        Request::LocalLabel(_) => "local_label",
        Request::LabelSettings(_) => "label_settings",
        Request::Settings(_) => "settings",
        Request::Template(_) => "template",
    }
}

/// Textes JSON de tous les exemples de requêtes
pub fn request_texts() -> Vec<String> {
    let lists: [&[&str]; 6] = [
        small_domains::EXAMPLES,
        account_domain::EXAMPLES,
        mailbox_domain::EXAMPLES,
        message_domain::EXAMPLES,
        search_domain::EXAMPLES,
        maintenance_domain::EXAMPLES,
    ];
    let texts = lists.iter().flat_map(|list| list.iter());
    texts.map(|text| (*text).to_string()).collect()
}

/// Noms attendus de toutes les requêtes, tous domaines confondus
pub fn expected_request_names() -> Vec<&'static str> {
    let lists: [&[&str]; 6] = [
        small_domains::NAMES,
        account_domain::NAMES,
        mailbox_domain::NAMES,
        message_domain::NAMES,
        search_domain::NAMES,
        maintenance_domain::NAMES,
    ];
    lists.iter().flat_map(|list| list.iter().copied()).collect()
}

/// Tous les exemples de réponses
pub fn all_responses() -> Vec<Response> {
    let mut all = responses_legacy::responses();
    all.extend(responses_read::responses());
    all
}
