//! Bout en bout : le clavier se ferme pendant la rédaction ; le texte tapé atteint quand même le moteur.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod e2e_support;
mod support;

use crossterm::event::{KeyCode, KeyEvent};
use e2e_support::{press, type_text, wait_for};
use mira_assembly::api::{ComposeRequest, Request, Response, WriteResponse};
use mira_assembly::mira_vocabulary::{Draft, MailboxRole};
use mira_tui::app::run_loop::{LoopHooks, run_loop_with};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use support::world::TestWorld;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

/// Répond au message, tape « Salut » puis ferme le clavier (comme une fenêtre de terminal fermée).
async fn type_then_close(keys: UnboundedSender<KeyEvent>, mut frames: UnboundedReceiver<Buffer>) {
    wait_for(&mut frames, "boîte listée", |s| s.contains("Réception")).await;
    press(&keys, &[KeyCode::Enter]);
    wait_for(&mut frames, "message listé", |s| s.contains("Facture")).await;
    press(
        &keys,
        &[KeyCode::Char('j'), KeyCode::Enter, KeyCode::Char('r')],
    );
    wait_for(&mut frames, "éditeur", |s| s.contains("Re: Facture")).await;
    type_text(&keys, "Salut");
    wait_for(&mut frames, "texte tapé", |s| s.contains("Salut")).await;
    drop(keys);
}

/// Brouillons du compte de test, lus par le contrat du moteur.
async fn drafts_of(world: &TestWorld) -> Vec<Draft> {
    let request = Request::Compose(ComposeRequest::ListDrafts {
        account: world.account().unwrap(),
    });
    match world.engine().execute(request).await.unwrap() {
        Response::Write(WriteResponse::Drafts(drafts)) => drafts,
        other => panic!("réponse inattendue : {other:?}"),
    }
}

/// Clavier fermé avec un brouillon modifié : le moteur garde le texte tapé.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn closed_keyboard_still_saves_the_draft() {
    let world = TestWorld::with_imap_account().await;
    world
        .seed_mailbox("INBOX", MailboxRole::Inbox, &["Facture"])
        .unwrap();
    let mut terminal = Terminal::new(TestBackend::new(140, 30)).unwrap();
    let (keys, key_rx) = mpsc::unbounded_channel();
    let (frame_tx, frames) = mpsc::unbounded_channel();
    let hooks = LoopHooks {
        frames: Some(frame_tx),
        ..LoopHooks::default()
    };
    let session = run_loop_with(&mut terminal, world.engine(), key_rx, hooks);
    let (outcome, ()) = tokio::join!(session, type_then_close(keys, frames));
    outcome.unwrap();
    let drafts = drafts_of(&world).await;
    assert_eq!(drafts.len(), 1, "{drafts:?}");
    assert!(
        drafts[0].body_text.starts_with("Salut"),
        "{:?}",
        drafts[0].body_text
    );
    world.close().await.unwrap();
}
