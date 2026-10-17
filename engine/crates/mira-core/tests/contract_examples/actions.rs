//! Exemples du domaine « actions » : application en lot, annulation, écriture sur les boîtes.

use mira_core::api::{ActionRequest, Request, Response, TargetSelection, WriteResponse};
use mira_core::reading::ListQuery;
use mira_vocabulary::{
    AccountId, ActionId, ActionKind, IdempotencyKey, MailboxId, MessageId, StandardView, ViewRef,
};

use super::{Named, numbered};

/// Clé d'idempotence des exemples
fn key() -> IdempotencyKey {
    IdempotencyKey("key-1".to_string())
}

/// Application à une liste de messages, application au résultat d'une requête, annulation
fn message_requests() -> Vec<ActionRequest> {
    let messages = TargetSelection::Messages {
        messages: vec![MessageId::from_raw("message-1")],
    };
    let view = ViewRef::Standard {
        view: StandardView::Unread,
    };
    let query = TargetSelection::Query {
        query: ListQuery::of(view),
    };
    let apply = |kind, targets| ActionRequest::ApplyAction {
        kind,
        targets,
        idempotency_key: key(),
    };
    vec![
        apply(ActionKind::MarkRead { read: true }, messages),
        apply(ActionKind::Archive, query),
        ActionRequest::UndoAction {
            action: ActionId::from_raw("action-1"),
        },
    ]
}

/// Création, renommage, suppression et vidage d'une boîte
fn mailbox_requests() -> Vec<ActionRequest> {
    let mailbox = MailboxId::from_raw("mailbox-1");
    vec![
        ActionRequest::CreateMailbox {
            account: AccountId::from_raw("account-1"),
            name: "Factures".to_string(),
            parent: None,
            idempotency_key: key(),
        },
        ActionRequest::RenameMailbox {
            mailbox: mailbox.clone(),
            name: "Archives 2026".to_string(),
            idempotency_key: key(),
        },
        ActionRequest::DeleteMailbox {
            mailbox: mailbox.clone(),
            idempotency_key: key(),
        },
        ActionRequest::EmptyMailbox {
            mailbox,
            idempotency_key: key(),
        },
    ]
}

/// Au moins une requête par variante de `ActionRequest`
pub fn requests() -> Vec<Named<Request>> {
    let items = [message_requests(), mailbox_requests()].concat();
    let ops: Vec<&str> = items.iter().map(op_name).collect();
    let names = numbered("action", &ops);
    let wrap = |request| Request::Action(request);
    names.into_iter().zip(items.into_iter().map(wrap)).collect()
}

/// Réponses d'exemple du domaine : une action, un lot d'actions
pub fn responses() -> Vec<Named<Response>> {
    let one = WriteResponse::Action(ActionId::from_raw("action-1"));
    let ids = vec![
        ActionId::from_raw("action-1"),
        ActionId::from_raw("action-2"),
    ];
    vec![
        ("write.action".to_string(), Response::Write(one)),
        (
            "write.actions".to_string(),
            Response::Write(WriteResponse::Actions(ids)),
        ),
    ]
}

/// Nom de l'opération, par `match` exhaustif : une nouvelle variante empêche de compiler tant qu'elle n'a pas d'exemple
pub fn op_name(request: &ActionRequest) -> &'static str {
    match request {
        ActionRequest::ApplyAction { .. } => "apply_action",
        ActionRequest::UndoAction { .. } => "undo_action",
        ActionRequest::CreateMailbox { .. } => "create_mailbox",
        ActionRequest::RenameMailbox { .. } => "rename_mailbox",
        ActionRequest::DeleteMailbox { .. } => "delete_mailbox",
        ActionRequest::EmptyMailbox { .. } => "empty_mailbox",
    }
}
