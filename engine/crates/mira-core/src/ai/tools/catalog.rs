//! Catalogue des outils : fiches annoncées au modèle, type de contenu de chaque résultat, extrait d'appel.

use mira_vocabulary::{ContentKind, ToolCall, ToolSpec};
use serde_json::Value;

use super::render::{cut, one_line};
use super::{get_message, get_thread, list_mailboxes, list_messages, search_messages};

const SUMMARY_CHARS: usize = 80;

/// Fiches de tous les outils.
pub fn specs() -> Vec<ToolSpec> {
    vec![
        search_messages::spec(),
        list_messages::spec(),
        get_message::spec(),
        get_thread::spec(),
        list_mailboxes::spec(),
    ]
}

/// Type de contenu que renvoie l'outil, ou rien s'il est inconnu.
pub fn kind_of(name: &str) -> Option<ContentKind> {
    match name {
        search_messages::NAME | list_messages::NAME => Some(ContentKind::MailList),
        get_message::NAME | get_thread::NAME => Some(ContentKind::MailMessage),
        list_mailboxes::NAME => Some(ContentKind::ToolResult),
        _ => None,
    }
}

/// Extrait d'argument à montrer à l'utilisateur (la requête saisie par le modèle), sur une ligne.
pub fn summary_of(call: &ToolCall) -> String {
    let arguments: Option<Value> = serde_json::from_str(&call.arguments).ok();
    let query = arguments
        .as_ref()
        .and_then(|value| value.get("query")?.as_str());
    cut(&one_line(query.unwrap_or_default()), SUMMARY_CHARS)
}
