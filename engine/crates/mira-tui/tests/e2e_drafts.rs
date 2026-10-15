//! Bout en bout : un brouillon enregistré puis fermé se rouvre par `B`, se modifie, et n'est jamais supprimé.
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

/// Ctrl+s puis attente de la confirmation du moteur.
async fn save(keys: &UnboundedSender<KeyEvent>, frames: &mut UnboundedReceiver<Buffer>) {
    keys.send(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL))
        .unwrap();
    wait_for(frames, "brouillon enregistré", |s| {
        s.contains("Brouillon enregistré")
    })
    .await;
}

/// Répondre au message, taper « Salut », enregistrer, fermer l'éditeur.
async fn save_a_reply(keys: &UnboundedSender<KeyEvent>, frames: &mut UnboundedReceiver<Buffer>) {
    wait_for(frames, "boîte listée", |s| s.contains("Réception")).await;
    press(keys, &[KeyCode::Enter]);
    wait_for(frames, "message listé", |s| s.contains("Facture")).await;
    press(
        keys,
        &[KeyCode::Char('j'), KeyCode::Enter, KeyCode::Char('r')],
    );
    wait_for(frames, "éditeur", |s| s.contains("Re: Facture")).await;
    type_text(keys, "Salut");
    wait_for(frames, "texte tapé", |s| s.contains("Salut")).await;
    save(keys, frames).await;
    press(keys, &[KeyCode::Esc]);
    wait_for(frames, "éditeur fermé", |s| s.contains("Messages")).await;
}

/// Enregistre une réponse ; `B`, Entrée, taper « X », enregistrer, fermer, quitter.
async fn roundtrip(keys: UnboundedSender<KeyEvent>, mut frames: UnboundedReceiver<Buffer>) {
    save_a_reply(&keys, &mut frames).await;
    press(&keys, &[KeyCode::Char('B')]);
    let listed = |s: &str| s.contains("Brouillons (1)") && s.contains("Re: Facture");
    wait_for(&mut frames, "vue Brouillons", listed).await;
    press(&keys, &[KeyCode::Enter]);
    let reopened = |s: &str| s.contains("Corps") && s.contains("Salut");
    wait_for(&mut frames, "brouillon rouvert", reopened).await;
    type_text(&keys, "X");
    wait_for(&mut frames, "modification", |s| s.contains("XSalut")).await;
    save(&keys, &mut frames).await;
    press(&keys, &[KeyCode::Esc]);
    wait_for(&mut frames, "retour à la liste", |s| {
        s.contains("Messages")
    })
    .await;
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

/// Le moteur voit la modification faite après réouverture, et le brouillon existe toujours.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn e2e_drafts_roundtrip() {
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
    let (outcome, ()) = tokio::join!(session, roundtrip(keys, frames));
    outcome.unwrap();
    let drafts = drafts_of(&world).await;
    assert_eq!(drafts.len(), 1, "{drafts:?}");
    assert_eq!(drafts[0].subject, "Re: Facture");
    assert!(
        drafts[0].body_text.starts_with("XSalut"),
        "{:?}",
        drafts[0].body_text
    );
    world.close().await.unwrap();
}
