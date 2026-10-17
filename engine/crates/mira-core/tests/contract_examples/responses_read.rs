//! Exemples de réponses des domaines de lecture et de maintenance.

use mira_core::api::Response;
use mira_core::api::contract::{IntegrityReport, RawSourceView, ThreadView};
use mira_core::html::SafeHtml;
use mira_core::reading::BodyView;
use mira_vocabulary::{Cursor, MailboxRole, Page};

use super::response_samples::{
    sample_attachment, sample_export, sample_search_result, sample_stats, sample_thread,
};
use crate::support_mirror::fixtures::{sample_mailbox, sample_message};

/// Réponses de lecture : boîtes, messages, corps, source, pièces jointes, HTML
fn message_responses() -> Vec<Response> {
    let page = Page {
        items: vec![sample_message("m", "a")],
        next_cursor: Some(Cursor("c".to_string())),
    };
    let body = BodyView {
        text: "texte".to_string(),
        html_available: true,
    };
    let raw = RawSourceView {
        size: 3,
        content_base64: "YWJj".to_string(),
    };
    let safe = SafeHtml {
        html: "<p>x</p>".to_string(),
        blocked_resources: vec!["https://t.example/p.gif".to_string()],
    };
    vec![
        Response::Mailboxes(vec![sample_mailbox("b", "a", "INBOX", MailboxRole::Inbox)]),
        Response::Messages(page),
        Response::Message(sample_message("m", "a")),
        Response::MessageBody(body),
        Response::RawSource(raw),
        Response::Attachments(vec![sample_attachment()]),
        Response::SafeHtml(safe),
    ]
}

/// Réponses des fils, de la recherche serveur et de la maintenance
fn thread_and_maintenance_responses() -> Vec<Response> {
    let view = ThreadView {
        thread: sample_thread(),
        messages: vec![sample_message("m", "a")],
    };
    let threads = Page {
        items: vec![sample_thread()],
        next_cursor: None,
    };
    vec![
        Response::Threads(threads),
        Response::Thread(view),
        Response::ServerSearch(sample_search_result()),
        Response::StorageStats(sample_stats()),
        Response::Integrity(IntegrityReport { ok: true }),
        Response::Exported(sample_export()),
    ]
}

/// Un exemple par réponse des domaines de lecture et de maintenance
pub fn responses() -> Vec<Response> {
    let mut all = message_responses();
    all.extend(thread_and_maintenance_responses());
    all
}
