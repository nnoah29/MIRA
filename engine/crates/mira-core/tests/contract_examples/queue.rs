//! Exemples du domaine « file d'actions » : lister, relancer, annuler.

use mira_core::api::{QueueRequest, Request, Response, WriteResponse};
use mira_vocabulary::{
    AccountId, ActionId, ActionKind, ActionState, IdempotencyKey, MessageId, PendingAction,
    Timestamp,
};

use super::{Named, numbered};

/// Une requête par variante de `QueueRequest`
pub fn requests() -> Vec<Named<Request>> {
    let action = ActionId::from_raw("action-1");
    let items = vec![
        QueueRequest::ListActions {
            account: AccountId::from_raw("account-1"),
            states: vec![ActionState::Pending, ActionState::Failed],
        },
        QueueRequest::RetryAction {
            action: action.clone(),
        },
        QueueRequest::CancelAction { action },
    ];
    let ops: Vec<&str> = items.iter().map(op_name).collect();
    let names = numbered("queue", &ops);
    let wrap = |request| Request::Queue(request);
    names.into_iter().zip(items.into_iter().map(wrap)).collect()
}

/// Réponse d'exemple : une action en attente de la file
pub fn responses() -> Vec<Named<Response>> {
    let pending = PendingAction {
        id: ActionId::from_raw("action-1"),
        account: AccountId::from_raw("account-1"),
        kind: ActionKind::MarkRead { read: true },
        targets: vec![MessageId::from_raw("message-1")],
        idempotency_key: IdempotencyKey("key-1".to_string()),
        state: ActionState::Pending,
        attempts: 0,
        last_error: None,
        created_at: Timestamp::from_millis(1_700_000_000_000),
        not_before: Timestamp::from_millis(1_700_000_005_000),
    };
    let queued = Response::Write(WriteResponse::QueuedActions(vec![pending]));
    vec![("write.queued_actions".to_string(), queued)]
}

/// Nom de l'opération, par `match` exhaustif
pub fn op_name(request: &QueueRequest) -> &'static str {
    match request {
        QueueRequest::ListActions { .. } => "list_actions",
        QueueRequest::RetryAction { .. } => "retry_action",
        QueueRequest::CancelAction { .. } => "cancel_action",
    }
}
