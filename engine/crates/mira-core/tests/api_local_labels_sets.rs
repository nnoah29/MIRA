#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Domaine « local_label » par la façade : pose et retrait sur les conversations, poses listées, rejeu.

mod support;
mod support_api;
mod support_imap;
mod support_label_api;
mod support_mirror;
mod support_write;

use mira_core::api::{LocalLabelRequest, LocalLabelResponse};
use mira_vocabulary::ThreadId;
use support_imap::within;
use support_label_api::calls::{call, changed_count, create, set};
use support_label_api::threads::seed_thread;
use support_write::fixture::Fixture;

/// Poser sur deux conversations puis retirer sur une : la liste des poses suit, par conversation
#[tokio::test]
async fn set_then_list_assignments_follows_threads() {
    within(async {
        let f = Fixture::start().await;
        let (one, _) = seed_thread(&f.db, &f.account, "un");
        let (two, _) = seed_thread(&f.db, &f.account, "deux");
        let urgent = create(f.engine(), "Urgent", "k-1").await.unwrap();
        let both = [one.clone(), two.clone()];
        set(f.engine(), &urgent, &both, true, "k-2").await.unwrap();
        let first = std::slice::from_ref(&one);
        set(f.engine(), &urgent, first, false, "k-3").await.unwrap();
        let answer = call(f.engine(), LocalLabelRequest::ListLocalLabelAssignments).await;
        let LocalLabelResponse::Assignments(listed) = answer.unwrap() else {
            panic!("réponse inattendue");
        };
        assert_eq!(listed.len(), 1);
        assert_eq!(
            (&listed[0].thread, &listed[0].labels),
            (&two, &vec![urgent.id.clone()])
        );
    })
    .await;
}

/// Un fil inconnu ou plus de 1000 fils : toute la pose est refusée, rien n'est écrit
#[tokio::test]
async fn set_refuses_unknown_thread_and_oversized_batches() {
    within(async {
        let f = Fixture::start().await;
        let (known, _) = seed_thread(&f.db, &f.account, "un");
        let urgent = create(f.engine(), "Urgent", "k-1").await.unwrap();
        let ghost = ThreadId::from_raw("zz");
        let mixed = set(f.engine(), &urgent, &[known.clone(), ghost], true, "k-2").await;
        assert_eq!(mixed.unwrap_err(), "not_found.thread");
        let many = vec![known.clone(); 1001];
        let oversized = set(f.engine(), &urgent, &many, true, "k-3").await;
        assert_eq!(oversized.unwrap_err(), "invalid_input.parameter");
        let answer = call(f.engine(), LocalLabelRequest::ListLocalLabelAssignments).await;
        assert!(matches!(answer.unwrap(), LocalLabelResponse::Assignments(a) if a.is_empty()));
    })
    .await;
}

/// Rejouer la même pose ne change rien et n'émet pas d'événement ; un libellé inconnu est refusé
#[tokio::test]
async fn set_replay_emits_no_second_event() {
    within(async {
        let f = Fixture::start().await;
        let (thread, _) = seed_thread(&f.db, &f.account, "un");
        let urgent = create(f.engine(), "Urgent", "k-1").await.unwrap();
        let here = std::slice::from_ref(&thread);
        set(f.engine(), &urgent, here, true, "k-2").await.unwrap();
        set(f.engine(), &urgent, here, true, "k-2").await.unwrap();
        assert_eq!(changed_count(f.engine()).await, 2);
        let mut ghost = urgent.clone();
        ghost.id = mira_vocabulary::LocalLabelId::from_raw("zz");
        let unknown = set(f.engine(), &ghost, here, true, "k-3").await;
        assert_eq!(unknown.unwrap_err(), "not_found.local_label");
        assert_eq!(changed_count(f.engine()).await, 2);
    })
    .await;
}
