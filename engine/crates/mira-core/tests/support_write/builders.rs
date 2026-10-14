//! Constructeurs de requêtes d'écriture : application d'action, annulation, envoi, clé d'idempotence.

use mira_core::api::{ActionRequest, ComposeRequest, Request, TargetSelection};
use mira_vocabulary::{ActionId, ActionKind, DraftId, IdempotencyKey, MessageId};

/// Requête d'application d'une action à une liste de messages
pub fn apply_request(kind: ActionKind, ids: &[&str], key: &str) -> Request {
    let messages = ids.iter().map(|id| MessageId::from_raw(*id)).collect();
    Request::Action(ActionRequest::ApplyAction {
        kind,
        targets: TargetSelection::Messages { messages },
        idempotency_key: IdempotencyKey(key.to_string()),
    })
}

/// Requête d'annulation d'une action récente
pub fn undo_request(action: &ActionId) -> Request {
    Request::Action(ActionRequest::UndoAction {
        action: action.clone(),
    })
}

/// Requête d'envoi d'un brouillon
pub fn send_request(draft: &DraftId, key: &str) -> Request {
    Request::Compose(ComposeRequest::SendDraft {
        draft: draft.clone(),
        at: None,
        idempotency_key: IdempotencyKey(key.to_string()),
    })
}

/// Clé d'idempotence de test
pub fn key(text: &str) -> IdempotencyKey {
    IdempotencyKey(text.to_string())
}

/// Décision de l'utilisateur sur un envoi incertain
pub fn resolve_request(action: &ActionId, resend: bool) -> Request {
    Request::Compose(ComposeRequest::ResolveUncertainSend {
        action: action.clone(),
        resend,
    })
}

/// Annulation d'un envoi encore dans son délai
pub fn cancel_send_request(action: &ActionId) -> Request {
    Request::Compose(ComposeRequest::CancelSend {
        action: action.clone(),
    })
}
