//! Fiche du fournisseur d'IA Anthropic : adresse, authentification, format de dialogue.

use mira_vocabulary::{
    AiProviderDescriptor, AiProviderId, AiQuirks, AuthScheme, MaxTokensField, WireFormat,
};

/// Identifiant du fournisseur dans le catalogue.
pub const PROVIDER_ID: &str = "anthropic";
/// Version du protocole Messages envoyée dans l'en-tête `anthropic-version`.
pub const API_VERSION: &str = "2023-06-01";

/// Construit la fiche statique du fournisseur.
pub fn descriptor() -> AiProviderDescriptor {
    AiProviderDescriptor {
        id: AiProviderId(PROVIDER_ID.to_string()),
        display_name: "Anthropic".to_string(),
        base_url: "https://api.anthropic.com/v1".to_string(),
        auth: AuthScheme::XApiKey,
        format: WireFormat::AnthropicMessages,
        models_path: "/models".to_string(),
        model_exclusions: vec![],
        static_headers: vec![("anthropic-version".to_string(), API_VERSION.to_string())],
        quirks: AiQuirks {
            max_tokens_field: MaxTokensField::MaxTokens,
        },
        base_url_editable: false,
        supports_tools: true,
    }
}
