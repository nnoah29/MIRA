#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Import YAML des libellés locaux : événements, fichier invalide, panne de la base, confidentialité.

mod support;
mod support_api;
mod support_imap;
mod support_label_api;
mod support_mirror;
mod support_write;

use mira_core::profile::ProfilePaths;
use serde_json::json;
use support_api::leaks::{events_text, failure_text};
use support_api::queries::event_kinds;
use support_imap::within;
use support_label_api::calls::{changed_count, labels};
use support_label_api::files::{file_of, import_file};
use support_label_api::seeds::import_request;
use support_write::fixture::Fixture;

/// Un événement après une écriture ; aucun pour un rejeu ni pour un import sans effet
#[tokio::test]
async fn one_event_after_a_write_and_none_otherwise() {
    within(async {
        let f = Fixture::start().await;
        let yaml = file_of(vec![json!({"name": "Un"}), json!({"name": "Deux"})]);
        let before = changed_count(f.engine()).await;
        import_file(f.engine(), &yaml, false, "k-imp")
            .await
            .unwrap();
        assert_eq!(changed_count(f.engine()).await, before + 1);
        import_file(f.engine(), &yaml, false, "k-imp")
            .await
            .unwrap();
        import_file(f.engine(), &yaml, false, "k-again")
            .await
            .unwrap();
        assert_eq!(changed_count(f.engine()).await, before + 1);
    })
    .await;
}

/// Un fichier invalide est refusé en entier : rien d'écrit, aucun événement
#[tokio::test]
async fn invalid_file_writes_nothing_and_emits_nothing() {
    within(async {
        let f = Fixture::start().await;
        let before = event_kinds(f.engine()).await.len();
        let error = import_file(f.engine(), "mira_export: [\n", false, "k-imp")
            .await
            .unwrap_err();
        assert_eq!(error, "invalid_input.yaml_syntax");
        assert!(labels(f.engine()).await.is_empty());
        assert_eq!(event_kinds(f.engine()).await.len(), before);
    })
    .await;
}

/// Une panne de la base en cours d'import n'écrit rien du tout
#[tokio::test]
async fn database_failure_mid_import_writes_nothing() {
    within(async {
        let f = Fixture::start().await;
        let trigger = "CREATE TRIGGER boom BEFORE INSERT ON local_labels WHEN NEW.name = 'Boom' \
                       BEGIN SELECT RAISE(ABORT, 'panne'); END;";
        let paths = ProfilePaths::resolve(&f.config.profile, &f.config.environment).unwrap();
        rusqlite::Connection::open(paths.database)
            .unwrap()
            .execute_batch(trigger)
            .unwrap();
        let yaml = file_of(vec![json!({"name": "Un"}), json!({"name": "Boom"})]);
        assert!(
            import_file(f.engine(), &yaml, false, "k-imp")
                .await
                .is_err()
        );
        assert!(labels(f.engine()).await.is_empty());
    })
    .await;
}

/// Ni les erreurs (syntaxe, panne de la base) ni les événements ne portent les noms de libellés
#[tokio::test]
async fn import_never_leaks_label_names_in_errors_or_events() {
    within(async {
        let f = Fixture::start().await;
        let trigger = "CREATE TRIGGER boom BEFORE INSERT ON local_labels WHEN NEW.name = 'SECRET-MARK-42' \
                       BEGIN SELECT RAISE(ABORT, 'panne'); END;";
        let paths = ProfilePaths::resolve(&f.config.profile, &f.config.environment).unwrap();
        let db = rusqlite::Connection::open(paths.database).unwrap();
        db.execute_batch(trigger).unwrap();
        let failing = file_of(vec![json!({"name": "SECRET-MARK-42"})]);
        let storage = failure_text(f.engine(), import_request(&failing, "k-1")).await;
        let syntax = failure_text(f.engine(), import_request("mira_export: [\nSECRET-MARK-42", "k-2")).await;
        assert!(storage.contains("local_storage."), "{storage}");
        assert!(syntax.contains("invalid_input.yaml_syntax"), "{syntax}");
        db.execute_batch("DROP TRIGGER boom").unwrap();
        import_file(f.engine(), &failing, false, "k-3").await.unwrap();
        for text in [&storage, &syntax, &events_text(f.engine()).await] {
            assert!(!text.contains("SECRET-MARK-42"), "{text}");
        }
    })
    .await;
}
