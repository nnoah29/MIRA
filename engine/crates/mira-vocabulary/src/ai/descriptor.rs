//! Fiche d'un fournisseur d'IA : adresse, authentification, format de dialogue, particularités.

use serde::{Deserialize, Serialize};

use super::local_runtime::AiLocalRuntime;
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
    OpenAiResponses,
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
    #[serde(default = "default_true")]
    pub supports_tools: bool,
    #[serde(default)]
    pub key_login: Option<AiKeyLogin>,
    #[serde(default)]
    pub session_login: Option<AiSessionLogin>,
    #[serde(default)]
    pub local_runtime: Option<AiLocalRuntime>,
}

/// Adresses d'une connexion qui rend une clé d'API après autorisation dans le navigateur (OAuth PKCE).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiKeyLogin {
    pub authorize_url: String,
    pub exchange_url: String,
}

/// Connexion par navigateur qui rend une session de jetons renouvelables (partage de l'abonnement de l'utilisateur).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiSessionLogin {
    pub authorize_url: String,
    pub token_url: String,
    pub resource: String,
    pub scopes: Vec<String>,
    pub required_scope: String,
    pub agent_name: String,
}

/// Valeur par défaut d'une fiche écrite avant les outils : ils sont annoncés.
fn default_true() -> bool {
    true
}
