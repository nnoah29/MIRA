//! Ligne du journal des envois à un service d'IA : des comptes et des codes, jamais de contenu.

use serde::{Deserialize, Serialize};

use super::content::ContentKind;
use super::provider::AiProviderId;
use crate::ids::kinds::{AiConnectionId, OperationId};
use crate::time::Timestamp;

/// Nombre d'appels d'un outil pendant une génération ; un nom et un nombre, jamais d'argument.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiToolCount {
    pub name: String,
    pub count: u32,
}

/// Une génération telle que le journal la garde ; `outcome` vaut `running` tant qu'elle dure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiJournalEntry {
    pub id: String,
    pub at: Timestamp,
    pub operation: OperationId,
    pub connection: AiConnectionId,
    pub provider: AiProviderId,
    pub model: String,
    pub local: bool,
    pub kinds: Vec<ContentKind>,
    pub parts: u32,
    pub chars: u32,
    pub masked: u32,
    pub input_tokens: Option<u32>,
    pub output_tokens: Option<u32>,
    pub outcome: String,
    #[serde(default)]
    pub tools: Vec<AiToolCount>,
}

/// Ce que l'on apprend à la fin d'une génération et qui complète sa ligne.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiJournalFinish {
    pub masked: u32,
    pub input_tokens: Option<u32>,
    pub output_tokens: Option<u32>,
    pub outcome: String,
    #[serde(default)]
    pub tools: Vec<AiToolCount>,
    #[serde(default)]
    pub extra_kinds: Vec<ContentKind>,
    #[serde(default)]
    pub extra_chars: u32,
}
