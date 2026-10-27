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
use support_api::queries::event_kinds;
use support_imap::within;
use support_label_api::calls::{changed_count, labels};
use support_label_api::files::{file_of, import_file};
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

/// Ni les erreurs ni les événements ne portent les noms de libellés
#[tokio::test]
async fn import_never_leaks_label_names_in_errors_or_events() {
    within(async {
        let f = Fixture::start().await;
        let broken = "mira_export: [\nSECRET-MARK-42";
        let error = import_file(f.engine(), broken, false, "k-1")
            .await
            .unwrap_err();
        assert!(!error.contains("SECRET-MARK-42"));
        let yaml = file_of(vec![json!({"name": "SECRET-MARK-42"})]);
        import_file(f.engine(), &yaml, false, "k-2").await.unwrap();
        let events = format!("{:?}", event_kinds(f.engine()).await);
        assert!(!events.contains("SECRET-MARK-42"));
    })
    .await;
}
