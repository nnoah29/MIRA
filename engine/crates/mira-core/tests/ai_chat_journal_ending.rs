#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Fin de la ligne de journal : annulation, échec, reprise à l'ouverture du moteur, rétention.

mod ai_chat_support;
mod ai_models_support;
mod support_ai_journal;
mod support_api;

use ai_chat_support::{collect, journal_rows, loopback_stream, wait_first_request};
use ai_models_support::{ModelsFixture, TestProvider};
use mira_core::api::{OperationRequest, Request};
use mira_core::profile::ProfilePaths;
use mira_core::storage::Database;
use mira_testkit::ai::{Behavior, openai_stream};
use support_ai_journal::{entry, finish, seed_history};

/// Annuler en plein envoi complète la ligne en `cancelled`
#[tokio::test]
async fn cancelled_generation_completes_the_row_as_cancelled() {
    let fixture = ModelsFixture::start(TestProvider::new("test-provider")).await;
    let mut stream = loopback_stream(&fixture, Behavior::Hang, "Salut").await;
    wait_first_request(&fixture.server).await;
    let cancel = OperationRequest::CancelOperation {
        operation: stream.operation.clone(),
    };
    fixture
        .engine
        .execute(Request::Operation(cancel))
        .await
        .unwrap();
    collect(&mut stream).await;
    let rows = journal_rows(&fixture);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].outcome, "cancelled");
}

/// Un échec du service garde son code stable comme issue de la ligne
#[tokio::test]
async fn failed_generation_records_the_error_code() {
    let fixture = ModelsFixture::start(TestProvider::new("test-provider")).await;
    let refusal = Behavior::Status {
        code: 401,
        headers: Vec::new(),
        body: r#"{"error":{"message":"refus"}}"#.to_string(),
    };
    let mut stream = loopback_stream(&fixture, refusal, "Salut").await;
    collect(&mut stream).await;
    let rows = journal_rows(&fixture);
    assert_eq!(rows[0].outcome, "authentication.ai_key_rejected");
    assert_eq!(rows[0].operation, stream.operation);
}

/// Ouvrir le moteur marque `interrupted` les lignes restées `running` d'un arrêt brutal
#[test]
fn engine_open_marks_running_rows_interrupted() {
    let home = tempfile::tempdir().unwrap();
    let config = support_api::config(home.path());
    let path = ProfilePaths::resolve(&config.profile, &config.environment)
        .unwrap()
        .database;
    let first = support_api::open(home.path());
    let db = Database::open(&path).unwrap();
    db.write(|tx| {
        tx.insert_ai_journal(&entry("j-1", "op-1", 10))?;
        tx.insert_ai_journal(&entry("j-2", "op-2", 20))?;
        tx.finish_ai_journal(
            &mira_vocabulary::OperationId::from_raw("op-2"),
            &finish("end_turn", 0),
        )
    })
    .unwrap();
    first.shutdown().unwrap();
    support_api::open(home.path()).shutdown().unwrap();
    let rows = db.read(|tx| tx.list_ai_journal(10, None)).unwrap();
    let outcomes: Vec<&str> = rows.iter().map(|row| row.outcome.as_str()).collect();
    assert_eq!(outcomes, ["end_turn", "interrupted"]);
}

/// Après une génération, le journal est borné : 2000 lignes, rien de plus vieux que 90 jours, `running` épargné
#[tokio::test]
async fn retention_runs_after_each_write() {
    let fixture = ModelsFixture::start(TestProvider::new("test-provider")).await;
    seed_history(&fixture.db);
    let behavior = openai_stream(&["ok"], Some((1, 1)));
    let mut stream = loopback_stream(&fixture, behavior, "Salut").await;
    collect(&mut stream).await;
    let rows = journal_rows(&fixture);
    let ids: Vec<&str> = rows.iter().map(|row| row.id.as_str()).collect();
    assert_eq!(rows.len(), 2001);
    assert!(ids.contains(&"stuck") && !ids.contains(&"old") && !ids.contains(&"j-0"));
    assert!(
        rows.iter()
            .any(|row| row.operation == stream.operation && row.outcome == "end_turn")
    );
}
