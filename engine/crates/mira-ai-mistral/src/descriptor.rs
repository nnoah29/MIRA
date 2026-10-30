//! Fiche du fournisseur d'IA Mistral AI : adresse, authentification, format de dialogue.

use mira_vocabulary::{
    AiProviderDescriptor, AiProviderId, AiQuirks, AuthScheme, MaxTokensField, WireFormat,
};

/// Identifiant du fournisseur dans le catalogue.
pub const PROVIDER_ID: &str = "mistral";

/// Construit la fiche statique du fournisseur.
pub fn descriptor() -> AiProviderDescriptor {
    AiProviderDescriptor {
        id: AiProviderId(PROVIDER_ID.to_string()),
        display_name: "Mistral AI".to_string(),
        base_url: "https://api.mistral.ai/v1".to_string(),
        auth: AuthScheme::Bearer,
        format: WireFormat::OpenAiChat,
        models_path: "/models".to_string(),
        model_exclusions: vec![
            "embed".to_string(),
            "moderation".to_string(),
            "transcribe".to_string(),
            "audio".to_string(),
        ],
        static_headers: Vec::new(),
        quirks: AiQuirks {
            max_tokens_field: MaxTokensField::MaxTokens,
        },
        base_url_editable: false,
    }
}
