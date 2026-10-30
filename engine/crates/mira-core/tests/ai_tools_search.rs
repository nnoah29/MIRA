#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Outil `search_messages` : lignes trouvées, filtres, plafond de 20, erreurs d'arguments.

mod ai_tools_support;
mod support;
mod support_api;
mod support_mirror;
mod support_read;

use ai_tools_support::{call, index_bodies};
use mira_core::ai::tools::run;
use mira_vocabulary::{ContentKind, ToolState};
use serde_json::json;
use support_read::world::start_world;

/// La recherche rend les messages trouvés avec leur identifiant, et eux seuls comme sources
#[tokio::test]
async fn search_returns_matching_lines_with_ids() {
    let world = start_world().await;
    index_bodies(&world, 3, "rien");
    let id = mira_vocabulary::MessageId::from_raw("m2");
    support_read::seed::add_body(&world.mirror, &id, "le devis de mars", None);
    let found = run(
        &world.fixture.engine,
        &call("search_messages", json!({"query": "devis"})),
    )
    .await;
    assert_eq!(found.state, ToolState::Ok);
    assert_eq!(found.kind, ContentKind::MailList);
    assert_eq!(found.sources, vec!["m2".to_string()]);
    assert_eq!(found.results, 1);
    assert!(found.text.contains("id=m2"));
    assert!(!found.text.contains("id=m1"));
}

/// Le filtre d'expéditeur restreint la recherche
#[tokio::test]
async fn search_applies_sender_filter() {
    let world = start_world().await;
    index_bodies(&world, 2, "devis");
    let engine = &world.fixture.engine;
    let none = json!({"query": "devis", "from_contains": "personne@nulle.part"});
    assert_eq!(run(engine, &call("search_messages", none)).await.results, 0);
    let bob = json!({"query": "devis", "from_contains": "bob@example.org"});
    assert_eq!(run(engine, &call("search_messages", bob)).await.results, 2);
}

/// Au plus vingt résultats par appel, avec l'indication qu'il y en a d'autres et le curseur
#[tokio::test]
async fn search_caps_at_twenty_and_reports_more() {
    let world = start_world().await;
    index_bodies(&world, 25, "devis");
    let found = run(
        &world.fixture.engine,
        &call("search_messages", json!({"query": "devis"})),
    )
    .await;
    assert_eq!(found.results, 20);
    assert!(found.text.contains("has_more: true"));
    assert!(found.text.contains("next_cursor: "));
}

/// Une date illisible est une erreur d'outil, pas une panique
#[tokio::test]
async fn invalid_date_is_a_tool_error() {
    let world = start_world().await;
    let args = json!({"query": "x", "since": "pas une date"});
    let found = run(&world.fixture.engine, &call("search_messages", args)).await;
    assert_eq!(found.state, ToolState::Error);
    assert_eq!(found.results, 0);
}

/// Une date seule borne la journée entière ; une borne récente écarte les anciens messages
#[tokio::test]
async fn date_only_bounds_are_accepted() {
    let world = start_world().await;
    index_bodies(&world, 2, "devis");
    let engine = &world.fixture.engine;
    let recent = json!({"query": "devis", "since": "2026-01-01"});
    assert_eq!(
        run(engine, &call("search_messages", recent)).await.results,
        0
    );
    let old = json!({"query": "devis", "since": "1970-01-01", "until": "1970-01-02"});
    assert_eq!(run(engine, &call("search_messages", old)).await.results, 2);
}

/// Des arguments qui ne sont pas du JSON sont une erreur d'outil
#[tokio::test]
async fn unreadable_arguments_are_a_tool_error() {
    let world = start_world().await;
    let mut bad = call("search_messages", json!({}));
    bad.arguments = "{pas du json".to_string();
    let found = run(&world.fixture.engine, &bad).await;
    assert_eq!(found.state, ToolState::Error);
}
