#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Domaine « label_settings » par la façade : liste, réglages partiels, ordre, nettoyage à la suppression.

mod support;
mod support_api;
mod support_imap;
mod support_label_api;
mod support_mirror;
mod support_write;

use mira_core::api::{LabelSettingsRequest, LocalLabelRequest};
use mira_vocabulary::{CounterMode, LabelRef, MailboxId, MailboxRole};
use support_imap::within;
use support_label_api::calls::{call as local_call, create};
use support_label_api::settings::{call, changed_count, labels, local_ref, server_ref, set};
use support_mirror::fixtures::sample_mailbox;
use support_write::fixture::Fixture;

/// Ajoute à la base un libellé du serveur (boîte « autre ») du compte du banc
fn add_server_label(f: &Fixture, id: &str, name: &str) -> LabelRef {
    let mailbox = sample_mailbox(id, f.account.as_str(), name, MailboxRole::Other);
    f.db.write(|tx| tx.upsert_mailbox(&mailbox)).unwrap();
    server_ref(id)
}

/// La liste met les libellés locaux d'abord, puis ceux du compte (dont « Work », déjà là) ; réglages par défaut, couleur effective, « inutilisé »
#[tokio::test]
async fn list_shows_local_then_server_labels_with_defaults() {
    within(async {
        let f = Fixture::start().await;
        let urgent = create(f.engine(), "Urgent", "k-1").await.unwrap();
        add_server_label(&f, "srv-1", "Factures");
        let shown = labels(f.engine()).await;
        let names: Vec<&str> = shown.iter().map(|l| l.name.as_str()).collect();
        assert_eq!(names, ["Urgent", "Factures", "Work"]);
        assert_eq!(shown[0].label, local_ref(urgent.id.as_str()));
        assert_eq!(shown[0].color, urgent.color);
        assert!(shown[..2].iter().all(|l| l.unused));
        assert!(shown.iter().all(|l| !l.settings.hidden));
        assert!(shown.iter().all(|l| l.settings.notifications));
        assert_eq!(shown[1].account.as_ref(), Some(&f.account));
    })
    .await;
}

/// Seuls les champs fournis changent ; rien ne change, rien n'est annoncé ; libellé inconnu : refusé
#[tokio::test]
async fn set_changes_only_given_fields_and_announces_once() {
    within(async {
        let f = Fixture::start().await;
        let label = add_server_label(&f, "srv-1", "Factures");
        let hidden = set(f.engine(), &label, Some(true), None, None)
            .await
            .unwrap();
        assert!(hidden.settings.hidden && !hidden.settings.hide_from_inbox);
        let counted = set(f.engine(), &label, None, None, Some(CounterMode::Total))
            .await
            .unwrap();
        assert!(counted.settings.hidden);
        assert_eq!(counted.settings.counter, CounterMode::Total);
        set(f.engine(), &label, Some(true), None, None)
            .await
            .unwrap();
        assert_eq!(changed_count(f.engine()).await, 2);
        let unknown = set(f.engine(), &server_ref("zz"), Some(true), None, None).await;
        assert_eq!(unknown.unwrap_err(), "not_found.label");
    })
    .await;
}

/// Réordonner fixe l'ordre d'affichage ; une liste incomplète ou en double est refusée
#[tokio::test]
async fn reorder_sets_positions_and_refuses_bad_lists() {
    within(async {
        let f = Fixture::start().await;
        add_server_label(&f, "srv-a", "Alpha");
        add_server_label(&f, "srv-b", "Bêta");
        let all: Vec<LabelRef> = labels(f.engine())
            .await
            .into_iter()
            .map(|l| l.label)
            .collect();
        let mut wanted = all.clone();
        wanted.sort_by_key(|l| std::cmp::Reverse(l.id.clone()));
        let reorder = |order: Vec<LabelRef>| LabelSettingsRequest::ReorderLabels { order };
        call(f.engine(), reorder(wanted.clone())).await.unwrap();
        let shown: Vec<LabelRef> = labels(f.engine())
            .await
            .into_iter()
            .map(|l| l.label)
            .collect();
        assert_eq!(shown, wanted);
        let missing = call(f.engine(), reorder(vec![all[0].clone()])).await;
        assert_eq!(missing.unwrap_err(), "invalid_input.parameter");
        let twice = call(f.engine(), reorder([vec![all[0].clone()], all].concat())).await;
        assert_eq!(twice.unwrap_err(), "invalid_input.parameter");
    })
    .await;
}

/// Supprimer un libellé local ou une boîte efface ses réglages
#[tokio::test]
async fn deleting_a_label_removes_its_settings() {
    within(async {
        let f = Fixture::start().await;
        let urgent = create(f.engine(), "Urgent", "k-1").await.unwrap();
        let server = add_server_label(&f, "srv-1", "Factures");
        set(
            f.engine(),
            &local_ref(urgent.id.as_str()),
            Some(true),
            None,
            None,
        )
        .await
        .unwrap();
        set(f.engine(), &server, Some(true), None, None)
            .await
            .unwrap();
        let delete = LocalLabelRequest::DeleteLocalLabel {
            label: urgent.id.clone(),
        };
        local_call(f.engine(), delete).await.unwrap();
        f.db.write(|tx| tx.delete_mailbox(&MailboxId::from_raw("srv-1")))
            .unwrap();
        assert!(f.db.read(|tx| tx.list_label_settings()).unwrap().is_empty());
    })
    .await;
}
