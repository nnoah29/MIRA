//! Fiche du fournisseur d'IA OpenRouter : adresse, authentification, connexion par navigateur.

use mira_vocabulary::{
    AiKeyLogin, AiProviderDescriptor, AiProviderId, AiQuirks, AuthScheme, MaxTokensField,
    WireFormat,
};

/// Identifiant du fournisseur dans le catalogue.
pub const PROVIDER_ID: &str = "openrouter";

/// Construit la fiche statique du fournisseur.
pub fn descriptor() -> AiProviderDescriptor {
    AiProviderDescriptor {
        id: AiProviderId(PROVIDER_ID.to_string()),
        display_name: "OpenRouter".to_string(),
        base_url: "https://openrouter.ai/api/v1".to_string(),
        auth: AuthScheme::Bearer,
        format: WireFormat::OpenAiChat,
        models_path: "/models".to_string(),
        model_exclusions: vec!["embedding".to_string()],
        static_headers: Vec::new(),
        quirks: AiQuirks {
            max_tokens_field: MaxTokensField::MaxTokens,
        },
        base_url_editable: false,
        supports_tools: true,
        key_login: Some(key_login()),
    }
}

/// Adresses de la connexion OAuth PKCE qui rend une clé d'API.
fn key_login() -> AiKeyLogin {
    AiKeyLogin {
        authorize_url: "https://openrouter.ai/auth".to_string(),
        exchange_url: "https://openrouter.ai/api/v1/auth/keys".to_string(),
    }
}
