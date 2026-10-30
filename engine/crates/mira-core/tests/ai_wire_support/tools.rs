//! Outil et conversation avec appels d'outils, partagés par les tests d'encodage des deux formats.
#![allow(dead_code)]

use mira_vocabulary::{
    AiProviderDescriptor, ChatMessage, ChatRequest, ChatRole, MaxTokensField, ToolCall,
    ToolOutcome, ToolSpec, WireFormat,
};

/// Outil de recherche avec un schéma d'arguments à un champ.
pub fn search_tool() -> ToolSpec {
    ToolSpec {
        name: "search_messages".to_string(),
        description: "Cherche dans la boîte.".to_string(),
        parameters_schema:
            r#"{"type":"object","properties":{"query":{"type":"string"}},"required":["query"]}"#
                .to_string(),
    }
}

/// Appel d'outil dont les arguments sont donnés tels quels.
pub fn call(id: &str, name: &str, arguments: &str) -> ToolCall {
    ToolCall {
        id: id.to_string(),
        name: name.to_string(),
        arguments: arguments.to_string(),
    }
}

/// Message de résultat d'un appel.
pub fn result(call_id: &str, text: &str, is_error: bool) -> ChatMessage {
    let mut message = ChatMessage::plain(ChatRole::Tool, String::new());
    message.tool_result = Some(ToolOutcome {
        call_id: call_id.to_string(),
        text: text.to_string(),
        is_error,
    });
    message
}

/// Message de l'assistant qui annonce du texte puis les appels donnés.
pub fn assistant_with(text: &str, calls: Vec<ToolCall>) -> ChatMessage {
    let mut message = ChatMessage::plain(ChatRole::Assistant, text.to_string());
    message.tool_calls = calls;
    message
}

/// Conversation d'un tour d'outils : question, deux appels, deux résultats dont un en erreur.
pub fn tool_turn(text: &str) -> ChatRequest {
    let calls = vec![
        call("c1", "search_messages", r#"{"query":"devis"}"#),
        call("c2", "get_thread", r#"{"thread":"t1"}"#),
    ];
    let mut request = super::request();
    request.system = None;
    request.messages = vec![
        ChatMessage::plain(ChatRole::User, "Trouve le devis".to_string()),
        assistant_with(text, calls),
        result("c1", "r1", false),
        result("c2", "r2", true),
    ];
    request
}

/// Descripteur parlant le protocole Messages.
pub fn messages_descriptor() -> AiProviderDescriptor {
    let mut descriptor = super::descriptor(MaxTokensField::MaxTokens);
    descriptor.format = WireFormat::AnthropicMessages;
    descriptor
}
