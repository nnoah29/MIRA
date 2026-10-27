#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Import YAML des règles : événements, atomicité, fichier invalide, confidentialité.

mod support;
mod support_api;
mod support_imap;
mod support_mirror;
mod support_rule_api;
mod support_write;

use mira_core::profile::ProfilePaths;
use support_api::queries::event_kinds;
use support_imap::within;
use support_rule_api::calls::list;
use support_rule_api::portability::{exported_items, file_of, import, variant};
use support_rule_api::samples::changed_rules;
use support_rule_api::writes::create_named;
use support_write::fixture::Fixture;

/// Un événement par règle écrite ; aucun pour un import sans effet ni pour un rejeu
#[tokio::test]
async fn one_rule_changed_event_per_written_rule_and_none_otherwise() {
    within(async {
        let f = Fixture::start().await;
        create_named(f.engine(), &["A"]).await;
        let base = exported_items(f.engine()).await.remove(0);
        let before = changed_rules(f.engine()).await.len();
        let yaml = file_of(vec![
            base.clone(),
            variant(&base, "n1", "Un"),
            variant(&base, "n2", "Deux"),
        ]);
        import(f.engine(), &yaml, false, "k-imp").await.unwrap();
        let after = changed_rules(f.engine()).await;
        assert_eq!(&after[before..], ["n1", "n2"]);
        import(f.engine(), &yaml, false, "k-imp").await.unwrap();
        import(f.engine(), &yaml, false, "k-again").await.unwrap();
        assert_eq!(changed_rules(f.engine()).await.len(), after.len());
    })
    .await;
}

/// Un fichier invalide est refusé en entier : rien d'écrit, aucun événement
#[tokio::test]
async fn invalid_file_writes_nothing_and_emits_nothing() {
    within(async {
        let f = Fixture::start().await;
        let before = event_kinds(f.engine()).await.len();
        let error = import(f.engine(), "mira_export: [\n", false, "k-imp")
            .await
            .unwrap_err();
        assert_eq!(error, "invalid_input.yaml_syntax");
        assert!(list(f.engine()).await.is_empty());
        assert_eq!(event_kinds(f.engine()).await.len(), before);
    })
    .await;
}

/// Une panne de la base en cours d'import n'écrit rien du tout
#[tokio::test]
async fn database_failure_mid_import_writes_nothing() {
    within(async {
        let f = Fixture::start().await;
        create_named(f.engine(), &["A"]).await;
        let base = exported_items(f.engine()).await.remove(0);
        let trigger = "CREATE TRIGGER boom BEFORE INSERT ON rules WHEN NEW.name = 'Boom'                        BEGIN SELECT RAISE(ABORT, 'panne'); END;";
        let paths = ProfilePaths::resolve(&f.config.profile, &f.config.environment).unwrap();
        rusqlite::Connection::open(paths.database).unwrap().execute_batch(trigger).unwrap();
        let items = vec![variant(&base, "n1", "Un"), variant(&base, "n2", "Boom")];
        assert!(import(f.engine(), &file_of(items), false, "k-imp").await.is_err());
        assert_eq!(list(f.engine()).await.len(), 1);
    })
    .await;
}

/// Ni les erreurs ni les événements ne portent le contenu des règles
#[tokio::test]
async fn import_never_leaks_rule_content_in_errors_or_events() {
    within(async {
        let f = Fixture::start().await;
        create_named(f.engine(), &["A"]).await;
        let base = exported_items(f.engine()).await.remove(0);
        let error = import(f.engine(), "mira_export: [\nSECRET-MARK-42", false, "k-1")
            .await
            .unwrap_err();
        assert!(!error.contains("SECRET-MARK-42"));
        let secret = variant(&base, "n1", "SECRET-MARK-42");
        import(f.engine(), &file_of(vec![secret]), false, "k-2")
            .await
            .unwrap();
        let events = format!("{:?}", event_kinds(f.engine()).await);
        assert!(!events.contains("SECRET-MARK-42"));
    })
    .await;
}
