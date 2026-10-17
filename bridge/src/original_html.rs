//! HTML d'origine d'un message : lu dans la source brute locale, sans assainissement (AG-47).

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use mail_parser::{MessageParser, PartType};
use mira_assembly::api::{MessageRequest, Request, Response};
use mira_assembly::mira_vocabulary::MessageId;
use serde_json::json;

use crate::envelope::{failure, internal_failure, success};
use crate::session::current;

/// Pages HTML (vraies parties `text/html`) d'une source brute, jointes bout à bout.
pub fn html_of_raw(raw: &[u8]) -> Option<String> {
    let message = MessageParser::default().parse(raw)?;
    let pages: Vec<String> = message
        .html_body
        .iter()
        .filter_map(|id| message.part(*id))
        .filter_map(|part| match &part.body {
            PartType::Html(html) => Some(html.to_string()),
            _ => None,
        })
        .collect();
    (!pages.is_empty()).then(|| pages.join("\n"))
}

/// Lit la source brute locale du message et rend l'enveloppe `{html}` (`null` sans partie HTML).
pub async fn fetch(message: &str) -> String {
    let Some(engine) = current() else {
        return internal_failure("moteur non ouvert");
    };
    let request = Request::Message(MessageRequest::GetRawSource {
        message: MessageId::from_raw(message),
    });
    match engine.execute(request).await {
        Ok(Response::RawSource(view)) => decode(&view.content_base64),
        Ok(_) => internal_failure("réponse inattendue"),
        Err(error) => failure(&error),
    }
}

/// Décode la source en base64 et en extrait le HTML.
fn decode(content_base64: &str) -> String {
    match STANDARD.decode(content_base64) {
        Ok(raw) => success(json!({ "html": html_of_raw(&raw) })),
        Err(_) => internal_failure("source brute illisible"),
    }
}
