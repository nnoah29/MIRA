//! Réponses du moteur étiquetées pour les pièces jointes et le fil du message `m1`.
#![allow(dead_code)]

use crate::message_fixtures::summary;
use crate::reader_fixtures::{attachment, thread_view};
use mira_assembly::api::Response;
use mira_assembly::mira_vocabulary::{MessageId, ThreadId};
use mira_tui::commands::Tag;
use mira_tui::update::{Msg, Reply};

/// Réponse « pièces jointes » avec ce nombre de fichiers pour `m1`.
pub fn attachments_reply(count: usize) -> Msg {
    let files = (0..count)
        .map(|n| attachment(&format!("f{n}"), "m1", &format!("pj{n}.pdf")))
        .collect();
    Msg::Reply(Reply {
        tag: Tag::Attachments(MessageId::from_raw("m1")),
        result: Ok(Response::Attachments(files)),
    })
}

/// Réponse « fil » `t1` avec ce nombre de messages.
pub fn thread_reply(count: usize) -> Msg {
    let messages = (0..count)
        .map(|n| summary(&format!("t{n}"), &format!("fil{n}"), "Ann"))
        .collect();
    Msg::Reply(Reply {
        tag: Tag::Thread(ThreadId::from_raw("t1")),
        result: Ok(Response::Thread(thread_view("t1", messages))),
    })
}
