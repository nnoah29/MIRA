//! Réponses du domaine « libellés système » : poses par conversation, accusé.

use mira_vocabulary::SystemLabelAssignment;
use serde::{Deserialize, Serialize};

/// Réponse à une requête du domaine libellés système.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum SystemLabelResponse {
    Assignments(Vec<SystemLabelAssignment>),
    Done,
}
