//! Exemples du domaine de la lecture des messages.

use mira_core::api::MessageRequest;

pub const NAMES: &[&str] = &[
    "message.list_messages",
    "message.get_message",
    "message.get_message_body",
    "message.fetch_message_body",
    "message.get_raw_source",
    "message.list_attachments",
    "message.save_attachment",
    "message.get_safe_html",
    "message.allow_remote_content",
];

pub const EXAMPLES: &[&str] = &[
    r#"{"domain":"message","request":{"op":"list_messages","view":{"type":"mailbox","mailbox":"m"},"filters":{"unread_only":false,"flagged_only":false,"has_attachments":false,"from_contains":null,"since":null,"until":null},"page":{"cursor":null,"limit":50}}}"#,
    r#"{"domain":"message","request":{"op":"get_message","message":"x"}}"#,
    r#"{"domain":"message","request":{"op":"get_message_body","message":"x"}}"#,
    r#"{"domain":"message","request":{"op":"fetch_message_body","message":"x"}}"#,
    r#"{"domain":"message","request":{"op":"get_raw_source","message":"x"}}"#,
    r#"{"domain":"message","request":{"op":"list_attachments","message":"x"}}"#,
    r#"{"domain":"message","request":{"op":"save_attachment","attachment":"t","destination":"/tmp/out.bin"}}"#,
    r#"{"domain":"message","request":{"op":"get_safe_html","message":"x"}}"#,
    r#"{"domain":"message","request":{"op":"allow_remote_content","scope":{"type":"sender","address":"a@b.c"}}}"#,
];

/// Nom `domaine.opération` d'une requête du domaine messages
pub fn message_name(request: &MessageRequest) -> &'static str {
    match request {
        MessageRequest::ListMessages(_) => "message.list_messages",
        MessageRequest::GetMessage { .. } => "message.get_message",
        MessageRequest::GetMessageBody { .. } => "message.get_message_body",
        MessageRequest::FetchMessageBody { .. } => "message.fetch_message_body",
        MessageRequest::GetRawSource { .. } => "message.get_raw_source",
        MessageRequest::ListAttachments { .. } => "message.list_attachments",
        MessageRequest::SaveAttachment(_) => "message.save_attachment",
        MessageRequest::GetSafeHtml { .. } => "message.get_safe_html",
        MessageRequest::AllowRemoteContent { .. } => "message.allow_remote_content",
    }
}
