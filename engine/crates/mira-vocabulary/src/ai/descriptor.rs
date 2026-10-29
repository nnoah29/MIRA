//! Fiche d'un fournisseur d'IA : adresse, authentification, format de dialogue, particularités.

use serde::{Deserialize, Serialize};

use super::provider::AiProviderId;

/// Façon dont la clé est présentée au service.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthScheme {
    Bearer,
    XApiKey,
}

/// Format des échanges de discussion avec le service.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WireFormat {
    AnthropicMessages,
    OpenAiChat,
}

/// Nom du champ qui borne la longueur de la réponse.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaxTokensField {
    #[default]
    MaxTokens,
    MaxCompletionTokens,
}

/// Écarts d'un service par rapport au format de dialogue standard.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiQuirks {
    pub max_tokens_field: MaxTokensField,
}

/// Fiche d'identité d'un fournisseur d'IA : données pures, sans entrée-sortie.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiProviderDescriptor {
    pub id: AiProviderId,
    pub display_name: String,
    pub base_url: String,
    pub auth: AuthScheme,
    pub format: WireFormat,
    pub models_path: String,
    pub model_exclusions: Vec<String>,
    pub static_headers: Vec<(String, String)>,
    pub quirks: AiQuirks,
    pub base_url_editable: bool,
}
