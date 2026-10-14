//! Pilotage de bout en bout : touches injectées dans la boucle et attente d'un écran attendu.
#![allow(dead_code)]

use std::time::Duration;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use mira_assembly::api::{QueueRequest, Request, Response, WriteResponse};
use mira_assembly::mira_vocabulary::{AccountId, PendingAction};
use ratatui::buffer::Buffer;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::support::world::TestWorld;

const SCREEN_WAIT: Duration = Duration::from_secs(15);

/// Texte d'une image de l'écran, ligne par ligne.
pub fn screen_text(buffer: &Buffer) -> String {
    let width = usize::from(buffer.area.width.max(1));
    let symbols: Vec<&str> = buffer.content().iter().map(|cell| cell.symbol()).collect();
    symbols
        .chunks(width)
        .map(|row| row.concat())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Attend une image de l'écran qui vérifie la condition et rend son texte ; échoue après le délai.
pub async fn wait_for(
    frames: &mut UnboundedReceiver<Buffer>,
    what: &str,
    accept: impl Fn(&str) -> bool,
) -> String {
    let mut last = String::new();
    let deadline = tokio::time::Instant::now() + SCREEN_WAIT;
    while let Ok(Some(buffer)) = tokio::time::timeout_at(deadline, frames.recv()).await {
        last = screen_text(&buffer);
        if accept(&last) {
            return last;
        }
    }
    panic!("écran attendu jamais vu : {what}\n--- dernier écran ---\n{last}");
}

/// Envoie ces touches à la boucle, dans l'ordre.
pub fn press(keys: &UnboundedSender<KeyEvent>, codes: &[KeyCode]) {
    for code in codes {
        keys.send(KeyEvent::new(*code, KeyModifiers::NONE)).unwrap();
    }
}

/// Tape ce texte touche par touche.
pub fn type_text(keys: &UnboundedSender<KeyEvent>, text: &str) {
    let codes: Vec<KeyCode> = text.chars().map(KeyCode::Char).collect();
    press(keys, &codes);
}

/// Actions de la file du compte de test, tous états confondus, lues par le contrat du moteur.
pub async fn queued_actions(world: &TestWorld, account: AccountId) -> Vec<PendingAction> {
    let request = Request::Queue(QueueRequest::ListActions {
        account,
        states: Vec::new(),
    });
    match world.engine().execute(request).await.unwrap() {
        Response::Write(WriteResponse::QueuedActions(actions)) => actions,
        other => panic!("réponse inattendue : {other:?}"),
    }
}
