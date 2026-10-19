//! Exemples du domaine « rédaction » : brouillons, pièces jointes, envoi, boîte d'envoi.

use mira_core::api::{ComposeRequest, OutboxEntry, Request, Response, WriteResponse};
use mira_vocabulary::{
    AccountId, ActionId, ActionState, AddressEntry, Draft, DraftAttachment, DraftId, DraftState,
    IdentityId, MessageId, Timestamp,
};

use super::compose_drafts::{creation_requests, editing_requests, server_copy_requests};
use super::compose_sending::sending_requests;
use super::{Named, numbered};

/// Brouillon d'exemple, avec une pièce jointe et un envoi planifié
fn sample_draft() -> Draft {
    let bob = AddressEntry {
        name: Some("Bob".to_string()),
        address: "bob@example.org".to_string(),
    };
    let attachment = DraftAttachment {
        filename: "note.txt".to_string(),
        mime_type: "text/plain".to_string(),
        size: 12,
    };
    Draft {
        id: DraftId::from_raw("draft-1"),
        account: AccountId::from_raw("account-1"),
        identity: IdentityId::from_raw("identity-1"),
        to: vec![bob],
        cc: Vec::new(),
        bcc: Vec::new(),
        subject: "Bonjour".to_string(),
        body_text: "Corps".to_string(),
        body_html: None,
        attachments: vec![attachment],
        in_reply_to: Some(MessageId::from_raw("message-1")),
        state: DraftState::Scheduled {
            at: Timestamp::from_millis(1_700_000_010_000),
        },
        updated_at: Timestamp::from_millis(1_700_000_000_000),
    }
}

/// Au moins une requête par variante de `ComposeRequest`
pub fn requests() -> Vec<Named<Request>> {
    let items = [
        creation_requests(),
        editing_requests(),
        server_copy_requests(),
        sending_requests(),
    ]
    .concat();
    let ops: Vec<&str> = items.iter().map(op_name).collect();
    let names = numbered("compose", &ops);
    let wrap = |request| Request::Compose(request);
    names.into_iter().zip(items.into_iter().map(wrap)).collect()
}

/// Réponses d'exemple : un brouillon, une liste de brouillons, la boîte d'envoi
pub fn responses() -> Vec<Named<Response>> {
    let entry = OutboxEntry {
        action: ActionId::from_raw("action-1"),
        draft: DraftId::from_raw("draft-1"),
        state: ActionState::Uncertain,
        scheduled_for: Timestamp::from_millis(1_700_000_010_000),
        subject: "Bonjour".to_string(),
        last_error: None,
    };
    let draft = WriteResponse::Draft(Box::new(sample_draft()));
    let drafts = WriteResponse::Drafts(vec![sample_draft()]);
    vec![
        ("write.draft".to_string(), Response::Write(draft)),
        ("write.drafts".to_string(), Response::Write(drafts)),
        (
            "write.outbox".to_string(),
            Response::Write(WriteResponse::Outbox(vec![entry])),
        ),
    ]
}

/// Nom de l'opération, par `match` exhaustif
pub fn op_name(request: &ComposeRequest) -> &'static str {
    match request {
        ComposeRequest::CreateDraft { .. } => "create_draft",
        ComposeRequest::UpdateDraft { .. } => "update_draft",
        ComposeRequest::AttachFile { .. } => "attach_file",
        ComposeRequest::RemoveAttachment { .. } => "remove_attachment",
        ComposeRequest::DeleteDraft { .. } => "delete_draft",
        ComposeRequest::SaveDraftCopy { .. } => "save_draft_copy",
        ComposeRequest::GetDraft { .. } => "get_draft",
        ComposeRequest::ListDrafts { .. } => "list_drafts",
        ComposeRequest::SendDraft { .. } => "send_draft",
        ComposeRequest::CancelSend { .. } => "cancel_send",
        ComposeRequest::ListOutbox { .. } => "list_outbox",
        ComposeRequest::ResolveUncertainSend { .. } => "resolve_uncertain_send",
    }
}
