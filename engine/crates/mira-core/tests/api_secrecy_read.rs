#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Invariant 15 pour la lecture : mot de passe et jetons absents des réponses, erreurs, événements, exports et fichiers.

mod support;
mod support_api;
mod support_mirror;
mod support_read;

use mira_core::api::{MessageRequest, Request, SearchRequest};
use mira_core::operations::OperationStatus;
use mira_vocabulary::{MessageId, SecretKind};
use support_api::files::files_containing;
use support_api::fixture::FOLDERS_PASSWORD;
use support_api::oauth::add_oauth_account;
use support_read::calls::{operation_of, record, run, wait_finished};
use support_read::mail;
use support_read::requests::{exports_and_events, message_reads};
use support_read::world::{World, start_world};

/// Joue lecture, téléchargement, recherche serveur et exports ; rend tout ce que les clients ont vu
async fn scenario_output(world: &World, exports: &std::path::Path) -> String {
    let mut seen = String::new();
    let id = world.deliver_header_only(&mail::with_attachment(500));
    assert_eq!(world.fetch_body(&id).await, OperationStatus::Finished);
    let absent = MessageRequest::GetMessageBody {
        message: MessageId::from_raw("absent"),
    };
    let search = SearchRequest::SearchServer {
        account: world.account.clone(),
        query: "joint".into(),
    };
    let engine = &world.fixture.engine;
    let operation = operation_of(run(engine, Request::Search(search)).await.unwrap());
    seen.push_str(&format!("{:?}", wait_finished(engine, &operation).await));
    let mut requests = message_reads(&id);
    requests.push(Request::Message(absent));
    requests.extend(exports_and_events(exports));
    for request in requests {
        record(&mut seen, &run(engine, request).await);
    }
    seen
}

/// Mot de passe du compte à mot de passe, jeton d'accès et jeton de rafraîchissement du compte OAuth
fn secrets_of(world: &World, oauth: &mira_vocabulary::Account) -> Vec<String> {
    let stored = |kind| world.fixture.stored_secret(&oauth.id, kind).unwrap();
    vec![
        FOLDERS_PASSWORD.to_string(),
        stored(SecretKind::AccessToken),
        stored(SecretKind::RefreshToken),
    ]
}

/// Invariant 15 : aucun secret (mot de passe, jetons) dans les sorties, les exports, les événements ni la base
#[tokio::test]
async fn invariant_15_no_secret_in_events_errors_or_responses() {
    let world = start_world().await;
    let oauth = add_oauth_account(&world.fixture, "oauth-key").await;
    let exports = tempfile::tempdir().unwrap();
    let mut seen = scenario_output(&world, exports.path()).await;
    for name in ["d.json", "p.json"] {
        seen.push_str(&std::fs::read_to_string(exports.path().join(name)).unwrap());
    }
    world.fixture.imap.shutdown();
    let offline = Request::Message(MessageRequest::FetchMessageBody {
        message: MessageId::from_raw("m1"),
    });
    record(&mut seen, &run(&world.fixture.engine, offline).await);
    let secrets = secrets_of(&world, &oauth);
    assert!(secrets.iter().all(|secret| !seen.contains(secret.as_str())));
    let home = world.fixture.home.path().to_path_buf();
    world.fixture.engine.shutdown().unwrap();
    for secret in &secrets {
        assert!(
            files_containing(&home, secret).is_empty(),
            "secret écrit sur disque"
        );
    }
}
