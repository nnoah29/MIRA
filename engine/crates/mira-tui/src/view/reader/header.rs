//! En-têtes du message ouvert, pièces jointes et fil de conversation.

use mira_assembly::mira_vocabulary::{AddressEntry, Attachment, MessageSummary};
use ratatui::text::Line;

use super::fit::{clip, fit};
use crate::state::AppState;
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
fn field(label: &str, value: String) -> Option<String> {
    (!value.is_empty()).then(|| format!("{label} : {value}"))
}

/// Une ligne par pièce jointe, précédée de son titre, avec le nom neutralisé et la taille.
fn attachment_lines(files: &[Attachment]) -> Vec<String> {
    if files.is_empty() {
        return Vec::new();
    }
    let mut lines = vec!["Pièces jointes :".to_string()];
    for file in files {
        let name = file
            .filename
            .as_deref()
            .map_or_else(|| "(sans nom)".into(), display_safe);
        lines.push(format!("  {name} ({} o)", file.size));
    }
    lines
}

/// Fil de conversation : une ligne par message, dans l'ordre reçu (du plus ancien au plus récent).
fn thread_lines(thread: &[MessageSummary]) -> Vec<String> {
    if thread.len() < 2 {
        return Vec::new();
    }
    let mut lines = vec![format!("Fil ({} messages) :", thread.len())];
    for message in thread {
        let date = message.date.or(message.internal_date).map(short_date);
        let sender = addresses(&message.from);
        let subject = display_safe(&message.subject);
        lines.push(format!("  {} {sender} {subject}", date.unwrap_or_default()));
    }
    lines
}

/// Champs d'en-tête non vides du message.
fn essential_lines(message: &MessageSummary) -> Vec<String> {
    let date = message.date.or(message.internal_date).map(short_date);
    [
        field("De", addresses(&message.from)),
        field("À", addresses(&message.to)),
        field("Cc", addresses(&message.cc)),
        field("Date", date.unwrap_or_default()),
        field("Objet", display_safe(&message.subject)),
    ]
    .into_iter()
    .flatten()
    .collect()
}

/// Lignes de l'en-tête tenant dans `budget` lignes de `width` colonnes ; fil et pièces jointes cèdent d'abord.
pub fn header_lines(
    message: &MessageSummary,
    state: &AppState,
    width: usize,
    budget: usize,
) -> Vec<Line<'static>> {
    let mut lines = essential_lines(message);
    let attachments = attachment_lines(&state.reader.attachments);
    let thread = thread_lines(&state.reader.thread);
    let room = budget.saturating_sub(lines.len() + 1);
    let for_files = if thread.is_empty() {
        room
    } else {
        room - room / 2
    };
    let files = fit(attachments, for_files);
    lines.extend(files.iter().cloned());
    lines.extend(fit(thread, room - files.len()));
    lines.push("─".repeat(width));
    lines.truncate(budget);
    lines
        .iter()
        .map(|line| Line::from(clip(line, width)))
        .collect()
}
