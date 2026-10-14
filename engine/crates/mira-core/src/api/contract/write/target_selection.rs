//! Cibles d'une action en lot : une liste de messages ou le résultat d'une requête de liste.

use mira_vocabulary::MessageId;
use serde::{Deserialize, Serialize};

use crate::reading::ListQuery;

/// Messages visés par une action ; une requête est résolue en base au moment de la soumission.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TargetSelection {
    Messages { messages: Vec<MessageId> },
    Query { query: ListQuery },
}
