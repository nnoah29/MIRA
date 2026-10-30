//! Outil `get_message` : en-têtes et corps texte d'un message, bornés à 8 000 caractères en tout.

use mira_vocabulary::{MessageId, ToolSpec};
use serde::Deserialize;

use super::args::parse;
use super::body::{RESULT_CHARS, body_text};
use super::execute::execute_read;
use super::message_view::header_block;
use super::run_result::ToolRun;
use crate::api::{Engine, MessageRequest, Request, Response};
use mira_vocabulary::ContentKind;

pub(super) const NAME: &str = "get_message";

const SCHEMA: &str = r#"{"type":"object","properties":{
"message":{"type":"string","description":"Identifiant du message (champ id d'une liste ou d'une recherche)."}
},"required":["message"]}"#;

/// Arguments de la lecture d'un message.
#[derive(Debug, Deserialize)]
struct MessageArgs {
    message: String,
}

/// Fiche de l'outil annoncée au modèle.
pub(super) fn spec() -> ToolSpec {
    ToolSpec {
        name: NAME.to_string(),
        description: "Lit un message : en-têtes et corps en texte, coupé s'il est très long."
            .to_string(),
        parameters_schema: SCHEMA.to_string(),
    }
}

/// Lit le message demandé par le modèle.
pub(super) async fn run(engine: &Engine, arguments: &str) -> ToolRun {
    let args: MessageArgs = match parse(arguments) {
        Ok(args) => args,
        Err(text) => return ToolRun::error(text),
    };
    let id = MessageId::from_raw(args.message);
    let head = Request::Message(MessageRequest::GetMessage {
        message: id.clone(),
    });
    match execute_read(engine, head).await {
        Ok(Response::Message(summary)) => {
            let head = header_block(&summary);
            let room = RESULT_CHARS.saturating_sub(head.chars().count() + 2);
            match body_text(engine, &id, room).await {
                Ok(body) => ToolRun::listed(
                    format!("{head}\n\n{body}"),
                    ContentKind::MailMessage,
                    vec![id.as_str().to_string()],
                ),
                Err(text) => ToolRun::error(text),
            }
        }
        Ok(_) => ToolRun::error("réponse inattendue du moteur".to_string()),
        Err(failure) => ToolRun::error(failure.text()),
    }
}
