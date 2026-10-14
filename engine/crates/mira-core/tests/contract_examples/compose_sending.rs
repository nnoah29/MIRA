//! Exemples d'envoi : envoi planifié, annulation, boîte d'envoi, décision sur un envoi incertain.

use mira_core::api::ComposeRequest;
use mira_vocabulary::{AccountId, ActionId, DraftId, IdempotencyKey, Timestamp};

/// Envoi, annulation d'envoi, boîte d'envoi, résolution d'un envoi incertain
pub fn sending_requests() -> Vec<ComposeRequest> {
    let action = ActionId::from_raw("action-1");
    vec![
        ComposeRequest::SendDraft {
            draft: DraftId::from_raw("draft-1"),
            at: Some(Timestamp::from_millis(1_700_000_010_000)),
            idempotency_key: IdempotencyKey("key-1".to_string()),
        },
        ComposeRequest::CancelSend {
            action: action.clone(),
        },
        ComposeRequest::ListOutbox {
            account: AccountId::from_raw("account-1"),
        },
        ComposeRequest::ResolveUncertainSend {
            action,
            resend: false,
        },
    ]
}
