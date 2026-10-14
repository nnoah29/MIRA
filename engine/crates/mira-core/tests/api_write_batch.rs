#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Actions en lot par la façade : sélection par requête, doublons, lot vide, plusieurs comptes (12.4, inv. 12, 18).

mod support;
mod support_api;
mod support_imap;
mod support_mirror;
mod support_write;

use mira_vocabulary::{ActionKind, ActionState, MessageId, StandardView};
use support_imap::within;
use support_write::builders::apply_request;
use support_write::fixture::Fixture;

/// Une sélection par requête devient une seule action portant sur tous les messages de la vue
#[tokio::test]
async fn apply_action_on_the_result_of_a_query() {
    within(async {
        let f = Fixture::start().await;
        let read = ActionKind::MarkRead { read: true };
        let ids = f.apply_to_view(read, StandardView::Unread, "k1").await;
        assert_eq!(ids.len(), 1);
        assert_eq!(f.actions(&[]).await[0].targets.len(), 3);
        let all_read = ["msg-1", "msg-2", "msg-3"].iter();
        assert!(all_read.into_iter().all(|id| f.message(id).flags.seen));
        f.advance(5_001);
        f.wait_for_state(&ids[0], ActionState::Succeeded).await;
        assert!([1, 2, 3].iter().all(|uid| f.server_seen("INBOX", *uid)));
    })
    .await;
}

/// Une sélection vide est refusée ; les doublons sont ramenés à un seul message
#[tokio::test]
async fn empty_selection_is_refused_and_duplicates_are_merged() {
    within(async {
        let f = Fixture::start().await;
        let empty = apply_request(ActionKind::Archive, &[], "k1");
        assert_eq!(
            f.refused(empty).await.code,
            "invalid_input.missing_parameter"
        );
        let ids = f
            .apply(
                ActionKind::MarkFlagged { flagged: true },
                &["msg-1", "msg-1"],
                "k2",
            )
            .await;
        assert_eq!(ids.len(), 1);
        let queued = f.actions(&[]).await;
        assert_eq!(queued[0].targets, vec![MessageId::from_raw("msg-1")]);
    })
    .await;
}

/// Un lot sur deux comptes donne une action par compte, mises en file ensemble et rejouées chacune sur son serveur
#[tokio::test]
async fn apply_action_batch_spans_accounts_with_one_action_each() {
    within(async {
        let f = Fixture::start().await;
        let second = f.add_second_account().await;
        let (read, both) = (ActionKind::MarkRead { read: true }, ["msg-1", "msg-b1"]);
        let ids = f.apply(read.clone(), &both, "k1").await;
        assert_eq!(ids.len(), 2);
        assert!(f.message("msg-1").flags.seen && f.message("msg-b1").flags.seen);
        assert_eq!(f.apply(read, &both, "k1").await, ids);
        f.advance(5_001);
        for id in &ids {
            f.wait_for_state(id, ActionState::Succeeded).await;
        }
        let views = f.store.messages(support_write::SECOND_USER, "INBOX");
        assert!(
            f.server_seen("INBOX", 1) && views[0].flags.iter().any(|flag| flag.contains("Seen"))
        );
        let owned = f.db.read(|tx| tx.get_action(&ids[1])).unwrap().unwrap();
        assert_eq!(owned.account, second);
    })
    .await;
}

/// Un message inconnu annule tout le lot multi-comptes : aucun effet local, aucune action
#[tokio::test]
async fn multi_account_batch_is_all_or_nothing() {
    within(async {
        let f = Fixture::start().await;
        f.add_second_account().await;
        let request = apply_request(
            ActionKind::MarkRead { read: true },
            &["msg-1", "msg-b1", "msg-x"],
            "k1",
        );
        assert_eq!(f.refused(request).await.code, "not_found.message");
        assert!(!f.message("msg-1").flags.seen && !f.message("msg-b1").flags.seen);
        assert!(f.actions(&[]).await.is_empty());
    })
    .await;
}
