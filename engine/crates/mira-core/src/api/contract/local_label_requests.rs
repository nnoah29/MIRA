//! Requêtes du domaine « libellés locaux » (spec R2 §5) : libellés, poses sur les conversations, reprise.

use mira_vocabulary::{IdempotencyKey, LocalLabelId, ThreadId};
use serde::{Deserialize, Serialize};

/// Un libellé de l'app à reprendre, avec les conversations qui le portent (fil, sinon message).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportedLabel {
    pub name: String,
    pub color: u32,
    pub conversations: Vec<String>,
}

/// Opérations sur les libellés locaux ; la création attribue l'identifiant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum LocalLabelRequest {
    ListLocalLabels,
    CreateLocalLabel {
        name: String,
        color: Option<u32>,
        idempotency_key: IdempotencyKey,
    },
    RenameLocalLabel {
        label: LocalLabelId,
        name: String,
    },
    RecolorLocalLabel {
        label: LocalLabelId,
        color: u32,
    },
    DeleteLocalLabel {
        label: LocalLabelId,
    },
    SetLocalLabel {
        label: LocalLabelId,
        threads: Vec<ThreadId>,
        on: bool,
        idempotency_key: IdempotencyKey,
    },
    ListLocalLabelAssignments,
    ImportLocalLabels {
        labels: Vec<ImportedLabel>,
        idempotency_key: IdempotencyKey,
    },
}
