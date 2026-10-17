//! Bout en bout : répondre depuis le lecteur, taper, enregistrer ; le moteur garde le brouillon modifié.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod e2e_support;
mod support;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use e2e_support::{press, type_text, wait_for};
use mira_assembly::api::{ComposeRequest, Request, Response, WriteResponse};
use mira_assembly::mira_vocabulary::{Draft, MailboxRole};
use mira_tui::app::run_loop::{LoopHooks, run_loop_with};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use support::world::TestWorld;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

/// Script : ouvrir la boîte puis le message, répondre, taper dans le corps, enregistrer, fermer, quitter.
async fn reply_script(keys: UnboundedSender<KeyEvent>, mut frames: UnboundedReceiver<Buffer>) {
    wait_for(&mut frames, "boîte listée", |s| s.contains("Réception")).await;
    press(&keys, &[KeyCode::Enter]);
    wait_for(&mut frames, "message listé", |s| s.contains("Facture")).await;
    press(
        &keys,
        &[KeyCode::Char('j'), KeyCode::Enter, KeyCode::Char('r')],
    );
    let editor = |s: &str| s.contains("Objet") && s.contains("Re: Facture");
    wait_for(&mut frames, "éditeur ouvert", editor).await;
    type_text(&keys, "Salut");
    wait_for(&mut frames, "texte tapé", |s| s.contains("Salut")).await;
    keys.send(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL))
        .unwrap();
    wait_for(&mut frames, "brouillon enregistré", |s| {
        s.contains("Brouillon enregistré")
    })
    .await;
    press(&keys, &[KeyCode::Esc]);
    wait_for(&mut frames, "éditeur fermé", |s| s.contains("Messages")).await;
    press(&keys, &[KeyCode::Char('q')]);
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

/// Répondre, taper « Salut » dans le corps, Ctrl+s : le moteur a un brouillon de réponse modifié.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn e2e_reply_edit_and_save_reaches_the_engine() {
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
    let (outcome, ()) = tokio::join!(session, reply_script(keys, frames));
    outcome.unwrap();
    let drafts = drafts_of(&world).await;
    assert_eq!(drafts.len(), 1, "{drafts:?}");
    assert_eq!(drafts[0].subject, "Re: Facture");
    assert!(
        drafts[0].body_text.starts_with("Salut"),
        "{:?}",
        drafts[0].body_text
    );
    assert_eq!(drafts[0].to[0].address, "bob@example.org");
    world.close().await.unwrap();
}
