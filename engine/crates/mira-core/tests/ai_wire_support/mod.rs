//! Descripteur et requête génériques partagés par les tests du format OpenAI Chat.

use mira_vocabulary::{
    AiProviderDescriptor, AiProviderId, AiQuirks, AuthScheme, ChatMessage, ChatRequest, ChatRole,
    MaxTokensField, WireFormat,
};

/// Descripteur générique d'un service parlant le protocole OpenAI Chat.
pub fn descriptor(field: MaxTokensField) -> AiProviderDescriptor {
    AiProviderDescriptor {
        id: AiProviderId("test-provider".to_string()),
        display_name: "Test".to_string(),
        base_url: "http://127.0.0.1:9/v1".to_string(),
        auth: AuthScheme::Bearer,
        format: WireFormat::OpenAiChat,
        models_path: "/models".to_string(),
        model_exclusions: Vec::new(),
        static_headers: vec![("x-test".to_string(), "1".to_string())],
        quirks: AiQuirks {
            max_tokens_field: field,
        },
        base_url_editable: false,
        supports_tools: true,
    }
}

/// Requête minimale : une consigne et un message utilisateur.
pub fn request() -> ChatRequest {
    ChatRequest {
        model: None,
        system: Some("Sois bref.".to_string()),
        messages: vec![ChatMessage::plain(ChatRole::User, "Bonjour".to_string())],
        max_output_tokens: None,
        temperature: None,
        tools: Vec::new(),
        context: Vec::new(),
    }
}

pub mod tools;
