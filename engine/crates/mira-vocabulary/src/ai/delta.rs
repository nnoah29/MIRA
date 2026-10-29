//! Morceaux de réponse diffusés pendant une discussion, et raison de l'arrêt.

use serde::{Deserialize, Serialize};

/// Raison pour laquelle la réponse s'arrête ; un échec porte son code stable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    EndTurn,
    MaxTokens,
    Cancelled,
    Failed {
        code: String,
        retry_after_secs: Option<u64>,
    },
}

/// Un morceau de la réponse ; sérialisé avec un champ `type` en `snake_case`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChatDelta {
    Text {
        text: String,
    },
    Usage {
        input_tokens: u32,
        output_tokens: u32,
    },
    Done {
        reason: StopReason,
    },
}
