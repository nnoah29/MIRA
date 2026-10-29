#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Ni la clé ni le texte de la discussion n'apparaissent dans les événements, erreurs ou sorties `Debug`.

mod ai_chat_support;
mod ai_models_support;
mod support_api;

use ai_chat_support::{add_chat_connection, chat_start, collect};
use ai_models_support::{ModelsFixture, TestProvider};
use mira_testkit::ai::{Behavior, openai_stream};
use support_api::leaks::events_text;

const KEY: &str = "SECRET-KEY-123";

/// La clé, envoyée au service, n'apparaît dans aucun événement, erreur ni sortie `Debug`
#[tokio::test]
async fn key_absent_from_events_errors_and_debug_output() {
    let fixture = ModelsFixture::start(TestProvider::new("test-provider")).await;
    let base = Some(fixture.server.base_url());
    let connection = add_chat_connection(&fixture, "conn-1", base, Some("model-a"));
    fixture.store_key("conn-1", KEY);
    fixture.server.queue(openai_stream(&["ok"], Some((1, 1))));
    let start = chat_start(&connection, "Salut");
    let start_debug = format!("{start:?}");
    let mut stream = fixture.engine.chat_stream(start).await.unwrap();
    let stream_debug = format!("{stream:?}");
    let deltas = collect(&mut stream).await;
    let sent = fixture.server.requests()[0]
        .header("authorization")
        .map(str::to_string);
    assert_eq!(sent, Some(format!("Bearer {KEY}")));
    let missing = add_chat_connection(&fixture, "conn-2", Some(fixture.server.base_url()), None);
    fixture.store_key("conn-2", KEY);
    let error = fixture
        .engine
        .chat_stream(chat_start(&missing, "Salut"))
        .await
        .unwrap_err();
    let (stored, events) = (
        fixture.stored(&connection),
        events_text(&fixture.engine).await,
    );
    let all = format!("{start_debug} {stream_debug} {deltas:?} {error:?} {stored:?} {events}");
    assert!(!all.contains(KEY), "clé divulguée : {all}");
}

/// Un rejet de la clé ne recopie ni la clé ni le corps de la réponse dans le delta ou les événements
#[tokio::test]
async fn rejected_key_leaks_nothing() {
    let fixture = ModelsFixture::start(TestProvider::new("test-provider")).await;
    let base = Some(fixture.server.base_url());
    let connection = add_chat_connection(&fixture, "conn-1", base, Some("model-a"));
    fixture.store_key("conn-1", KEY);
    fixture.server.queue(Behavior::Status {
        code: 401,
        headers: Vec::new(),
        body: format!(r#"{{"error":"bad key {KEY} CORPS-DU-SERVICE"}}"#),
    });
    let mut stream = fixture
        .engine
        .chat_stream(chat_start(&connection, "Salut"))
        .await
        .unwrap();
    let deltas = collect(&mut stream).await;
    let all = format!("{deltas:?} {}", events_text(&fixture.engine).await);
    assert!(!all.contains(KEY) && !all.contains("CORPS-DU-SERVICE"));
}

/// Les sorties `Debug` du démarrage et du flux ne montrent ni la consigne ni les messages
#[tokio::test]
async fn debug_output_hides_prompt_texts() {
    let fixture = ModelsFixture::start(TestProvider::new("test-provider")).await;
    let base = Some(fixture.server.base_url());
    let connection = add_chat_connection(&fixture, "conn-1", base, Some("model-a"));
    fixture.server.queue(openai_stream(&["ok"], None));
    let mut start = chat_start(&connection, "TEXTE-DU-MAIL");
    start.request.system = Some("CONSIGNE-SYSTEME".to_string());
    let start_debug = format!("{start:?}");
    let stream = fixture.engine.chat_stream(start).await.unwrap();
    let all = format!("{start_debug} {stream:?}");
    assert!(
        !all.contains("TEXTE-DU-MAIL") && !all.contains("CONSIGNE-SYSTEME"),
        "{all}"
    );
    assert!(start_debug.contains("conn-1"));
}
