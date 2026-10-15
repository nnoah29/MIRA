//! Bout en bout : nouveau message, Ctrl+Entrée, confirmation ; le faux SMTP le reçoit une seule fois.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod e2e_support;
mod support;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use e2e_support::{press, queued_actions, type_text, wait_for};
use mira_assembly::mira_vocabulary::{ActionKind, MailboxRole};
use mira_tui::app::run_loop::{LoopHooks, run_loop_with};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use support::send_world::SendWorld;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

/// Ouvre la boîte, commence un message à bob, objet et corps tapés, puis Ctrl+Entrée jusqu'à la confirmation.
async fn fill_and_ask(keys: &UnboundedSender<KeyEvent>, frames: &mut UnboundedReceiver<Buffer>) {
    wait_for(frames, "boîte listée", |s| s.contains("Réception")).await;
    press(keys, &[KeyCode::Enter]);
    wait_for(frames, "message listé", |s| s.contains("Facture")).await;
    press(keys, &[KeyCode::Char('c')]);
    wait_for(frames, "éditeur ouvert", |s| s.contains("Objet")).await;
    type_text(keys, "bob@example.org");
    press(keys, &[KeyCode::Enter, KeyCode::Enter, KeyCode::Enter]);
    type_text(keys, "Bonjour Bob");
    press(keys, &[KeyCode::Enter]);
    type_text(keys, "Salut");
    wait_for(frames, "texte tapé", |s| s.contains("Salut")).await;
    keys.send(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL))
        .unwrap();
    wait_for(frames, "enregistré", |s| {
        s.contains("Brouillon enregistré")
    })
    .await;
    keys.send(KeyEvent::new(KeyCode::Enter, KeyModifiers::CONTROL))
        .unwrap();
    wait_for(frames, "confirmation", |s| s.contains("Envoyer ce message")).await;
}

/// Confirme, voit l'envoi en file puis dans la vue « file », revient et quitte.
async fn send_script(keys: UnboundedSender<KeyEvent>, mut frames: UnboundedReceiver<Buffer>) {
    fill_and_ask(&keys, &mut frames).await;
    press(&keys, &[KeyCode::Enter]);
    wait_for(&mut frames, "en file", |s| s.contains("En file d'envoi")).await;
    press(&keys, &[KeyCode::Char('Q')]);
    let listed = |s: &str| s.contains("Bonjour Bob") && s.contains("en attente");
    wait_for(&mut frames, "file listée", listed).await;
    press(&keys, &[KeyCode::Esc, KeyCode::Char('q')]);
}

/// Confirme puis quitte aussitôt (Ctrl+c), sans attendre la réponse du moteur.
async fn quit_script(keys: UnboundedSender<KeyEvent>, mut frames: UnboundedReceiver<Buffer>) {
    fill_and_ask(&keys, &mut frames).await;
    press(&keys, &[KeyCode::Enter]);
    keys.send(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL))
        .unwrap();
}

/// Fait tourner la boucle sur ce monde avec ce script et attend la fin des deux.
async fn run_with<F, S>(world: &SendWorld, script: S)
where
    S: FnOnce(UnboundedSender<KeyEvent>, UnboundedReceiver<Buffer>) -> F,
    F: std::future::Future<Output = ()>,
{
    world
        .world
        .seed_mailbox("INBOX", MailboxRole::Inbox, &["Facture"])
        .unwrap();
    let mut terminal = Terminal::new(TestBackend::new(140, 30)).unwrap();
    let (keys, key_rx) = mpsc::unbounded_channel();
    let (frame_tx, frames) = mpsc::unbounded_channel();
    let hooks = LoopHooks {
        frames: Some(frame_tx),
        ..LoopHooks::default()
    };
    let session = run_loop_with(&mut terminal, world.world.engine(), key_rx, hooks);
    let (outcome, ()) = tokio::join!(session, script(keys, frames));
    outcome.unwrap();
}

/// Brouillon → envoi confirmé → le faux SMTP reçoit le message une seule fois, adressé à bob.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn e2e_compose_and_send() {
    let world = SendWorld::start().await;
    run_with(&world, send_script).await;
    assert_eq!(world.deliver().await, 1);
    assert_eq!(world.smtp.received()[0].recipients, ["bob@example.org"]);
    let SendWorld { world, smtp, clock } = world;
    drop((smtp, clock));
    world.close().await.unwrap();
}

/// Quitter juste après la confirmation ne perd pas l'envoi : il est dans la file du moteur et part une fois.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn quit_right_after_confirm_keeps_the_send() {
    let world = SendWorld::start().await;
    run_with(&world, quit_script).await;
    let account = world.world.account().unwrap();
    let actions = queued_actions(&world.world, account).await;
    let sends = actions
        .iter()
        .filter(|a| matches!(a.kind, ActionKind::Send { .. }))
        .count();
    assert_eq!(sends, 1, "{actions:?}");
    assert_eq!(world.deliver().await, 1);
}
