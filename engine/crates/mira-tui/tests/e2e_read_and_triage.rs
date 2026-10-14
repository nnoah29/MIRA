//! Bout en bout : la vraie boucle sur un moteur de test, des touches injectées, un écran de test.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod e2e_support;
mod support;

use crossterm::event::KeyCode;
use e2e_support::{press, queued_actions, type_text, wait_for};
use mira_assembly::mira_vocabulary::{ActionKind, MailboxRole, MessageId};
use mira_tui::app::run_loop::run_loop_with_frames;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use support::world::TestWorld;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

const SUBJECTS: [&str; 3] = ["Facture", "Rendez-vous", "Vacances"];

/// Monde de test avec une boîte de réception de trois messages et une boîte d'archives vide.
async fn seeded_world() -> TestWorld {
    let world = TestWorld::with_imap_account().await;
    world
        .seed_mailbox("INBOX", MailboxRole::Inbox, &SUBJECTS)
        .unwrap();
    world
        .seed_mailbox("Archive", MailboxRole::Archive, &[])
        .unwrap();
    world
}

/// Script du tri en mode étroit : ouvrir la boîte, ouvrir le 2e message, l'archiver, quitter.
async fn triage_script(
    keys: UnboundedSender<crossterm::event::KeyEvent>,
    mut frames: UnboundedReceiver<Buffer>,
) {
    wait_for(&mut frames, "boîte listée", |s| s.contains("Réception")).await;
    press(&keys, &[KeyCode::Enter]);
    let all = |s: &str| s.contains("Messages") && SUBJECTS.iter().all(|t| s.contains(t));
    wait_for(&mut frames, "liste de 3 messages", all).await;
    press(&keys, &[KeyCode::Char('j'), KeyCode::Enter]);
    let reading = |s: &str| s.contains("Lecture") && s.contains("Rendez-vous");
    wait_for(&mut frames, "lecteur ouvert", reading).await;
    press(&keys, &[KeyCode::Char('e'), KeyCode::Esc]);
    let two = |s: &str| {
        s.contains("Messages")
            && s.contains("Facture")
            && s.contains("Vacances")
            && !s.contains("Rendez-vous")
    };
    wait_for(&mut frames, "liste de 2 messages", two).await;
    let queued = |s: &str| s.contains("2 actions en file") && two(s);
    wait_for(&mut frames, "lu et archivage en file", queued).await;
    press(&keys, &[KeyCode::Char('q')]);
}

/// Naviguer, ouvrir une boîte puis un message, archiver : la liste perd le message, le moteur a l'action.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn e2e_read_and_triage() {
    let world = seeded_world().await;
    let mut terminal = Terminal::new(TestBackend::new(90, 24)).unwrap();
    let (keys, key_rx) = mpsc::unbounded_channel();
    let (frame_tx, frames) = mpsc::unbounded_channel();
    let session = run_loop_with_frames(&mut terminal, world.engine(), key_rx, frame_tx);
    let (outcome, ()) = tokio::join!(session, triage_script(keys, frames));
    outcome.unwrap();
    let actions = queued_actions(&world, world.account().unwrap()).await;
    let archived = MessageId::from_raw("INBOX-2");
    let kinds: Vec<(ActionKind, Vec<MessageId>)> =
        actions.into_iter().map(|a| (a.kind, a.targets)).collect();
    let read = ActionKind::MarkRead { read: true };
    let expected = vec![
        (read, vec![archived.clone()]),
        (ActionKind::Archive, vec![archived]),
    ];
    assert_eq!(kinds, expected);
    world.close().await.unwrap();
}

/// Script de recherche : `/`, texte, Entrée, résultats, Échap rend la boîte, puis quitter.
async fn search_script(
    keys: UnboundedSender<crossterm::event::KeyEvent>,
    mut frames: UnboundedReceiver<Buffer>,
) {
    let all = |s: &str| SUBJECTS.iter().all(|t| s.contains(t));
    wait_for(&mut frames, "boîte chargée", all).await;
    press(&keys, &[KeyCode::Char('/')]);
    type_text(&keys, "facture");
    press(&keys, &[KeyCode::Enter]);
    let found = |s: &str| {
        s.contains("Recherche : facture") && s.contains("Facture") && !s.contains("Vacances")
    };
    wait_for(&mut frames, "résultats de recherche", found).await;
    press(&keys, &[KeyCode::Esc]);
    let restored = |s: &str| !s.contains("Recherche :") && all(s);
    wait_for(&mut frames, "boîte restaurée", restored).await;
    press(&keys, &[KeyCode::Char('q')]);
}

/// Chercher puis quitter la recherche par Échap rend la boîte d'avant ; le moteur se ferme ensuite.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn e2e_search_then_escape_restores_mailbox() {
    let world = seeded_world().await;
    let mut terminal = Terminal::new(TestBackend::new(140, 30)).unwrap();
    let (keys, key_rx) = mpsc::unbounded_channel();
    let (frame_tx, frames) = mpsc::unbounded_channel();
    let session = run_loop_with_frames(&mut terminal, world.engine(), key_rx, frame_tx);
    let (outcome, ()) = tokio::join!(session, search_script(keys, frames));
    outcome.unwrap();
    world.close().await.unwrap();
}
