//! Catalogue des outils : fiches annoncées au modèle, type de contenu de chaque résultat, extrait d'appel.

use mira_vocabulary::{ChatMode, ContentKind, ToolCall, ToolSpec};
use serde_json::Value;

use super::render::{cut, one_line};
use super::{
    compose_draft, edit_draft, get_message, get_thread, list_identities, list_mailboxes,
    list_messages, propose_action, search_messages,
};

const SUMMARY_CHARS: usize = 80;

/// Fiches de tous les outils.
pub fn specs() -> Vec<ToolSpec> {
    vec![
        search_messages::spec(),
        list_messages::spec(),
        get_message::spec(),
        get_thread::spec(),
        list_mailboxes::spec(),
        list_identities::spec(),
    ]
}

/// Outils annoncés dans ce mode : la lecture, plus l'outil de brouillon du mode.
pub fn specs_for(mode: ChatMode) -> Vec<ToolSpec> {
    let mut all = specs();
    match mode {
        ChatMode::Assistant => all.extend([compose_draft::spec(), propose_action::spec()]),
        ChatMode::Composer { selection } => all.push(edit_draft::spec(selection)),
    }
    all
}

/// Dit si l'outil est une proposition de brouillon : il n'envoie rien au service et n'exige aucun type de contenu.
pub fn is_proposal(name: &str) -> bool {
    [compose_draft::NAME, edit_draft::NAME, propose_action::NAME].contains(&name)
}

/// Type de contenu que renvoie l'outil, ou rien s'il est inconnu.
pub fn kind_of(name: &str) -> Option<ContentKind> {
    match name {
        search_messages::NAME | list_messages::NAME => Some(ContentKind::MailList),
        get_message::NAME | get_thread::NAME => Some(ContentKind::MailMessage),
        list_mailboxes::NAME | list_identities::NAME => Some(ContentKind::ToolResult),
        _ => None,
    }
}

/// Extrait d'argument à montrer à l'utilisateur (la requête saisie par le modèle), sur une ligne.
pub fn summary_of(call: &ToolCall) -> String {
    let arguments: Option<Value> = serde_json::from_str(&call.arguments).ok();
    let key = match call.name.as_str() {
        propose_action::NAME => "reason",
        name if is_proposal(name) => "subject",
        _ => "query",
    };
    let query = arguments
        .as_ref()
        .and_then(|value| value.get(key)?.as_str());
    cut(&one_line(query.unwrap_or_default()), SUMMARY_CHARS)
}
