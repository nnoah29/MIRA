//! Fil de lecture du clavier (crossterm `poll`/`read`, sans flux asynchrone) vers la boucle.

use std::thread::JoinHandle;
use std::time::Duration;

use crossterm::event::{self, Event, KeyEvent};
use tokio::sync::mpsc::UnboundedSender;

const POLL: Duration = Duration::from_millis(100);

/// Lance le fil qui envoie chaque touche ; il s'arrête quand la boucle ne l'écoute plus ou que la lecture échoue.
pub fn spawn(sender: UnboundedSender<KeyEvent>) -> JoinHandle<()> {
    std::thread::spawn(move || read_keys(&sender))
}

/// Lit le terminal tant que la boucle écoute ; une erreur de lecture ferme le canal, donc la boucle.
fn read_keys(sender: &UnboundedSender<KeyEvent>) {
    while !sender.is_closed() {
        match next_key() {
            Ok(Some(key)) => {
                if sender.send(key).is_err() {
                    return;
                }
            }
            Ok(None) => {}
            Err(_) => return,
        }
    }
}

/// Attend une touche au plus `POLL` ; le redimensionnement est relu par la boucle à chaque tour, le reste est ignoré.
fn next_key() -> std::io::Result<Option<KeyEvent>> {
    if !event::poll(POLL)? {
        return Ok(None);
    }
    match event::read()? {
        Event::Key(key) => Ok(Some(key)),
        Event::Resize(..)
        | Event::FocusGained
        | Event::FocusLost
        | Event::Mouse(_)
        | Event::Paste(_) => Ok(None),
    }
}
