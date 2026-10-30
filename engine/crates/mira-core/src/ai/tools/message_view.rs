//! En-têtes d'un message rendus pour le modèle, avant son corps.

use mira_vocabulary::MessageSummary;

use super::instant::rfc3339_of;
use super::render::{addresses, one_line};

/// Bloc d'en-têtes : identifiant, personnes, date, sujet, pièces jointes.
pub(super) fn header_block(message: &MessageSummary) -> String {
    let date = message.date.or(message.internal_date).map(rfc3339_of);
    let mut lines = vec![
        format!("id={}", message.id.as_str()),
        format!("de: {}", addresses(&message.from)),
        format!("à: {}", addresses(&message.to)),
    ];
    if !message.cc.is_empty() {
        lines.push(format!("cc: {}", addresses(&message.cc)));
    }
    lines.push(format!(
        "date: {}",
        date.unwrap_or_else(|| "(sans date)".to_string())
    ));
    lines.push(format!("sujet: {}", one_line(&message.subject)));
    let attached = if message.has_attachments {
        "oui"
    } else {
        "non"
    };
    lines.push(format!("pièces jointes: {attached}"));
    lines.join("\n")
}
