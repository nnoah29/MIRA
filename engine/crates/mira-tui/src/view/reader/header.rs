//! En-têtes du message ouvert, pièces jointes et fil de conversation.

use mira_assembly::mira_vocabulary::{AddressEntry, Attachment, MessageSummary};
use ratatui::text::Line;

use crate::state::AppState;
use crate::theme;
use crate::view::message_list::short_date::short_date;
use crate::view::safe_text::display_safe;

/// Adresses sous la forme `nom <adresse>`, neutralisées, séparées par des virgules.
fn addresses(entries: &[AddressEntry]) -> String {
    let shown: Vec<String> = entries
        .iter()
        .map(
            |entry| match entry.name.as_deref().filter(|n| !n.trim().is_empty()) {
                Some(name) => format!("{} <{}>", display_safe(name), display_safe(&entry.address)),
                None => display_safe(&entry.address),
            },
        )
        .collect();
    shown.join(", ")
}

/// Ligne `Libellé : valeur` ; rien si la valeur est vide.
fn field(label: &str, value: String) -> Option<Line<'static>> {
    (!value.is_empty()).then(|| Line::from(format!("{label} : {value}")))
}

/// Une ligne par pièce jointe avec son nom neutralisé et sa taille.
fn attachment_lines(files: &[Attachment]) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    for file in files {
        let name = file
            .filename
            .as_deref()
            .map_or_else(|| "(sans nom)".into(), display_safe);
        lines.push(Line::from(format!("  {name} ({} o)", file.size)));
    }
    lines
}

/// Fil de conversation : une ligne par message, dans l'ordre reçu (du plus ancien au plus récent).
fn thread_lines(thread: &[MessageSummary]) -> Vec<Line<'static>> {
    let mut lines = vec![Line::styled(
        format!("Fil ({} messages) :", thread.len()),
        theme::dim(),
    )];
    for message in thread {
        let date = message.date.or(message.internal_date).map(short_date);
        let sender = addresses(&message.from);
        let subject = display_safe(&message.subject);
        lines.push(Line::from(format!(
            "  {} {sender} {subject}",
            date.unwrap_or_default()
        )));
    }
    lines
}

/// Toutes les lignes de l'en-tête du lecteur, séparateur compris.
pub fn header_lines(message: &MessageSummary, state: &AppState) -> Vec<Line<'static>> {
    let date = message.date.or(message.internal_date).map(short_date);
    let mut lines: Vec<Line<'static>> = [
        field("De", addresses(&message.from)),
        field("À", addresses(&message.to)),
        field("Cc", addresses(&message.cc)),
        field("Date", date.unwrap_or_default()),
        field("Objet", display_safe(&message.subject)),
    ]
    .into_iter()
    .flatten()
    .collect();
    if !state.reader.attachments.is_empty() {
        lines.push(Line::styled("Pièces jointes :", theme::dim()));
        lines.extend(attachment_lines(&state.reader.attachments));
    }
    if state.reader.thread.len() > 1 {
        lines.extend(thread_lines(&state.reader.thread));
    }
    lines.push(Line::styled("─".repeat(20), theme::dim()));
    lines
}
