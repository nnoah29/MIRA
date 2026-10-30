//! Fiche du fournisseur d'IA Groq : adresse, authentification, format de dialogue.

use mira_vocabulary::{
    AiProviderDescriptor, AiProviderId, AiQuirks, AuthScheme, MaxTokensField, WireFormat,
};

/// Identifiant du fournisseur dans le catalogue.
pub const PROVIDER_ID: &str = "groq";

/// Construit la fiche statique du fournisseur.
pub fn descriptor() -> AiProviderDescriptor {
    AiProviderDescriptor {
        id: AiProviderId(PROVIDER_ID.to_string()),
        display_name: "Groq".to_string(),
        base_url: "https://api.groq.com/openai/v1".to_string(),
        auth: AuthScheme::Bearer,
        format: WireFormat::OpenAiChat,
        models_path: "/models".to_string(),
        model_exclusions: vec!["whisper".to_string(), "tts".to_string()],
        static_headers: Vec::new(),
        quirks: AiQuirks {
            max_tokens_field: MaxTokensField::MaxCompletionTokens,
        },
        base_url_editable: false,
        supports_tools: true,
    }
}
