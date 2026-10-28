#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Import YAML des règles : idempotence par clé (invariant 18).

mod support;
mod support_api;
mod support_imap;
mod support_mirror;
mod support_rule_api;
mod support_write;

use mira_vocabulary::RuleImportOutcome as Outcome;
use support_imap::within;
use support_rule_api::calls::list;
use support_rule_api::portability::{exported_items, file_of, import, outcomes, variant};
use support_rule_api::writes::create_named;
use support_write::fixture::Fixture;

/// Fichier d'une règle nouvelle, tiré d'une règle existante
async fn new_rule_file(f: &Fixture) -> String {
    create_named(f.engine(), &["A"]).await;
    let base = exported_items(f.engine()).await.remove(0);
    file_of(vec![variant(&base, "n1", "Un")])
}

/// La même clé rejouée rend le même rapport sans rien écrire
#[tokio::test]
async fn same_key_same_payload_replays_same_report_without_writing() {
    within(async {
        let f = Fixture::start().await;
        let yaml = new_rule_file(&f).await;
        let first = import(f.engine(), &yaml, false, "k-1").await.unwrap();
        let again = import(f.engine(), &yaml, false, "k-1").await.unwrap();
        assert_eq!(outcomes(&first), vec![Outcome::Imported]);
        assert_eq!(again, first);
        assert_eq!(list(f.engine()).await.len(), 2);
    })
    .await;
}

/// La même clé avec un autre texte est un conflit d'idempotence
#[tokio::test]
async fn same_key_other_yaml_is_idempotency_conflict() {
    within(async {
        let f = Fixture::start().await;
        let yaml = new_rule_file(&f).await;
        import(f.engine(), &yaml, false, "k-1").await.unwrap();
        let other = file_of(vec![]);
        let error = import(f.engine(), &other, false, "k-1").await.unwrap_err();
        assert_eq!(error, "invalid_input.idempotency_conflict");
    })
    .await;
}

/// La même clé avec une autre option de remplacement est un conflit d'idempotence
#[tokio::test]
async fn same_key_other_replace_flag_is_idempotency_conflict() {
    within(async {
        let f = Fixture::start().await;
        let yaml = new_rule_file(&f).await;
        import(f.engine(), &yaml, false, "k-1").await.unwrap();
        let error = import(f.engine(), &yaml, true, "k-1").await.unwrap_err();
        assert_eq!(error, "invalid_input.idempotency_conflict");
    })
    .await;
}
