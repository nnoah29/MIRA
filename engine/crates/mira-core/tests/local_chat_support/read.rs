#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, dead_code)]
//! Lecture d'un flux de discussion et construction des démarrages de test.

use mira_core::api::{ChatStart, ChatStream};
use mira_vocabulary::{AiConnectionId, ChatDelta, ChatMessage, ChatMode, ChatRequest, ChatRole};

/// Lit tous les deltas jusqu'à la fin du flux (10 s au plus)
pub async fn collect(stream: &mut ChatStream) -> Vec<ChatDelta> {
    let mut deltas = Vec::new();
    let read_all = async {
        while let Some(delta) = stream.next().await {
            deltas.push(delta);
        }
    };
    tokio::time::timeout(std::time::Duration::from_secs(10), read_all)
        .await
        .unwrap();
    deltas
}

/// Texte de la réponse : les deltas de texte mis bout à bout
pub fn answer_of(deltas: &[ChatDelta]) -> String {
    let texts = deltas.iter().filter_map(|delta| match delta {
        ChatDelta::Text { text } => Some(text.as_str()),
        _ => None,
    });
    texts.collect()
}

/// Démarrage d'une discussion d'un message vers la connexion `local-1`
pub fn start_for(text: &str, allow_tools: bool) -> ChatStart {
    ChatStart {
        connection: Some(AiConnectionId::from_raw("local-1")),
        request: ChatRequest {
            model: None,
            system: None,
            messages: vec![ChatMessage::plain(ChatRole::User, text.to_string())],
            max_output_tokens: None,
            temperature: None,
            tools: Vec::new(),
            context: Vec::new(),
        },
        allow_tools,
        mode: ChatMode::default(),
    }
}
