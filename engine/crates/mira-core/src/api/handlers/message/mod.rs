//! Domaine « lecture des messages » : requêtes locales et commandes réseau explicites.

mod allow_remote;
mod attachments;
mod body;
mod fetch_body;
mod get;
mod list;
mod raw_source;
mod safe_html;
mod save_attachment;

use mira_vocabulary::MiraError;

use self::allow_remote::handle_allow_remote_content;
use self::attachments::handle_list_attachments;
use self::body::handle_get_body;
use self::fetch_body::handle_fetch_message_body;
use self::get::handle_get_message;
use self::list::handle_list_messages;
use self::raw_source::handle_get_raw_source;
use self::safe_html::handle_get_safe_html;
use self::save_attachment::handle_save_attachment;
use crate::api::contract::{MessageRequest, Response};
use crate::api::engine::Engine;

/// Route une requête du domaine messages
pub(crate) async fn route(engine: &Engine, request: MessageRequest) -> Result<Response, MiraError> {
    match request {
        MessageRequest::ListMessages(query) => handle_list_messages(engine, &query),
        MessageRequest::GetMessage { message } => handle_get_message(engine, &message),
        MessageRequest::GetMessageBody { message } => handle_get_body(engine, &message),
        MessageRequest::FetchMessageBody { message } => handle_fetch_message_body(engine, message),
        MessageRequest::GetRawSource { message } => handle_get_raw_source(engine, &message),
        MessageRequest::ListAttachments { message } => handle_list_attachments(engine, &message),
        MessageRequest::SaveAttachment(target) => handle_save_attachment(engine, target),
        MessageRequest::GetSafeHtml { message } => handle_get_safe_html(engine, &message),
        MessageRequest::AllowRemoteContent { scope } => handle_allow_remote_content(engine, &scope),
    }
}
