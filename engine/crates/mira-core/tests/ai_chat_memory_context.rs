#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Rappel automatique : les souvenirs proches rejoignent le contexte de l'assistant, masqués et soumis au consentement.

mod ai_loop_support;
mod ai_models_support;
mod support;
mod support_ai_memory_loop;
mod support_api;
mod support_mirror;
mod support_read;

use ai_loop_support::fixture::LoopFixture;
use ai_loop_support::turns::{FORMATS, collect, shape, text_turn};
use mira_testkit::ai::Behavior;
use mira_vocabulary::ChatMode;
use support_ai_memory_loop::{enable_with_one_memory, one_vector, run_turns};

/// Corps de la requête de discussion (la dernière), en texte
fn last_body(f: &LoopFixture) -> String {
    f.bodies().last().unwrap().to_string()
}

/// Mémoire activée : le souvenir proche est rendu dans le dernier message, avec son identifiant
#[tokio::test]
async fn enabled_memory_adds_a_rendered_memory_part() {
    let format = FORMATS[1];
    let f = LoopFixture::start(format).await;
    enable_with_one_memory(&f);
    let turns = [one_vector(), text_turn(format, "Il s'appelle Mochi.", None)];
    let deltas = run_turns(&f, turns, "comment s'appelle mon chat ?").await;
    assert_eq!(shape(&deltas), ["text", "done:end_turn"]);
    let body = last_body(&f);
    assert!(
        body.contains("kind=\\\"memory\\\"") && body.contains("Mochi") && body.contains("mem-1")
    );
}

/// Mémoire désactivée : aucune requête d'embeddings et aucun contexte de mémoire
#[tokio::test]
async fn disabled_memory_adds_nothing() {
    let format = FORMATS[1];
    let f = LoopFixture::start(format).await;
    run_turns(&f, [text_turn(format, "Bonjour.", None)], "bonjour").await;
    assert_eq!(f.bodies().len(), 1);
    assert!(!last_body(&f).contains("memory"));
}

/// En mode composeur, la mémoire n'est pas consultée
#[tokio::test]
async fn composer_mode_does_not_consult_the_memory() {
    let format = FORMATS[1];
    let f = LoopFixture::start(format).await;
    enable_with_one_memory(&f);
    f.queue(text_turn(format, "Voici.", None));
    let mut start = f.start_of("écris un mail sur mon chat", true);
    start.mode = ChatMode::Composer { selection: false };
    let mut stream = f.engine.chat_stream_with_tools(start).await.unwrap();
    collect(&mut stream).await;
    assert_eq!(f.bodies().len(), 1);
}

/// Un embedder en panne ne casse pas la discussion
#[tokio::test]
async fn embedder_failure_does_not_break_the_chat() {
    let format = FORMATS[1];
    let f = LoopFixture::start(format).await;
    enable_with_one_memory(&f);
    let down = Behavior::Status {
        code: 500,
        headers: vec![],
        body: "panne".into(),
    };
    let deltas = run_turns(
        &f,
        [down, text_turn(format, "Bonjour.", None)],
        "mon chat ?",
    )
    .await;
    assert_eq!(shape(&deltas), ["text", "done:end_turn"]);
    assert!(!last_body(&f).contains("Mochi"));
}

/// La question est masquée avant d'être envoyée à l'embedder
#[tokio::test]
async fn the_query_is_masked_before_embedding() {
    let format = FORMATS[1];
    let f = LoopFixture::start(format).await;
    enable_with_one_memory(&f);
    let turns = [one_vector(), text_turn(format, "Ok.", None)];
    run_turns(
        &f,
        turns,
        "mon IBAN FR7630006000011234567890189 et mon chat",
    )
    .await;
    let embedding_request = f.bodies()[0].to_string();
    assert!(
        !embedding_request.contains("FR7630006000011234567890189"),
        "{embedding_request}"
    );
}
