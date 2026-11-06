//! Demandes de génération d'IA en JSON et configuration d'un moteur muni des six fournisseurs d'IA.

use mira_assembly::api::{ChatStart, EngineConfig};
use mira_assembly::build_ai_registry;
use mira_assembly::mira_vocabulary::{
    AiConnectionId, ChatMessage, ChatMode, ChatRequest, ChatRole,
};

use super::mail_fixture::config;

/// Configuration de test avec les fournisseurs d'IA, sans faux serveur de courrier.
pub fn ai_config(home: &std::path::Path) -> EngineConfig {
    let mut config = config(home, 1, 1);
    config.ai_registry = build_ai_registry();
    config
}

/// Demande JSON d'une discussion d'un message sur cette connexion (`None` : connexion par défaut), sans outils.
pub fn chat_json(connection: Option<&str>, text: &str) -> String {
    chat_json_with(connection, text, false, ChatMode::Assistant)
}

/// Comme `chat_json`, en permettant les outils en lecture seule.
pub fn chat_json_tools(connection: Option<&str>, text: &str) -> String {
    chat_json_with(connection, text, true, ChatMode::Assistant)
}

/// Comme `chat_json_tools`, dans le mode de génération donné (assistant ou composeur).
pub fn chat_json_mode(connection: Option<&str>, text: &str, mode: ChatMode) -> String {
    chat_json_with(connection, text, true, mode)
}

/// Demande JSON d'une discussion d'un message, avec ou sans outils.
fn chat_json_with(
    connection: Option<&str>,
    text: &str,
    allow_tools: bool,
    mode: ChatMode,
) -> String {
    let message = ChatMessage::plain(ChatRole::User, text.to_string());
    let request = ChatRequest {
        model: None,
        system: None,
        messages: vec![message],
        max_output_tokens: None,
        temperature: None,
        tools: Vec::new(),
        context: Vec::new(),
    };
    let connection = connection.map(AiConnectionId::from_raw);
    let start = ChatStart {
        connection,
        request,
        allow_tools,
        mode,
    };
    serde_json::to_string(&start).unwrap()
}
