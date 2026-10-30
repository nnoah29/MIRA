#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Fins de flux : première raison d'arrêt gardée, fin sans arrêt décodé, événement d'erreur du service.

mod ai_chat_support;
mod ai_models_support;
mod support_api;

use ai_chat_support::{add_chat_connection, chat_start, collect};
use ai_models_support::{ModelsFixture, TestProvider};
use mira_testkit::ai::Behavior;
use mira_vocabulary::error::codes::server;
use mira_vocabulary::{ChatDelta, StopReason, WireFormat};
use support_api::leaks::events_text;

/// Lance une génération dont le service répond par ces morceaux, et rend les deltas et le journal
async fn run_with(format: WireFormat, chunks: &[&str]) -> (Vec<ChatDelta>, String) {
    let mut provider = TestProvider::new("test-provider");
    provider.format = format;
    let fixture = ModelsFixture::start(provider).await;
    let base = Some(fixture.server.base_url());
    let connection = add_chat_connection(&fixture, "conn-1", base, Some("m"));
    let chunks = chunks
        .iter()
        .map(|chunk| chunk.as_bytes().to_vec())
        .collect();
    fixture.server.queue(Behavior::Stream {
        chunks,
        chunk_delay_ms: 0,
    });
    let mut stream = fixture
        .engine
        .chat_stream(chat_start(&connection, "Salut"))
        .await
        .unwrap();
    let deltas = collect(&mut stream).await;
    (deltas, events_text(&fixture.engine).await)
}

/// Delta de fin avec cette raison
fn done(reason: StopReason) -> ChatDelta {
    ChatDelta::Done { reason }
}

/// Format Messages : la raison `max_tokens` du message_delta n'est pas écrasée par le message_stop
#[tokio::test]
async fn first_stop_reason_is_kept() {
    let chunks = [
        "event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"delta\":{\"type\":\"text_delta\",\"text\":\"Bon\"}}\n\n",
        "event: message_delta\ndata: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"max_tokens\"},\"usage\":{\"output_tokens\":9}}\n\n",
        "event: message_stop\ndata: {\"type\":\"message_stop\"}\n\n",
    ];
    let (deltas, events) = run_with(WireFormat::AnthropicMessages, &chunks).await;
    let text = ChatDelta::Text {
        text: "Bon".to_string(),
    };
    let usage = ChatDelta::Usage {
        input_tokens: 0,
        output_tokens: 9,
    };
    assert_eq!(deltas, vec![text, usage, done(StopReason::MaxTokens)]);
    assert!(events.contains("max_tokens"));
}

/// Fin de flux sans raison d'arrêt mais avec du texte : fin normale
#[tokio::test]
async fn end_without_stop_after_text_is_end_turn() {
    let chunks = ["data: {\"choices\":[{\"delta\":{\"content\":\"Bon\"}}]}\n\n"];
    let (deltas, _) = run_with(WireFormat::OpenAiChat, &chunks).await;
    let text = ChatDelta::Text {
        text: "Bon".to_string(),
    };
    assert_eq!(deltas, vec![text, done(StopReason::EndTurn)]);
}

/// Fin de flux sans rien de décodé (événement d'erreur avalé) : réponse inattendue
#[tokio::test]
async fn end_without_anything_is_bad_response() {
    let chunks = [
        "data: {\"error\":{\"message\":\"surcharge\"}}\n\n",
        "data: [DONE]\n\n",
    ];
    let (deltas, events) = run_with(WireFormat::OpenAiChat, &chunks).await;
    let failed = StopReason::Failed {
        code: server::AI_BAD_RESPONSE.0.to_string(),
        retry_after_secs: None,
    };
    assert_eq!(deltas, vec![done(failed)]);
    assert!(events.contains(server::AI_BAD_RESPONSE.0) && !events.contains("surcharge"));
}

/// Événement `error` du format Messages en plein flux : service indisponible, texte reçu conservé
#[tokio::test]
async fn error_event_mid_stream_is_unavailable() {
    let chunks = [
        "event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"delta\":{\"type\":\"text_delta\",\"text\":\"Bon\"}}\n\n",
        "event: error\ndata: {\"type\":\"error\",\"error\":{\"type\":\"overloaded_error\"}}\n\n",
    ];
    let (deltas, _) = run_with(WireFormat::AnthropicMessages, &chunks).await;
    let text = ChatDelta::Text {
        text: "Bon".to_string(),
    };
    let failed = StopReason::Failed {
        code: server::AI_UNAVAILABLE.0.to_string(),
        retry_after_secs: None,
    };
    assert_eq!(deltas, vec![text, done(failed)]);
}
