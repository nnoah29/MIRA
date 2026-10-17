//! Touches de la vue « file » et de la décision sur un envoi incertain : seules les leurs comptent.

use crossterm::event::{KeyCode, KeyEvent};

use super::keys::lookup;
use crate::update::{Intent, QueueIntent};

const QUEUE_KEYS: [(KeyCode, Intent); 12] = [
    (KeyCode::Char('j'), Intent::MoveDown),
    (KeyCode::Down, Intent::MoveDown),
    (KeyCode::Char('k'), Intent::MoveUp),
    (KeyCode::Up, Intent::MoveUp),
    (KeyCode::Char('g'), Intent::Top),
    (KeyCode::Char('G'), Intent::Bottom),
    (KeyCode::Char('r'), Intent::Queue(QueueIntent::Retry)),
    (KeyCode::Char('x'), Intent::Queue(QueueIntent::Cancel)),
    (KeyCode::Char('u'), Intent::Queue(QueueIntent::Resolve)),
    (KeyCode::Enter, Intent::Queue(QueueIntent::Resolve)),
    (KeyCode::Esc, Intent::Back),
    (KeyCode::Char('q'), Intent::Quit),
];

const UNCERTAIN_KEYS: [(KeyCode, Intent); 3] = [
    (KeyCode::Char('r'), Intent::Queue(QueueIntent::Resend)),
    (KeyCode::Char('n'), Intent::Queue(QueueIntent::DontResend)),
    (KeyCode::Esc, Intent::Cancel),
];

/// Touches de la vue « file ».
pub fn intent(key: KeyEvent) -> Option<Intent> {
    lookup(&QUEUE_KEYS, key)
}

/// Touches de la décision : `r` ou `n` explicites, Échap remet à plus tard ; Entrée ne décide rien.
pub fn uncertain_intent(key: KeyEvent) -> Option<Intent> {
    lookup(&UNCERTAIN_KEYS, key)
}
