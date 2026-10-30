//! Fiche du fournisseur d'IA Compatible OpenAI : adresse, authentification, format de dialogue.

use mira_vocabulary::{
    AiProviderDescriptor, AiProviderId, AiQuirks, AuthScheme, MaxTokensField, WireFormat,
};

/// Identifiant du fournisseur dans le catalogue.
pub const PROVIDER_ID: &str = "generic";

/// Construit la fiche statique du fournisseur.
pub fn descriptor() -> AiProviderDescriptor {
    AiProviderDescriptor {
        id: AiProviderId(PROVIDER_ID.to_string()),
        display_name: "Compatible OpenAI".to_string(),
        base_url: "".to_string(),
        auth: AuthScheme::Bearer,
        format: WireFormat::OpenAiChat,
        models_path: "/models".to_string(),
        model_exclusions: vec![],
        static_headers: Vec::new(),
        quirks: AiQuirks {
            max_tokens_field: MaxTokensField::MaxTokens,
        },
        base_url_editable: true,
        supports_tools: true,
    }
}
