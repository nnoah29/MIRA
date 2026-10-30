#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Génération en flux : deltas de texte, consommation agrégée, fin unique, modèle et événements.

mod ai_chat_support;
mod ai_models_support;
mod support_api;

use ai_chat_support::{add_chat_connection, chat_start, collect, journal_kinds, loopback_stream};
use ai_models_support::{ModelsFixture, TestProvider};
use mira_testkit::ai::{Behavior, anthropic_stream, openai_stream};
use mira_vocabulary::error::codes::incompatible_state;
use mira_vocabulary::{AuthScheme, ChatDelta, EventKind, StopReason, WireFormat};

/// Suite attendue pour une réponse « Bon » + « jour » de 12 jetons d'entrée et 5 de sortie
fn expected_deltas() -> Vec<ChatDelta> {
    vec![
        ChatDelta::Text {
            text: "Bon".to_string(),
        },
        ChatDelta::Text {
            text: "jour".to_string(),
        },
        ChatDelta::Usage {
            input_tokens: 12,
            output_tokens: 5,
        },
        ChatDelta::Done {
            reason: StopReason::EndTurn,
        },
    ]
}

/// Format OpenAI : textes, puis consommation, puis une seule fin, avec le modèle de la connexion
#[tokio::test]
async fn streams_text_deltas_then_usage_then_done_openai() {
    let fixture = ModelsFixture::start(TestProvider::new("test-provider")).await;
    let base = Some(fixture.server.base_url());
    let connection = add_chat_connection(&fixture, "conn-1", base, Some("model-a"));
    fixture
        .server
        .queue(openai_stream(&["Bon", "jour"], Some((12, 5))));
    let mut stream = fixture
        .engine
        .chat_stream(chat_start(&connection, "Salut"))
        .await
        .unwrap();
    assert_eq!(collect(&mut stream).await, expected_deltas());
    let requests = fixture.server.requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].path, "/chat/completions");
    assert!(requests[0].body.contains(r#""model":"model-a""#));
}

/// Format Messages : deux fins décodées et deux relevés de consommation donnent une seule fin et un relevé agrégé
#[tokio::test]
async fn streams_text_deltas_then_usage_then_done_anthropic() {
    let mut provider = TestProvider::new("test-provider");
    provider.auth = AuthScheme::XApiKey;
    provider.format = WireFormat::AnthropicMessages;
    let fixture = ModelsFixture::start(provider).await;
    let base = Some(fixture.server.base_url());
    let connection = add_chat_connection(&fixture, "conn-1", base, Some("model-a"));
    fixture
        .server
        .queue(anthropic_stream(&["Bon", "jour"], Some((12, 5))));
    let mut stream = fixture
        .engine
        .chat_stream(chat_start(&connection, "Salut"))
        .await
        .unwrap();
    assert_eq!(collect(&mut stream).await, expected_deltas());
    assert_eq!(fixture.server.requests()[0].path, "/messages");
}

/// Le modèle demandé par la requête l'emporte sur celui de la connexion
#[tokio::test]
async fn model_from_request_wins_over_connection_default() {
    let fixture = ModelsFixture::start(TestProvider::new("test-provider")).await;
    let base = Some(fixture.server.base_url());
    let connection = add_chat_connection(&fixture, "conn-1", base, Some("model-a"));
    fixture.server.queue(openai_stream(&["ok"], None));
    let mut start = chat_start(&connection, "Salut");
    start.request.model = Some("model-b".to_string());
    let mut stream = fixture.engine.chat_stream(start).await.unwrap();
    collect(&mut stream).await;
    let body = &fixture.server.requests()[0].body;
    assert!(body.contains(r#""model":"model-b""#));
    assert!(!body.contains("model-a"));
}

/// Sans modèle ni dans la requête ni dans la connexion : erreur locale, rien n'est envoyé
#[tokio::test]
async fn missing_model_is_local_error_before_any_request() {
    let fixture = ModelsFixture::start(TestProvider::new("test-provider")).await;
    let base = Some(fixture.server.base_url());
    let connection = add_chat_connection(&fixture, "conn-1", base, None);
    fixture.server.queue(openai_stream(&["ok"], None));
    let error = fixture
        .engine
        .chat_stream(chat_start(&connection, "Salut"))
        .await
        .unwrap_err();
    assert_eq!(error.code, incompatible_state::AI_MODEL_MISSING.0);
    assert!(fixture.server.requests().is_empty());
}

/// Les événements de début et de fin portent fournisseur, modèle, jetons, durée et issue, jamais de texte
#[tokio::test]
async fn events_started_and_finished_have_no_content() {
    let fixture = ModelsFixture::start(TestProvider::new("test-provider")).await;
    let Behavior::Stream { chunks, .. } = openai_stream(&["REPONSE-SECRETE"], Some((12, 5))) else {
        panic!()
    };
    let slow = Behavior::Stream {
        chunks,
        chunk_delay_ms: 50,
    };
    let mut stream = loopback_stream(&fixture, slow, "PROMPT-SECRET").await;
    collect(&mut stream).await;
    let kinds = journal_kinds(&fixture.engine).await;
    assert!(kinds.contains(&EventKind::AiChatStarted {
        operation: stream.operation.clone(),
        provider: "test-provider".to_string(),
        model: "model-a".to_string(),
    }));
    assert!(kinds.contains(&EventKind::AiChatFinished {
        operation: stream.operation.clone(),
        input_tokens: Some(12),
        output_tokens: Some(5),
        duration_ms: 0,
        outcome: "end_turn".to_string(),
    }));
    let text = format!("{kinds:?}");
    assert!(!text.contains("PROMPT-SECRET") && !text.contains("REPONSE-SECRETE"));
}
