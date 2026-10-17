//! Bout en bout : les événements d'avant la session ne sont pas rejoués à l'écran.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod e2e_support;
mod support;

use std::time::Duration;

use crossterm::event::KeyCode;
use e2e_support::{press, screen_text, wait_for};
use mira_assembly::mira_vocabulary::{ActionId, EventKind, MailboxRole};
use mira_tui::app::run_loop::{LoopHooks, run_loop_with};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use support::world::TestWorld;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

const OLD_REASON: &str = "panne ancienne";

/// Monde avec une boîte amorcée et un échec d'action ancien au journal.
async fn world_with_history() -> TestWorld {
    let world = TestWorld::with_imap_account().await;
    world
        .seed_mailbox("INBOX", MailboxRole::Inbox, &["Facture"])
        .unwrap();
    let kind = EventKind::ActionFailed {
        account: world.account().unwrap(),
        action: ActionId::from_raw("vieille-action"),
        code: "server.mailbox_missing".to_string(),
        reason: OLD_REASON.to_string(),
    };
    world.seed_event(&kind).unwrap();
    world
}

/// Attend l'écran chargé, laisse passer quelques battements, vérifie qu'aucune image n'a montré l'ancienne raison.
async fn script(
    keys: UnboundedSender<crossterm::event::KeyEvent>,
    mut frames: UnboundedReceiver<Buffer>,
) {
    let loaded = |s: &str| s.contains("Réception") && s.contains("Facture");
    let first = wait_for(&mut frames, "boîte chargée", loaded).await;
    assert!(!first.contains(OLD_REASON), "{first}");
    tokio::time::sleep(Duration::from_millis(600)).await;
    while let Ok(buffer) = frames.try_recv() {
        let text = screen_text(&buffer);
        assert!(!text.contains(OLD_REASON), "{text}");
    }
    press(&keys, &[KeyCode::Char('q')]);
}

/// Un `ActionFailed` d'avant la session n'est pas affiché dans la barre d'état ; le moteur se ferme ensuite.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn old_events_are_not_replayed_at_startup() {
    let world = world_with_history().await;
    let mut terminal = Terminal::new(TestBackend::new(140, 30)).unwrap();
    let (keys, key_rx) = mpsc::unbounded_channel();
    let (frame_tx, frames) = mpsc::unbounded_channel();
    let hooks = LoopHooks {
        frames: Some(frame_tx),
        stop: None,
        ..LoopHooks::default()
    };
    let session = run_loop_with(&mut terminal, world.engine(), key_rx, hooks);
    let (outcome, ()) = tokio::join!(session, script(keys, frames));
    outcome.unwrap();
    world.close().await.unwrap();
}
