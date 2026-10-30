//! Connexion à un service d'IA : jamais de clé ici, seulement l'indicateur de sa présence.

use serde::{Deserialize, Serialize};

use super::content::ContentKind;
use super::masking::MaskingSettings;
use super::model::AiModelList;
use super::provider::AiProviderId;
use crate::ids::kinds::AiConnectionId;
use crate::time::Timestamp;

/// Nature du transport d'une connexion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum AiConnectionKind {
    Http,
}

/// Connexion configurée ; la clé vit dans le trousseau, `has_key` dit seulement qu'elle existe.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiConnection {
    pub id: AiConnectionId,
    pub provider: AiProviderId,
    pub label: String,
    pub base_url: Option<String>,
    pub default_model: Option<String>,
    pub is_default: bool,
    pub has_key: bool,
    pub consent_at: Option<Timestamp>,
    pub models: Option<AiModelList>,
    pub allowed_kinds: Vec<ContentKind>,
    pub masking: MaskingSettings,
}
