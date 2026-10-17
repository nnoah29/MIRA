//! Corps texte et HTML d'un message analysé.

use mail_parser::{Message, MessagePart, PartType};

use super::html_text::html_to_text;

/// Texte d'une partie lisible ; une partie HTML est convertie en texte.
fn part_text(part: &MessagePart<'_>) -> Option<String> {
    match &part.body {
        PartType::Text(text) => Some(text.to_string()),
        PartType::Html(html) => Some(html_to_text(html)),
        _ => None,
    }
}

/// Source HTML d'une partie, si elle est vraiment de type HTML.
fn part_html(part: &MessagePart<'_>) -> Option<String> {
    match &part.body {
        PartType::Html(html) => Some(html.to_string()),
        _ => None,
    }
}

/// Corps texte : les parties texte jointes bout à bout, ou l'HTML converti.
pub(crate) fn text_of(message: &Message<'_>) -> String {
    let texts: Vec<String> = message
        .text_body
        .iter()
        .filter_map(|id| message.part(*id))
        .filter_map(part_text)
        .collect();
    texts.join("\n")
}

/// Corps HTML : les parties HTML jointes bout à bout, `None` s'il n'y en a pas.
pub(crate) fn html_of(message: &Message<'_>) -> Option<String> {
    let pages: Vec<String> = message
        .html_body
        .iter()
        .filter_map(|id| message.part(*id))
        .filter_map(part_html)
        .collect();
    (!pages.is_empty()).then(|| pages.join("\n"))
}
