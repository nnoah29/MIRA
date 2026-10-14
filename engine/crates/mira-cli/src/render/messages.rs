//! Rendu humain d'une page de messages.

use mira_assembly::mira_vocabulary::{MessageSummary, Page};

use super::addresses::first_party;
use super::instant::format_optional_instant;
use super::pagination::with_next_page;
use super::table::render_table;

/// Drapeaux lisibles d'un message : non lu, suivi, répondu, pièces jointes.
fn flags_label(message: &MessageSummary) -> String {
    let mut words = Vec::new();
    if !message.flags.seen {
        words.push("non-lu");
    }
    if message.flags.flagged {
        words.push("suivi");
    }
    if message.flags.answered {
        words.push("répondu");
    }
    if message.has_attachments {
        words.push("pj");
    }
    words.join(",")
}

/// Cellules d'une ligne de message.
fn message_row(message: &MessageSummary) -> Vec<String> {
    vec![
        message.id.to_string(),
        format_optional_instant(message.date.or(message.internal_date)),
        first_party(&message.from),
        message.subject.clone(),
        flags_label(message),
    ]
}

/// Tableau des messages d'une page, suivi de la commande de la page suivante.
pub fn messages_page_text(page: &Page<MessageSummary>) -> String {
    let rows: Vec<Vec<String>> = page.items.iter().map(message_row).collect();
    let table = render_table(&["ID", "DATE", "DE", "SUJET", "ÉTAT"], &rows);
    with_next_page(table, page.next_cursor.as_ref())
}
