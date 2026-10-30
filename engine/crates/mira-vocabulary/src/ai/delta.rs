//! Morceaux de réponse diffusés pendant une discussion, et raison de l'arrêt.

use serde::{Deserialize, Serialize};

use super::content::ContentKind;
use super::tool::ToolState;

/// Raison pour laquelle la réponse s'arrête ; un échec porte son code stable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    EndTurn,
    MaxTokens,
    ToolUse,
    MaxToolRounds,
    Cancelled,
    Failed {
        code: String,
        retry_after_secs: Option<u64>,
    },
}

/// Un morceau de la réponse ; sérialisé avec un champ `type` en `snake_case`. `ToolFragment` et `ToolUse` sont internes : jamais remis au lecteur.
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
    Masked {
        count: u32,
    },
    ToolFragment {
        index: u32,
        id: Option<String>,
        name: Option<String>,
        arguments: String,
    },
    ToolCalled {
        id: String,
        name: String,
        summary: String,
    },
    ToolFinished {
        id: String,
        state: ToolState,
        results: u32,
        sources: Vec<String>,
        denied_kind: Option<ContentKind>,
    },
    Done {
        reason: StopReason,
    },
}
