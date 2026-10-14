//! Bout en bout : ce qui change pendant le balayage du journal au démarrage n'est pas perdu.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod e2e_support;
mod support;

use std::sync::Arc;
use std::time::Duration;

use crossterm::event::KeyCode;
use e2e_support::{press, wait_for};
use mira_assembly::mira_vocabulary::MailboxRole;
use mira_tui::app::event_cursor::JournalProbe;
use mira_tui::app::run_loop::{LoopHooks, run_loop_with};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use support::journal::add_mailbox_with_event;
use support::world::TestWorld;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

/// Couture : après la 1re page du journal, laisse le temps aux lectures déjà parties puis crée une boîte (avec son événement).
fn probe_adding_mailbox(world: &TestWorld) -> JournalProbe {
    let database = world.database_path().unwrap();
    let account = world.account().unwrap();
    JournalProbe(Arc::new(move |page| {
        if page == 0 {
            std::thread::sleep(Duration::from_millis(400));
            add_mailbox_with_event(&database, &account, "Projets").unwrap();
        }
    }))
}

/// La boîte créée pendant le balayage finit à l'écran, puis on quitte.
async fn script(
    keys: UnboundedSender<crossterm::event::KeyEvent>,
    mut frames: UnboundedReceiver<Buffer>,
) {
    let shown = |s: &str| s.contains("Réception") && s.contains("Projets");
    wait_for(&mut frames, "boîte créée pendant le balayage", shown).await;
    press(&keys, &[KeyCode::Char('q')]);
}

/// Journal de plusieurs pages, boîte créée pendant le balayage : l'écran la montre et le moteur se ferme.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn change_during_journal_scan_is_not_lost() {
    let world = TestWorld::with_imap_account().await;
    world
        .seed_mailbox("INBOX", MailboxRole::Inbox, &["Facture"])
        .unwrap();
    world.seed_old_events(2_500).unwrap();
    let mut terminal = Terminal::new(TestBackend::new(140, 30)).unwrap();
    let (keys, key_rx) = mpsc::unbounded_channel();
    let (frame_tx, frames) = mpsc::unbounded_channel();
    let hooks = LoopHooks {
        frames: Some(frame_tx),
        stop: None,
        journal_probe: Some(probe_adding_mailbox(&world)),
    };
    let session = run_loop_with(&mut terminal, world.engine(), key_rx, hooks);
    let (outcome, ()) = tokio::join!(session, script(keys, frames));
    outcome.unwrap();
    world.close().await.unwrap();
}
