#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Domaine « local_label » par la façade : reprise des libellés de l'app (ImportLocalLabels).

mod support;
mod support_api;
mod support_imap;
mod support_label_api;
mod support_mirror;
mod support_write;

use mira_core::api::{ImportReport, ImportedLabel, LocalLabelRequest, LocalLabelResponse};
use support_imap::within;
use support_label_api::calls::{call, changed_count, key, labels};
use support_label_api::threads::seed_thread;
use support_write::fixture::Fixture;

/// Demande de reprise d'un libellé et de ses conversations
fn imported(name: &str, conversations: &[&str]) -> ImportedLabel {
    ImportedLabel {
        name: name.to_string(),
        color: 0xFF6A_A8D9,
        conversations: conversations.iter().map(|c| c.to_string()).collect(),
    }
}

/// Envoie une reprise sous cette clé et rend le rapport, ou le code d'erreur
async fn import(
    f: &Fixture,
    list: Vec<ImportedLabel>,
    key_text: &str,
) -> Result<ImportReport, String> {
    let request = LocalLabelRequest::ImportLocalLabels {
        labels: list,
        idempotency_key: key(key_text),
    };
    match call(f.engine(), request).await? {
        LocalLabelResponse::Imported(report) => Ok(report),
        other => panic!("réponse inattendue : {other:?}"),
    }
}

/// Une clé de conversation vaut fil, sinon message (rattaché à son fil), sinon elle est ignorée et comptée
#[tokio::test]
async fn import_creates_labels_and_resolves_conversation_keys() {
    within(async {
        let f = Fixture::start().await;
        let (thread, _) = seed_thread(&f.db, &f.account, "un");
        let (_, message) = seed_thread(&f.db, &f.account, "deux");
        let keys = [thread.as_str(), message.as_str(), "inconnue"];
        let report = import(&f, vec![imported("À payer", &keys)], "k-1")
            .await
            .unwrap();
        assert_eq!(
            (report.labels, report.assignments, report.ignored),
            (1, 2, 1)
        );
        assert_eq!(labels(f.engine()).await.len(), 1);
        assert_eq!(changed_count(f.engine()).await, 1);
    })
    .await;
}

/// Un libellé de même nom est réutilisé ; la même clé rejouée rend le même rapport sans nouvel événement
#[tokio::test]
async fn import_reuses_same_name_and_replays_idempotently() {
    within(async {
        let f = Fixture::start().await;
        let (thread, _) = seed_thread(&f.db, &f.account, "un");
        let existing = support_label_api::calls::create(f.engine(), "urgent", "k-0")
            .await
            .unwrap();
        let list = vec![imported("Urgent", &[thread.as_str()])];
        let first = import(&f, list.clone(), "k-1").await.unwrap();
        let again = import(&f, list, "k-1").await.unwrap();
        assert_eq!(first, again);
        assert_eq!(labels(f.engine()).await, vec![existing]);
        assert_eq!(changed_count(f.engine()).await, 2);
    })
    .await;
}

/// Un nom invalide ou un trop grand nombre de libellés refuse toute la reprise, sans rien écrire
#[tokio::test]
async fn import_rejects_invalid_names_without_writing() {
    within(async {
        let f = Fixture::start().await;
        let (thread, _) = seed_thread(&f.db, &f.account, "un");
        let list = vec![imported("Bon", &[thread.as_str()]), imported("  ", &[])];
        let refused = import(&f, list, "k-1").await;
        assert_eq!(refused.unwrap_err(), "invalid_input.parameter");
        let many: Vec<ImportedLabel> = (0..201).map(|n| imported(&format!("L{n}"), &[])).collect();
        assert_eq!(
            import(&f, many, "k-2").await.unwrap_err(),
            "invalid_input.parameter"
        );
        assert!(labels(f.engine()).await.is_empty());
        assert_eq!(changed_count(f.engine()).await, 0);
    })
    .await;
}
