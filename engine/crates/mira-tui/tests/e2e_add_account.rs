//! Bout en bout : ajout d'un compte IMAP avec mot de passe et affichage des boîtes dans la barre latérale.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod e2e_support;
mod support;

use crossterm::event::KeyCode;
use e2e_support::{press, type_text, wait_for};
use mira_tui::app::run_loop::{LoopHooks, run_loop_with};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use support::imap_account::{PASSWORD, USER};
use support::world::TestWorld;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

/// Options de boucle pour les tests e2e : copie des images.
fn hooks(frames: UnboundedSender<Buffer>) -> LoopHooks {
    LoopHooks {
        frames: Some(frames),
        stop: None,
        ..LoopHooks::default()
    }
}

/// Saisit le mot de passe, envoie le formulaire et attend l'apparition des boîtes.
async fn submit_credentials(
    keys: &UnboundedSender<crossterm::event::KeyEvent>,
    frames: &mut UnboundedReceiver<Buffer>,
) {
    press(
        keys,
        &[
            KeyCode::Enter,
            KeyCode::Enter,
            KeyCode::Enter,
            KeyCode::Enter,
        ],
    );
    type_text(keys, PASSWORD);
    press(keys, &[KeyCode::Enter]);
    wait_for(frames, "compte ajouté et boîtes visibles", |s| {
        s.contains(USER) && (s.contains("Réception") || s.contains("INBOX"))
    })
    .await;
}

/// Script utilisateur : n, taper l'adresse, préparer, saisir le mot de passe, quitter.
async fn add_account_script(
    keys: UnboundedSender<crossterm::event::KeyEvent>,
    mut frames: UnboundedReceiver<Buffer>,
) {
    wait_for(&mut frames, "écran d'accueil sans compte", |s| {
        s.contains("Bienvenue dans MIRA") && s.contains("n pour ajouter un compte")
    })
    .await;
    press(&keys, &[KeyCode::Char('n')]);
    wait_for(&mut frames, "formulaire ouvert", |s| {
        s.contains("Ajouter un compte") && s.contains("Adresse :")
    })
    .await;
    type_text(&keys, USER);
    press(&keys, &[KeyCode::Enter]);
    wait_for(&mut frames, "serveurs préparés", |s| {
        s.contains("Mot de passe :")
    })
    .await;
    submit_credentials(&keys, &mut frames).await;
    press(&keys, &[KeyCode::Char('q')]);
}

/// Un compte ajouté par mot de passe contre le testkit apparaît dans la barre latérale avec ses boîtes.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn e2e_add_account() {
    let world = TestWorld::with_empty_imap().await;
    let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
    let (keys, key_rx) = mpsc::unbounded_channel();
    let (frame_tx, frames) = mpsc::unbounded_channel();
    let session = run_loop_with(&mut terminal, world.engine(), key_rx, hooks(frame_tx));
    let (outcome, ()) = tokio::join!(session, add_account_script(keys, frames));
    outcome.unwrap();
    world.close().await.unwrap();
}
