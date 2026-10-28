//! Requêtes du domaine « libellés système » (spec R4) : poses détectées ou choisies, choix manuel.

use mira_vocabulary::{SystemLabelKind, ThreadId};
use serde::{Deserialize, Serialize};

/// Opérations sur les libellés système ; la détection est faite par le moteur, seul le choix manuel s'écrit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum SystemLabelRequest {
    ListSystemLabelAssignments,
    SetSystemLabel {
        kind: SystemLabelKind,
        threads: Vec<ThreadId>,
        on: bool,
    },
}
