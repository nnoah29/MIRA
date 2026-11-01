#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Outil `list_identities` : identités d'envoi de chaque compte, pour écrire sous le bon nom.

mod ai_tools_support;
mod support;
mod support_api;
mod support_mirror;
mod support_read;

use ai_tools_support::call;
use mira_core::ai::tools::run;
use mira_core::api::{IdentityRequest, Request, Response};
use mira_vocabulary::{ContentKind, ToolState};
use serde_json::json;
use support_read::world::start_world;

/// Le résultat nomme chaque identité : identifiant, adresse, nom affiché et état par défaut
#[tokio::test]
async fn lists_identities_of_every_account() {
    let world = start_world().await;
    let found = run(&world.fixture.engine, &call("list_identities", json!({}))).await;
    assert_eq!(found.state, ToolState::Ok);
    assert_eq!(found.kind, ContentKind::ToolResult);
    assert!(found.results >= 1);
    assert!(found.text.contains(world.user()), "{}", found.text);
    assert!(found.text.contains("identité id="));
    assert!(found.text.contains("par_défaut=oui"));
    assert!(found.sources.is_empty());
}

/// Une signature très longue est coupée à 300 caractères : le résultat reste borné
#[tokio::test]
async fn long_signatures_are_cut() {
    let world = start_world().await;
    let engine = &world.fixture.engine;
    let list = Request::Identity(IdentityRequest::ListIdentities {
        account: world.account.clone(),
    });
    let Response::Identities(found) = engine.execute(list).await.unwrap() else {
        panic!("identités attendues");
    };
    let update = Request::Identity(IdentityRequest::UpdateIdentity {
        identity: found[0].id.clone(),
        display_name: None,
        signature: Some("é".repeat(2_000)),
    });
    engine.execute(update).await.unwrap();
    let shown = run(engine, &call("list_identities", json!({}))).await;
    assert!(shown.text.contains(&"é".repeat(300)));
    assert!(!shown.text.contains(&"é".repeat(301)));
}
