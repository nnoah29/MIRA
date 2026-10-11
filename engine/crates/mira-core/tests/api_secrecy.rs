#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Invariant 15 : aucun secret dans les réponses, les événements, les erreurs ni la base (P9, 7.7).

mod support_api;

use mira_core::api::{AuthRequest, EngineRequest, Request, SecretInput};
use mira_vocabulary::AuthMethodKind;
use support_api::files::files_containing;
use support_api::fixture::{self, FOLDERS_PASSWORD, Fixture};
use support_api::queries::{accounts, event_kinds, identities};
use support_api::requests::{add_request, draft};

const REPLACEMENT: &str = "remplacement-SECRET-555";
const WRONG: &str = "mauvais-SECRET-000";

/// Joue un scénario avec trois secrets et rend tout ce que les clients ont pu voir, en JSON
async fn scenario_output(f: &Fixture) -> String {
    let mut seen = String::new();
    let wrong = add_request(
        draft(fixture::FOLDERS_USER, AuthMethodKind::Password),
        WRONG,
        "bad",
    );
    let error = f.engine.execute(wrong).await.unwrap_err();
    seen.push_str(&serde_json::to_string(&error).unwrap());
    let id = f.add_folders_account("good").await;
    let store = Request::Auth(AuthRequest::StoreAppPassword {
        account: id.clone(),
        password: SecretInput::new(REPLACEMENT),
    });
    seen.push_str(&serde_json::to_string(&f.engine.execute(store).await.unwrap()).unwrap());
    let status = f
        .engine
        .execute(Request::Engine(EngineRequest::GetStatus))
        .await;
    seen.push_str(&serde_json::to_string(&status.unwrap()).unwrap());
    seen.push_str(&serde_json::to_string(&accounts(&f.engine).await).unwrap());
    seen.push_str(&serde_json::to_string(&identities(&f.engine, &id).await).unwrap());
    seen.push_str(&serde_json::to_string(&event_kinds(&f.engine).await).unwrap());
    seen
}

/// Mots de passe saisis, remplacés ou refusés : introuvables dans les sorties JSON et dans la base
#[tokio::test]
async fn invariant_15_no_secret_in_responses_events_or_database() {
    let f = fixture::start().await;
    let seen = scenario_output(&f).await;
    for secret in [FOLDERS_PASSWORD, REPLACEMENT, WRONG] {
        assert!(!seen.contains(secret), "{secret} visible dans les sorties");
    }
    let Fixture { home, engine, .. } = f;
    engine.shutdown().unwrap();
    for secret in [FOLDERS_PASSWORD, REPLACEMENT, WRONG] {
        let leaks = files_containing(home.path(), secret);
        assert!(leaks.is_empty(), "{secret} écrit dans {leaks:?}");
    }
}

/// Le `Debug` d'une requête qui porte un secret ne l'affiche pas
#[tokio::test]
async fn secret_inputs_are_masked_in_debug_output() {
    let request = add_request(
        draft(fixture::FOLDERS_USER, AuthMethodKind::Password),
        FOLDERS_PASSWORD,
        "k",
    );
    assert!(!format!("{request:?}").contains(FOLDERS_PASSWORD));
    let store = AuthRequest::StoreAppPassword {
        account: mira_vocabulary::AccountId::from_raw("a"),
        password: SecretInput::new(REPLACEMENT),
    };
    assert!(!format!("{store:?}").contains(REPLACEMENT));
}
