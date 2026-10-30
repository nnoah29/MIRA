#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Outil `list_messages` : vues, boîte obligatoire, sources, garde de lecture seule.

mod ai_tools_support;
mod support;
mod support_api;
mod support_mirror;
mod support_read;

use ai_tools_support::call;
use mira_core::ai::tools::{is_read, run};
use mira_core::api::{MailboxRequest, Request};
use mira_vocabulary::{AccountId, ToolState};
use serde_json::json;
use support_read::world::start_world;

/// La vue unifiée liste tous les messages et donne leurs identifiants comme sources
#[tokio::test]
async fn unified_view_lists_all_with_sources() {
    let world = start_world().await;
    world.seed_messages(3);
    let found = run(&world.fixture.engine, &call("list_messages", json!({}))).await;
    assert_eq!(found.state, ToolState::Ok);
    assert_eq!(found.results, 3);
    assert_eq!(found.sources.len(), 3);
    assert!(found.sources.contains(&"m1".to_string()));
}

/// La vue des non lus liste les messages non lus, celle des suivis aucun s'il n'y en a pas
#[tokio::test]
async fn unread_and_flagged_views_follow_flags() {
    let world = start_world().await;
    world.seed_messages(2);
    let engine = &world.fixture.engine;
    let unread = run(engine, &call("list_messages", json!({"view": "unread"}))).await;
    assert_eq!(unread.results, 2);
    let flagged = run(engine, &call("list_messages", json!({"view": "flagged"}))).await;
    assert_eq!(flagged.results, 0);
}

/// La vue d'une boîte liste ses messages
#[tokio::test]
async fn mailbox_view_lists_the_mailbox() {
    let world = start_world().await;
    world.seed_messages(2);
    let args = json!({"view": "mailbox", "mailbox": world.mailbox.as_str()});
    let found = run(&world.fixture.engine, &call("list_messages", args)).await;
    assert_eq!(found.results, 2);
}

/// La vue d'une boîte sans identifiant, ou une vue inconnue, est une erreur d'outil
#[tokio::test]
async fn mailbox_view_requires_mailbox_and_unknown_view_fails() {
    let world = start_world().await;
    let engine = &world.fixture.engine;
    let missing = run(engine, &call("list_messages", json!({"view": "mailbox"}))).await;
    assert_eq!(missing.state, ToolState::Error);
    let unknown = run(engine, &call("list_messages", json!({"view": "corbeille"}))).await;
    assert_eq!(unknown.state, ToolState::Error);
}

/// Un outil inconnu est une erreur d'outil
#[tokio::test]
async fn unknown_tool_is_a_tool_error() {
    let world = start_world().await;
    let found = run(&world.fixture.engine, &call("delete_everything", json!({}))).await;
    assert_eq!(found.state, ToolState::Error);
}

/// La garde refuse toute requête qui n'est pas une lecture
#[test]
fn guard_refuses_non_read_requests() {
    let write = Request::Mailbox(MailboxRequest::SetSyncedMailboxes {
        account: AccountId::from_raw("a"),
        mailboxes: vec![],
    });
    assert!(!is_read(&write));
    let read = Request::Mailbox(MailboxRequest::ListMailboxes {
        account: AccountId::from_raw("a"),
    });
    assert!(is_read(&read));
}
