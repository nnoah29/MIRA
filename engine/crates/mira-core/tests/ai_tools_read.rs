#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Outil `get_message` : en-têtes et corps texte, borne de 8 000 caractères, corps absent, message inconnu.

mod ai_tools_support;
mod support;
mod support_api;
mod support_mirror;
mod support_read;

use ai_tools_support::call;
use mira_core::ai::tools::run;
use mira_vocabulary::{ContentKind, MessageId, ToolState};
use serde_json::json;
use support_read::seed::add_body;
use support_read::world::start_world;

/// Le message rendu porte ses en-têtes puis son corps texte
#[tokio::test]
async fn get_message_returns_headers_and_text_body() {
    let world = start_world().await;
    world.seed_messages(1);
    add_body(
        &world.mirror,
        &MessageId::from_raw("m1"),
        "Voici le devis.",
        None,
    );
    let found = run(
        &world.fixture.engine,
        &call("get_message", json!({"message": "m1"})),
    )
    .await;
    assert_eq!(found.state, ToolState::Ok);
    assert_eq!(found.kind, ContentKind::MailMessage);
    assert_eq!(found.sources, vec!["m1".to_string()]);
    assert!(found.text.contains("id=m1"));
    assert!(found.text.contains("de: Bob <bob@example.org>"));
    assert!(found.text.contains("sujet: Sujet m1"));
    assert!(found.text.contains("Voici le devis."));
}

/// Un corps énorme est coupé : le résultat entier tient en 8 000 caractères et porte la marque
#[tokio::test]
async fn get_message_body_is_capped_with_mark() {
    let world = start_world().await;
    world.seed_messages(1);
    add_body(
        &world.mirror,
        &MessageId::from_raw("m1"),
        &"x".repeat(20_000),
        None,
    );
    let found = run(
        &world.fixture.engine,
        &call("get_message", json!({"message": "m1"})),
    )
    .await;
    assert!(found.text.chars().count() <= 8_000);
    assert!(found.text.ends_with("…[tronqué]"));
}

/// Un corps vide est dit explicitement
#[tokio::test]
async fn empty_body_is_rendered_explicitly() {
    let world = start_world().await;
    world.seed_messages(1);
    add_body(&world.mirror, &MessageId::from_raw("m1"), "", None);
    let found = run(
        &world.fixture.engine,
        &call("get_message", json!({"message": "m1"})),
    )
    .await;
    assert!(found.text.contains("(corps vide)"));
}

/// Un corps absent du miroir est annoncé, sans échec
#[tokio::test]
async fn body_unavailable_offline_is_ok_with_notice() {
    let world = start_world().await;
    world.seed_messages(1);
    let found = run(
        &world.fixture.engine,
        &call("get_message", json!({"message": "m1"})),
    )
    .await;
    assert_eq!(found.state, ToolState::Ok);
    assert!(found.text.contains("corps non disponible hors ligne"));
}

/// Un message inconnu est une erreur d'outil sans source
#[tokio::test]
async fn unknown_message_id_is_a_tool_error() {
    let world = start_world().await;
    let found = run(
        &world.fixture.engine,
        &call("get_message", json!({"message": "absent"})),
    )
    .await;
    assert_eq!(found.state, ToolState::Error);
    assert!(found.sources.is_empty());
}
