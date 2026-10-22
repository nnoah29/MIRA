//! Réponses du domaine « libellés locaux » : libellé, liste, poses, rapport de reprise.

use mira_vocabulary::{LocalLabel, LocalLabelAssignment};
use serde::{Deserialize, Serialize};

/// Bilan d'une reprise : libellés créés ou réutilisés, poses reprises, clés de conversation ignorées.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportReport {
    pub labels: u32,
    pub assignments: u32,
    pub ignored: u32,
}

/// Réponse à une requête du domaine libellés locaux.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum LocalLabelResponse {
    Label(LocalLabel),
    Labels(Vec<LocalLabel>),
    Assignments(Vec<LocalLabelAssignment>),
    Imported(ImportReport),
    Done,
}
