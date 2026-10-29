//! Fiche du fournisseur d'IA OpenAI : adresse, authentification, format de dialogue.

use mira_vocabulary::{
    AiProviderDescriptor, AiProviderId, AiQuirks, AuthScheme, MaxTokensField, WireFormat,
};

/// Identifiant du fournisseur dans le catalogue.
pub const PROVIDER_ID: &str = "openai";

/// Construit la fiche statique du fournisseur.
pub fn descriptor() -> AiProviderDescriptor {
    AiProviderDescriptor {
        id: AiProviderId(PROVIDER_ID.to_string()),
        display_name: "OpenAI".to_string(),
        base_url: "https://api.openai.com/v1".to_string(),
        auth: AuthScheme::Bearer,
        format: WireFormat::OpenAiChat,
        models_path: "/models".to_string(),
        model_exclusions: vec![
            "embedding".to_string(),
            "whisper".to_string(),
            "tts".to_string(),
            "dall-e".to_string(),
            "moderation".to_string(),
            "transcribe".to_string(),
            "image".to_string(),
            "audio".to_string(),
            "realtime".to_string(),
        ],
        static_headers: Vec::new(),
        quirks: AiQuirks {
            max_tokens_field: MaxTokensField::MaxCompletionTokens,
        },
        base_url_editable: false,
    }
}
