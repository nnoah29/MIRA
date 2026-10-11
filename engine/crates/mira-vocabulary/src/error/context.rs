//! Identifiants qui situent une erreur (jamais de contenu ni de secret).

use serde::{Deserialize, Serialize};

use crate::ids::kinds::{AccountId, ActionId, MailboxId, MessageId, OperationId};

/// Contexte d'une erreur : ce qui était visé quand elle est survenue.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorContext {
    pub account: Option<AccountId>,
    pub message: Option<MessageId>,
    pub mailbox: Option<MailboxId>,
    pub operation: Option<OperationId>,
    pub action: Option<ActionId>,
    pub diagnostic_id: Option<String>,
}
