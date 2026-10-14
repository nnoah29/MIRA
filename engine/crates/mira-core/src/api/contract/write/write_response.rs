//! Réponses des écritures : identifiants d'actions, brouillons, boîte d'envoi, file d'actions.

use mira_vocabulary::{ActionId, ActionState, Draft, DraftId, PendingAction, Timestamp};
use serde::{Deserialize, Serialize};

/// Un envoi de la boîte d'envoi : planifié, en cours, incertain ou échoué.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutboxEntry {
    pub action: ActionId,
    pub draft: DraftId,
    pub state: ActionState,
    pub scheduled_for: Timestamp,
    pub subject: String,
    pub last_error: Option<String>,
}

/// Réponse à une requête d'écriture.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum WriteResponse {
    Action(ActionId),
    Actions(Vec<ActionId>),
    Draft(Box<Draft>),
    Drafts(Vec<Draft>),
    Outbox(Vec<OutboxEntry>),
    QueuedActions(Vec<PendingAction>),
}
