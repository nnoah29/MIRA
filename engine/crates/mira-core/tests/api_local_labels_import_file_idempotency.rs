#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Import YAML des libellés locaux : idempotence par clé (invariant 18).

mod support;
mod support_api;
mod support_imap;
mod support_label_api;
mod support_mirror;
mod support_write;

use mira_vocabulary::LabelImportOutcome as Outcome;
use serde_json::json;
use support_imap::within;
use support_label_api::calls::labels;
use support_label_api::files::{file_of, import_file, outcomes};
use support_write::fixture::Fixture;

/// Fichier d'un seul libellé
fn one_label() -> String {
    file_of(vec![json!({"name": "Compta", "color": "#ff1e90ff"})])
}

/// La même clé rejouée rend le même rapport sans rien écrire
#[tokio::test]
async fn same_key_same_payload_replays_same_report_without_writing() {
    within(async {
        let f = Fixture::start().await;
        let first = import_file(f.engine(), &one_label(), false, "k-1")
            .await
            .unwrap();
        let again = import_file(f.engine(), &one_label(), false, "k-1")
            .await
            .unwrap();
        assert_eq!(outcomes(&first), vec![Outcome::Imported]);
        assert_eq!(again, first);
        assert_eq!(labels(f.engine()).await.len(), 1);
    })
    .await;
}

/// La même clé avec un autre texte est un conflit d'idempotence
#[tokio::test]
async fn same_key_other_yaml_is_idempotency_conflict() {
    within(async {
        let f = Fixture::start().await;
        import_file(f.engine(), &one_label(), false, "k-1")
            .await
            .unwrap();
        let other = file_of(vec![]);
        let error = import_file(f.engine(), &other, false, "k-1")
            .await
            .unwrap_err();
        assert_eq!(error, "invalid_input.idempotency_conflict");
    })
    .await;
}

/// La même clé avec une autre option de remplacement est un conflit d'idempotence
#[tokio::test]
async fn same_key_other_replace_flag_is_idempotency_conflict() {
    within(async {
        let f = Fixture::start().await;
        import_file(f.engine(), &one_label(), false, "k-1")
            .await
            .unwrap();
        let error = import_file(f.engine(), &one_label(), true, "k-1")
            .await
            .unwrap_err();
        assert_eq!(error, "invalid_input.idempotency_conflict");
    })
    .await;
}
