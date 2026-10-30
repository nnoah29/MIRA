//! Fiche du fournisseur d'IA Google Gemini : adresse, authentification, format de dialogue.

use mira_vocabulary::{
    AiProviderDescriptor, AiProviderId, AiQuirks, AuthScheme, MaxTokensField, WireFormat,
};

/// Identifiant du fournisseur dans le catalogue.
pub const PROVIDER_ID: &str = "google";

/// Construit la fiche statique du fournisseur.
pub fn descriptor() -> AiProviderDescriptor {
    AiProviderDescriptor {
        id: AiProviderId(PROVIDER_ID.to_string()),
        display_name: "Google Gemini".to_string(),
        base_url: "https://generativelanguage.googleapis.com/v1beta/openai".to_string(),
        auth: AuthScheme::Bearer,
        format: WireFormat::OpenAiChat,
        models_path: "/models".to_string(),
        model_exclusions: vec![
            "embedding".to_string(),
            "tts".to_string(),
            "image".to_string(),
            "audio".to_string(),
        ],
        static_headers: Vec::new(),
        quirks: AiQuirks {
            max_tokens_field: MaxTokensField::MaxTokens,
        },
        base_url_editable: false,
        supports_tools: true,
    }
}
