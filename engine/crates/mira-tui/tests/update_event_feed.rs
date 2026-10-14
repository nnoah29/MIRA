//! Flux d'événements : le moteur émet, la boucle reçoit des `Msg::Event`.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod support;

use std::time::Duration;

use mira_assembly::api::{ActionRequest, Request, TargetSelection};
use mira_assembly::mira_vocabulary::{
    ActionKind, EventKind, IdempotencyKey, MailboxRole, MessageId,
};
use mira_tui::app::event_feed::spawn_event_feed;
use mira_tui::update::Msg;
use support::world::TestWorld;
use tokio::sync::mpsc;
use tokio::time::timeout;

const WAIT: Duration = Duration::from_secs(10);

/// Un événement émis par le moteur arrive comme `Msg::Event`.
#[tokio::test]
async fn engine_event_arrives_as_event_message() {
    let world = TestWorld::with_imap_account().await;
    world
        .seed_mailbox("INBOX", MailboxRole::Inbox, &["Premier"])
        .unwrap();
    let engine = world.engine();
    let (sender, mut receiver) = mpsc::unbounded_channel();
    let feed = spawn_event_feed(&engine, sender);
    let request = Request::Action(ActionRequest::ApplyAction {
        kind: ActionKind::MarkRead { read: true },
        targets: TargetSelection::Messages {
            messages: vec![MessageId::from_raw("INBOX-1")],
        },
        idempotency_key: IdempotencyKey("feed-1".to_string()),
    });
    engine.execute(request).await.unwrap();
    assert!(next_action_event(&mut receiver).await);
    drop(receiver);
    drop(engine);
    feed.abort();
    world.close().await.unwrap();
}

/// Lit les messages jusqu'à un événement de file d'actions ; faux si le délai est dépassé.
async fn next_action_event(receiver: &mut mpsc::UnboundedReceiver<Msg>) -> bool {
    while let Ok(Some(Msg::Event(event))) = timeout(WAIT, receiver.recv()).await {
        if matches!(event.kind, EventKind::ActionQueued { .. }) {
            return true;
        }
    }
    false
}

/// La tâche s'arrête d'elle-même quand le récepteur est fermé.
#[tokio::test]
async fn feed_stops_when_receiver_is_closed() {
    let world = TestWorld::new();
    let engine = world.engine();
    let (sender, receiver) = mpsc::unbounded_channel();
    let feed = spawn_event_feed(&engine, sender);
    drop(receiver);
    timeout(WAIT, feed).await.unwrap().unwrap();
    drop(engine);
    world.close().await.unwrap();
}
