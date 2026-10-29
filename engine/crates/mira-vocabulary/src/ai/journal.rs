//! Ligne du journal des envois à un service d'IA : des comptes et des codes, jamais de contenu.

use serde::{Deserialize, Serialize};

use super::content::ContentKind;
use super::provider::AiProviderId;
use crate::ids::kinds::{AiConnectionId, OperationId};
use crate::time::Timestamp;

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
}

/// Ce que l'on apprend à la fin d'une génération et qui complète sa ligne.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiJournalFinish {
    pub masked: u32,
    pub input_tokens: Option<u32>,
    pub output_tokens: Option<u32>,
    pub outcome: String,
}
