#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Domaine « system_label » par la façade : choix manuel, poses listées, conversation inconnue, annonce.

mod support;
mod support_api;
mod support_imap;
mod support_label_api;
mod support_mirror;
mod support_system_api;
mod support_write;

use mira_vocabulary::{SystemLabelAssignment, SystemLabelKind as Kind, ThreadId};
use support_imap::within;
use support_label_api::threads::seed_thread;
use support_system_api::{changed_count, listed, set};
use support_write::fixture::Fixture;

/// Poser deux sortes sur une conversation, en retirer une : la liste suit et chaque écriture est annoncée
#[tokio::test]
async fn manual_set_and_remove_are_listed_and_announced() {
    within(async {
        let f = Fixture::start().await;
        let (thread, _) = seed_thread(&f.db, &f.account, "un");
        let here = std::slice::from_ref(&thread);
        set(f.engine(), Kind::Needs, here, true).await.unwrap();
        set(f.engine(), Kind::Waiting, here, true).await.unwrap();
        set(f.engine(), Kind::Needs, here, false).await.unwrap();
        let shown = listed(f.engine()).await;
        assert_eq!(
            shown,
            vec![SystemLabelAssignment {
                thread,
                kinds: vec![Kind::Waiting]
            }]
        );
        assert_eq!(changed_count(f.engine()).await, 3);
    })
    .await;
}

/// Une conversation inconnue refuse toute la pose, sans écriture ni annonce
#[tokio::test]
async fn an_unknown_thread_refuses_the_whole_call() {
    within(async {
        let f = Fixture::start().await;
        let (thread, _) = seed_thread(&f.db, &f.account, "un");
        let both = [thread, ThreadId::from_raw("fantome")];
        let refused = set(f.engine(), Kind::Mention, &both, true).await;
        assert_eq!(refused.unwrap_err(), "not_found.thread");
        assert!(listed(f.engine()).await.is_empty());
        assert_eq!(changed_count(f.engine()).await, 0);
    })
    .await;
}

/// Poser deux fois la même sorte ne change rien à la liste
#[tokio::test]
async fn setting_twice_is_idempotent() {
    within(async {
        let f = Fixture::start().await;
        let (thread, _) = seed_thread(&f.db, &f.account, "un");
        let here = std::slice::from_ref(&thread);
        set(f.engine(), Kind::Mention, here, true).await.unwrap();
        set(f.engine(), Kind::Mention, here, true).await.unwrap();
        assert_eq!(listed(f.engine()).await.len(), 1);
    })
    .await;
}
