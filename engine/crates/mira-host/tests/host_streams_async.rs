#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Variantes asynchrones des flux sur un hôte jamais ouvert : rien d'émis, ou une seule réponse fixe.

use mira_assembly::api::ChatStart;
use mira_assembly::mira_vocabulary::{ChatMessage, ChatMode, ChatRequest, ChatRole};
use mira_host::Host;
use serde_json::Value;

const SECRET_TEXT: &str = "TEXTE-CONFIDENTIEL-42";

/// Demande de génération valide, dont le texte ne doit jamais ressortir.
fn chat_json() -> String {
    let request = ChatRequest {
        model: None,
        system: None,
        messages: vec![ChatMessage::plain(ChatRole::User, SECRET_TEXT.to_string())],
        max_output_tokens: None,
        temperature: None,
        tools: Vec::new(),
        context: Vec::new(),
    };
    let start = ChatStart {
        connection: None,
        request,
        allow_tools: false,
        mode: ChatMode::default(),
    };
    serde_json::to_string(&start).unwrap()
}

/// Sans moteur, le flux d'événements asynchrone se termine sans rien émettre.
#[tokio::test(flavor = "multi_thread")]
async fn async_events_on_a_closed_host_emit_nothing() {
    let mut lines: Vec<String> = Vec::new();
    Host::new()
        .pump_events_async(None, |line| {
            lines.push(line);
            async { true }
        })
        .await;
    assert!(lines.is_empty());
}

/// Sans moteur, le chat asynchrone émet une seule ligne d'erreur fixe, sans le texte de la demande.
#[tokio::test(flavor = "multi_thread")]
async fn async_chat_on_a_closed_host_emits_one_fixed_error_line() {
    let mut lines: Vec<String> = Vec::new();
    Host::new()
        .pump_chat_async(&chat_json(), |line| {
            lines.push(line);
            async {
                tokio::task::yield_now().await;
                true
            }
        })
        .await;
    assert_eq!(lines.len(), 1);
    let line: Value = serde_json::from_str(&lines[0]).unwrap();
    assert_eq!(line["error"]["message"], "moteur non ouvert");
    assert!(!lines[0].contains(SECRET_TEXT));
}
