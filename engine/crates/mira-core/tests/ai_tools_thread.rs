#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Outils `get_thread` et `list_mailboxes` : fil ordonné, budget partagé, boîtes avec non lus.

mod ai_tools_support;
mod support;
mod support_api;
mod support_mirror;
mod support_read;

use ai_tools_support::{call, index_bodies};
use mira_core::ai::tools::run;
use mira_vocabulary::{ContentKind, ToolState};
use serde_json::json;
use support_read::seed::link_threads;
use support_read::world::start_world;

/// Le fil rend ses messages avec leurs corps, et ses identifiants comme sources
#[tokio::test]
async fn get_thread_returns_its_messages_with_bodies() {
    let world = start_world().await;
    index_bodies(&world, 3, "texte du fil");
    link_threads(&world.mirror, &world.account);
    let args = json!({"thread": "t-r1"});
    let found = run(&world.fixture.engine, &call("get_thread", args)).await;
    assert_eq!(found.state, ToolState::Ok);
    assert_eq!(found.kind, ContentKind::MailMessage);
    assert_eq!(found.sources.len(), 2);
    assert!(found.text.contains("id=m1") && found.text.contains("id=m2"));
    assert!(found.text.contains("texte du fil"));
}

/// Les corps du fil se partagent la borne de 8 000 caractères ; les messages omis sont comptés
#[tokio::test]
async fn thread_bodies_share_the_budget() {
    let world = start_world().await;
    index_bodies(&world, 3, &"y".repeat(20_000));
    link_threads(&world.mirror, &world.account);
    let args = json!({"thread": "t-r1"});
    let found = run(&world.fixture.engine, &call("get_thread", args)).await;
    assert!(found.text.chars().count() <= 8_000);
    assert_eq!(found.sources.len(), 1);
    assert!(found.text.contains("1 message(s) suivant(s) omis"));
}

/// Un fil inconnu est une erreur d'outil
#[tokio::test]
async fn unknown_thread_is_a_tool_error() {
    let world = start_world().await;
    let found = run(
        &world.fixture.engine,
        &call("get_thread", json!({"thread": "absent"})),
    )
    .await;
    assert_eq!(found.state, ToolState::Error);
}

/// Les boîtes de chaque compte sont listées avec leur identifiant et leurs non lus
#[tokio::test]
async fn list_mailboxes_lists_accounts_and_unread() {
    let world = start_world().await;
    let found = run(&world.fixture.engine, &call("list_mailboxes", json!({}))).await;
    assert_eq!(found.state, ToolState::Ok);
    assert_eq!(found.kind, ContentKind::ToolResult);
    assert!(
        found
            .text
            .contains(&format!("compte id={}", world.account.as_str()))
    );
    assert!(found.text.contains("boîte id=box-1"));
    assert!(found.text.contains("non_lus="));
    assert!(found.results >= 1);
}
