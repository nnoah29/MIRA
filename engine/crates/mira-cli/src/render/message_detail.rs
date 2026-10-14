//! Rendu humain d'un message : en-têtes, corps, source brute, HTML assaini.

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use mira_assembly::api::contract::RawSourceView;
use mira_assembly::html::SafeHtml;
use mira_assembly::mira_vocabulary::MessageSummary;
use mira_assembly::reading::BodyView;

use super::addresses::address_list;
use super::instant::format_optional_instant;
use super::label::json_label;

/// En-têtes, état du contenu et aperçu d'un message.
pub fn message_text(message: &MessageSummary) -> String {
    let boxes: Vec<String> = message.mailboxes.iter().map(ToString::to_string).collect();
    format!(
        "ID      : {}\nCompte  : {}\nDe      : {}\nÀ       : {}\nCc      : {}\nObjet   : {}\nDate    : {}\nBoîtes  : {}\nContenu : {}\nTaille  : {} octets\nAperçu  : {}",
        message.id,
        message.account,
        address_list(&message.from),
        address_list(&message.to),
        address_list(&message.cc),
        message.subject,
        format_optional_instant(message.date.or(message.internal_date)),
        boxes.join(", "),
        json_label(&message.content),
        message.size,
        message.preview
    )
}

/// Corps en texte, avec un rappel si une version HTML existe.
pub fn body_text(body: &BodyView) -> String {
    if body.html_available {
        format!(
            "{}\n(version HTML disponible : mira message safe-html)",
            body.text
        )
    } else {
        body.text.clone()
    }
}

/// Source brute décodée, ou un avertissement si le contenu n'est pas du base64 valide.
pub fn raw_source_text(raw: &RawSourceView) -> String {
    match STANDARD.decode(raw.content_base64.as_bytes()) {
        Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
        Err(_) => format!("(source illisible, {} octets)", raw.size),
    }
}

/// HTML assaini et liste des ressources distantes retirées.
pub fn safe_html_text(safe: &SafeHtml) -> String {
    if safe.blocked_resources.is_empty() {
        return safe.html.clone();
    }
    let blocked: Vec<String> = safe
        .blocked_resources
        .iter()
        .map(|url| format!("  {url}"))
        .collect();
    format!(
        "{}\n-- {} ressource(s) distante(s) bloquée(s) :\n{}",
        safe.html,
        blocked.len(),
        blocked.join("\n")
    )
}
