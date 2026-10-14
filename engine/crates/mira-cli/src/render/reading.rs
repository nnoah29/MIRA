//! Rendu humain des réponses de lecture : boîtes, messages, fils, pièces jointes, recherche.

use mira_assembly::api::Response;

use super::generic::pretty_json;
use super::{attachments, mailboxes, message_detail, messages, search, threads};

/// Texte d'une réponse de lecture ; les autres formes retombent sur le JSON du contrat.
pub fn reading_text(response: &Response) -> String {
    match response {
        Response::Mailboxes(list) => mailboxes::mailboxes_table(list),
        Response::Messages(page) => messages::messages_page_text(page),
        Response::Message(one) => message_detail::message_text(one),
        Response::MessageBody(body) => message_detail::body_text(body),
        Response::RawSource(raw) => message_detail::raw_source_text(raw),
        Response::Attachments(list) => attachments::attachments_table(list),
        Response::SafeHtml(safe) => message_detail::safe_html_text(safe),
        Response::Threads(page) => threads::threads_page_text(page),
        Response::Thread(view) => threads::thread_view_text(view),
        Response::ServerSearch(result) => search::server_search_text(result),
        other => pretty_json(other),
    }
}
