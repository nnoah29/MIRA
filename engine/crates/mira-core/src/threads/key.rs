//! Clé de fil d'un message (D-15) : l'identité de la racine de sa conversation.

use mira_vocabulary::MessageSummary;

use super::subject::normalize_subject;

/// Clé de fil : premier `References`, sinon `In-Reply-To`, sinon indice serveur, sinon `Message-ID`, sinon l'objet
pub fn thread_key(summary: &MessageSummary, hint: Option<&str>) -> String {
    let reference = summary.references.iter().find_map(|value| non_empty(value));
    let reply_to = summary.in_reply_to.as_deref().and_then(non_empty);
    let own_id = summary.message_id_header.as_deref().and_then(non_empty);
    let hint = hint
        .and_then(non_empty)
        .map(|value| format!("hint:{value}"));
    reference
        .or(reply_to)
        .map(str::to_string)
        .or(hint)
        .or_else(|| own_id.map(str::to_string))
        .unwrap_or_else(|| format!("subject:{}", normalize_subject(&summary.subject)))
}

/// Valeur d'en-tête débarrassée de ses espaces, ou rien si elle est vide
fn non_empty(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then_some(trimmed)
}
