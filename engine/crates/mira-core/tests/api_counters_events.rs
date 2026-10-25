#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Compteurs de boîtes annoncés par le moteur : toute variation est dite avant l'événement qui l'explique.

mod support_api;

use mira_core::api::{ActionRequest, Engine, Request, TargetSelection};
use mira_vocabulary::{
    AccountId, ActionKind, EventKind, IdempotencyKey, Mailbox, MessageId, MessageSummary,
};
use support_api::sync_calls::wait_for_events;
use support_api::sync_events::count_kinds;
use support_api::sync_reads::{mailbox_named, summaries_in, wait_for_subjects};
use support_api::synced::synced_account;
use support_api::syncing::deliver_one;

/// Compteurs d'une boîte annoncés par un événement : (boîte, total, non lus)
fn announced_counters(kind: &EventKind) -> Option<(String, u64, u64)> {
    match kind {
        EventKind::MailboxCountersUpdated {
            mailbox,
            total,
            unread,
            ..
        } => Some((mailbox.to_string(), *total, *unread)),
        _ => None,
    }
}

/// Marque un message lu par la façade
async fn mark_read(engine: &Engine, message: &MessageSummary, key: &str) {
    let request = Request::Action(ActionRequest::ApplyAction {
        kind: ActionKind::MarkRead { read: true },
        targets: TargetSelection::Messages {
            messages: vec![MessageId::from_raw(message.id.as_str())],
        },
        idempotency_key: IdempotencyKey(key.to_string()),
    });
    engine.execute(request).await.unwrap();
}

/// Réception de deux messages, synchronisée et annoncée ; rend la boîte et le premier message
async fn inbox_with_two_unread(
    f: &support_api::fixture::Fixture,
    account: &AccountId,
) -> (Mailbox, MessageSummary) {
    deliver_one(f, "INBOX", "Premier");
    deliver_one(f, "INBOX", "Second");
    wait_for_subjects(&f.engine, account, "INBOX", 2).await;
    let inbox = mailbox_named(&f.engine, account, "INBOX").await;
    wait_for_events(&f.engine, |kinds| {
        let last = kinds.iter().rev().find_map(announced_counters);
        last.is_some_and(|(id, _, unread)| id == inbox.id.to_string() && unread == 2)
    })
    .await;
    let first = summaries_in(&f.engine, account, "INBOX").await.remove(0);
    (inbox, first)
}

/// Marquer lu annonce les nouveaux compteurs de la boîte, avant l'annonce de l'action
#[tokio::test]
async fn marking_read_announces_counters_before_the_action() {
    let (f, account) = synced_account().await;
    let (inbox, message) = inbox_with_two_unread(&f, &account).await;
    let before = wait_for_events(&f.engine, |_| true).await.len();
    mark_read(&f.engine, &message, "k-read-1").await;
    let kinds = wait_for_events(&f.engine, |k| {
        count_kinds(&k[before..], |e| {
            matches!(e, EventKind::ActionQueued { .. })
        }) == 1
    })
    .await;
    let fresh = &kinds[before..];
    let counters = fresh.iter().position(|k| {
        announced_counters(k)
            .is_some_and(|(id, _, unread)| id == inbox.id.to_string() && unread == 1)
    });
    let queued = fresh
        .iter()
        .position(|k| matches!(k, EventKind::ActionQueued { .. }));
    assert!(counters.unwrap() < queued.unwrap());
}

/// Rien n'est annoncé quand une action ne change aucun compteur
#[tokio::test]
async fn unchanged_counters_are_not_announced_again() {
    let (f, account) = synced_account().await;
    let (_inbox, message) = inbox_with_two_unread(&f, &account).await;
    mark_read(&f.engine, &message, "k-read-2").await;
    wait_for_events(&f.engine, |k| {
        count_kinds(k, |e| matches!(e, EventKind::ActionQueued { .. })) == 1
    })
    .await;
    let announced = |k: &[EventKind]| count_kinds(k, |e| announced_counters(e).is_some());
    let before = announced(&wait_for_events(&f.engine, |_| true).await);
    mark_read(&f.engine, &message, "k-read-3").await;
    let kinds = wait_for_events(&f.engine, |k| {
        count_kinds(k, |e| matches!(e, EventKind::ActionQueued { .. })) == 2
    })
    .await;
    assert_eq!(announced(&kinds), before);
}

/// Après des arrivées et des actions, les derniers compteurs annoncés de chaque boîte sont ceux du miroir
#[tokio::test]
async fn last_announced_counters_match_the_mirror() {
    let (f, account) = synced_account().await;
    let (_inbox, message) = inbox_with_two_unread(&f, &account).await;
    mark_read(&f.engine, &message, "k-read-4").await;
    deliver_one(&f, "INBOX", "Troisième");
    wait_for_subjects(&f.engine, &account, "INBOX", 3).await;
    let inbox = mailbox_named(&f.engine, &account, "INBOX").await;
    let kinds = wait_for_events(&f.engine, |k| {
        let last = k.iter().rev().find_map(announced_counters);
        last == Some((inbox.id.to_string(), inbox.total, inbox.unread))
    })
    .await;
    assert!(count_kinds(&kinds, |e| announced_counters(e).is_some()) > 1);
}
