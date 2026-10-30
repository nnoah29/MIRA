//! Rendu texte compact des messages pour le modèle : une ligne par message, identifiant en tête.

use mira_vocabulary::{AddressEntry, ContentKind, MessageSummary, Page};

use super::instant::rfc3339_of;
use super::run_result::ToolRun;

const PREVIEW_CHARS: usize = 200;

/// Ligne d'un message : identifiant, expéditeur, date, sujet et aperçu coupé à 200 caractères.
pub fn mail_line(message: &MessageSummary) -> String {
    let date = message.date.or(message.internal_date).map(rfc3339_of);
    format!(
        "id={} | de: {} | {} | {} | {}",
        message.id.as_str(),
        addresses(&message.from),
        date.unwrap_or_else(|| "(sans date)".to_string()),
        one_line(&message.subject),
        cut(&one_line(&message.preview), PREVIEW_CHARS)
    )
}

/// Page de messages rendue en lignes, suivie de l'indication de page suivante et de son curseur.
pub(super) fn mail_list(page: &Page<MessageSummary>) -> ToolRun {
    let mut lines: Vec<String> = page.items.iter().map(mail_line).collect();
    if lines.is_empty() {
        lines.push("(aucun résultat)".to_string());
    }
    lines.push(format!("has_more: {}", page.next_cursor.is_some()));
    if let Some(cursor) = &page.next_cursor {
        lines.push(format!("next_cursor: {}", cursor.0));
    }
    let sources = page
        .items
        .iter()
        .map(|m| m.id.as_str().to_string())
        .collect();
    ToolRun::listed(lines.join("\n"), ContentKind::MailList, sources)
}

/// Personnes d'une liste d'adresses, séparées par des virgules ; « (inconnu) » si elle est vide.
pub(super) fn addresses(list: &[AddressEntry]) -> String {
    match list.is_empty() {
        true => "(inconnu)".to_string(),
        false => list.iter().map(person).collect::<Vec<_>>().join(", "),
    }
}

/// Une personne sous la forme `Nom <adresse>`, ou l'adresse seule.
fn person(entry: &AddressEntry) -> String {
    match &entry.name {
        Some(name) if !name.is_empty() => format!("{} <{}>", one_line(name), entry.address),
        _ => entry.address.clone(),
    }
}

/// Texte sur une seule ligne : retours à la ligne et tabulations deviennent des espaces.
pub(super) fn one_line(text: &str) -> String {
    text.replace(['\n', '\r', '\t'], " ").trim().to_string()
}

/// Garde les `max` premiers caractères.
pub(super) fn cut(text: &str, max: usize) -> String {
    text.chars().take(max).collect()
}
