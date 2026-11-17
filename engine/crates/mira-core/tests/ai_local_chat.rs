#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Une discussion vers le modèle local lance le serveur, le réutilise et ne touche pas au trousseau.

use std::time::Duration;

use mira_core::api::ChatStart;
use mira_vocabulary::ChatDelta;

mod local_chat_support;
mod local_support;
mod support_api;
use local_chat_support::fixture::LocalFixture;
use local_chat_support::read::{answer_of, collect, start_for};

/// Lit une discussion complète
async fn run(fixture: &LocalFixture, start: ChatStart) -> Vec<ChatDelta> {
    let mut stream = fixture.engine.chat_stream(start).await.unwrap();
    collect(&mut stream).await
}

/// La première discussion lance le faux serveur et reçoit sa réponse, sans consentement ni clé
#[tokio::test]
async fn chat_to_a_local_connection_launches_the_server_and_streams_the_answer() {
    let fixture = LocalFixture::start(300);
    fixture.add_connection(Some("mini"));
    let deltas = run(&fixture, start_for("bonjour", false)).await;
    assert_eq!(answer_of(&deltas), "ok");
    assert_eq!(fixture.starts(), 1);
}

/// La seconde discussion réutilise le serveur déjà lancé
#[tokio::test]
async fn second_chat_reuses_the_running_server() {
    let fixture = LocalFixture::start(300);
    fixture.add_connection(Some("mini"));
    run(&fixture, start_for("un", false)).await;
    run(&fixture, start_for("deux", false)).await;
    assert_eq!(fixture.starts(), 1);
    assert_eq!(fixture.entries("chat").len(), 2);
}

/// Après l'arrêt sur inactivité, la discussion suivante relance un serveur
#[tokio::test]
async fn chat_after_idle_stop_relaunches() {
    let fixture = LocalFixture::start(1);
    fixture.add_connection(Some("mini"));
    run(&fixture, start_for("un", false)).await;
    tokio::time::sleep(Duration::from_millis(2_500)).await;
    run(&fixture, start_for("deux", false)).await;
    assert_eq!(fixture.starts(), 2);
}

/// Sans fichier de modèle installé, l'erreur dit que le modèle manque et rien n'est lancé
#[tokio::test]
async fn chat_without_an_installed_model_gives_a_clear_error() {
    let fixture = LocalFixture::start(300);
    fixture.add_connection(Some("mini"));
    std::fs::remove_file(fixture.root.join("models").join("mini.bin")).unwrap();
    let error = fixture
        .engine
        .chat_stream(start_for("x", false))
        .await
        .unwrap_err();
    assert_eq!(error.code, "incompatible_state.ai_model_missing");
    assert_eq!(fixture.starts(), 0);
}

/// Le palier sans outils n'annonce aucun outil, le palier avancé en annonce
#[tokio::test]
async fn tools_are_announced_only_for_the_model_that_supports_them() {
    let fixture = LocalFixture::start(300);
    fixture.add_connection(Some("mini"));
    let mut stream = fixture
        .engine
        .chat_stream_with_tools(start_for("x", true))
        .await
        .unwrap();
    collect(&mut stream).await;
    let mut big = start_for("y", true);
    big.request.model = Some("big".to_string());
    let mut stream = fixture.engine.chat_stream_with_tools(big).await.unwrap();
    collect(&mut stream).await;
    let bodies = fixture.entries("chat");
    assert!(!bodies[0]["body"].as_str().unwrap().contains("\"tools\""));
    assert!(bodies[1]["body"].as_str().unwrap().contains("\"tools\""));
}
