//! Outil `get_thread` : messages d'un fil avec leurs corps, qui se partagent la borne de 8 000 caractères.

use mira_vocabulary::{ContentKind, ThreadId, ToolSpec};
use serde::Deserialize;

use super::args::parse;
use super::body::{RESULT_CHARS, body_text};
use super::execute::execute_read;
use super::message_view::header_block;
use super::render::{cut, one_line};
use super::run_result::ToolRun;
use crate::api::contract::ThreadView;
use crate::api::{Engine, Request, Response, ThreadRequest};

pub(super) const NAME: &str = "get_thread";

const SCHEMA: &str = r#"{"type":"object","properties":{
"thread":{"type":"string","description":"Identifiant du fil (champ thread d'un message)."}
},"required":["thread"]}"#;

const MIN_BODY_CHARS: usize = 200;
const OMITTED_RESERVE: usize = 64;

/// Arguments de la lecture d'un fil.
#[derive(Debug, Deserialize)]
struct ThreadArgs {
    thread: String,
}

/// Fiche de l'outil annoncée au modèle.
pub(super) fn spec() -> ToolSpec {
    ToolSpec {
        name: NAME.to_string(),
        description: "Lit un fil de conversation : ses messages dans l'ordre, avec leur corps en texte, coupé s'il est très long.".to_string(),
        parameters_schema: SCHEMA.to_string(),
    }
}

/// Lit le fil demandé par le modèle.
pub(super) async fn run(engine: &Engine, arguments: &str) -> ToolRun {
    let args: ThreadArgs = match parse(arguments) {
        Ok(args) => args,
        Err(text) => return ToolRun::error(text),
    };
    let request = Request::Thread(ThreadRequest::GetThread {
        thread: ThreadId::from_raw(args.thread),
    });
    match execute_read(engine, request).await {
        Ok(Response::Thread(view)) => thread_run(engine, &view).await,
        Ok(_) => ToolRun::error("réponse inattendue du moteur".to_string()),
        Err(failure) => ToolRun::error(failure.text()),
    }
}

/// Rend les messages du fil tant qu'il reste de la place, puis annonce combien sont omis.
async fn thread_run(engine: &Engine, view: &ThreadView) -> ToolRun {
    let mut text = format!("fil: {}\n", cut(&one_line(&view.thread.subject), 200));
    let mut sources = Vec::new();
    for message in &view.messages {
        let head = header_block(message);
        let used = text.chars().count() + head.chars().count() + OMITTED_RESERVE + 6;
        let room = RESULT_CHARS.saturating_sub(used);
        if room < MIN_BODY_CHARS {
            break;
        }
        match body_text(engine, &message.id, room).await {
            Ok(body) => text.push_str(&format!("---\n{head}\n\n{body}\n")),
            Err(error) => return ToolRun::error(error),
        }
        sources.push(message.id.as_str().to_string());
    }
    let omitted = view.messages.len() - sources.len();
    if omitted > 0 {
        text.push_str(&format!("[{omitted} message(s) suivant(s) omis]"));
    }
    ToolRun::listed(text, ContentKind::MailMessage, sources)
}
