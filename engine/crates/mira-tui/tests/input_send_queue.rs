//! Clavier de l'envoi et de la file : Ctrl+Entrée dans l'éditeur, `Q` hors édition, touches de la vue et de la décision.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod action_fixtures;
mod compose_fixtures;
mod input_fixtures;
mod message_fixtures;
mod navigation_fixtures;
mod queue_fixtures;
mod reader_fixtures;
mod sidebar_fixtures;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use input_fixtures::{code, key, map};
use mira_assembly::mira_vocabulary::ActionState;
use mira_tui::update::{ComposeIntent, EditKey, Intent, QueueIntent};
use navigation_fixtures::press;

/// Ctrl+Entrée réel (protocole clavier étendu) ou F5 demandent l'envoi ; Entrée reste un saut de ligne.
#[test]
fn ctrl_enter_requests_send_in_the_editor() {
    let state = compose_fixtures::open_reply();
    let ctrl_enter = KeyEvent::new(KeyCode::Enter, KeyModifiers::CONTROL);
    let send = Some(Intent::Compose(ComposeIntent::Send));
    assert_eq!(map(&state, ctrl_enter), send);
    assert_eq!(map(&state, code(KeyCode::F(5))), send);
    let newline = Some(Intent::Compose(ComposeIntent::Edit(EditKey::Newline)));
    assert_eq!(map(&state, code(KeyCode::Enter)), newline);
    assert_eq!(
        map(&state, key('Q')),
        Some(Intent::Compose(ComposeIntent::Edit(EditKey::Char('Q'))))
    );
}

/// `Q` ouvre la file depuis la liste et le lecteur ; Ctrl+Entrée hors de l'éditeur ne fait rien.
#[test]
fn queue_key_works_from_list_and_reader() {
    let mut state = action_fixtures::state_with_mailboxes();
    let open = Some(Intent::Queue(QueueIntent::Open));
    assert_eq!(map(&state, key('Q')), open);
    state.focus = mira_tui::state::Pane::Reader;
    assert_eq!(map(&state, key('Q')), open);
    let ctrl_enter = KeyEvent::new(KeyCode::Enter, KeyModifiers::CONTROL);
    assert_eq!(map(&state, ctrl_enter), None);
}

/// Dans la vue : j/k, r, x, u ou Entrée, Échap ; les touches d'action sur les messages n'y comptent pas.
#[test]
fn queue_view_keys() {
    let mut state = action_fixtures::state_with_mailboxes();
    queue_fixtures::open_queue_with(&mut state, Vec::new(), Vec::new());
    assert_eq!(map(&state, key('j')), Some(Intent::MoveDown));
    assert_eq!(map(&state, key('k')), Some(Intent::MoveUp));
    assert_eq!(
        map(&state, key('r')),
        Some(Intent::Queue(QueueIntent::Retry))
    );
    assert_eq!(
        map(&state, key('x')),
        Some(Intent::Queue(QueueIntent::Cancel))
    );
    assert_eq!(
        map(&state, key('u')),
        Some(Intent::Queue(QueueIntent::Resolve))
    );
    assert_eq!(
        map(&state, code(KeyCode::Enter)),
        Some(Intent::Queue(QueueIntent::Resolve))
    );
    assert_eq!(map(&state, code(KeyCode::Esc)), Some(Intent::Back));
    for c in ['e', 'd', 'D', 'c', 'Q', '/'] {
        assert_eq!(map(&state, key(c)), None, "{c}");
    }
}

/// Décision sur un envoi incertain : `r` renvoie, `n` ne renvoie pas, Échap remet à plus tard, Entrée ne décide rien.
#[test]
fn uncertain_popup_keys_require_an_explicit_choice() {
    let mut state = action_fixtures::state_with_mailboxes();
    let doubtful = queue_fixtures::outbox_entry("s1", ActionState::Uncertain, "Peut-être", None);
    queue_fixtures::open_queue_with(&mut state, vec![doubtful], Vec::new());
    press(&mut state, Intent::Queue(QueueIntent::Resolve));
    assert_eq!(
        map(&state, key('r')),
        Some(Intent::Queue(QueueIntent::Resend))
    );
    assert_eq!(
        map(&state, key('n')),
        Some(Intent::Queue(QueueIntent::DontResend))
    );
    assert_eq!(map(&state, code(KeyCode::Esc)), Some(Intent::Cancel));
    assert_eq!(map(&state, code(KeyCode::Enter)), None);
    assert_eq!(map(&state, key('y')), None);
}

/// La confirmation d'envoi accepte Entrée et `y`, refuse Échap et `n`, ignore Ctrl+Entrée.
#[test]
fn confirm_send_popup_keys() {
    let mut state = compose_fixtures::open_reply();
    press(&mut state, Intent::Compose(ComposeIntent::Send));
    assert_eq!(map(&state, code(KeyCode::Enter)), Some(Intent::Confirm));
    assert_eq!(map(&state, key('y')), Some(Intent::Confirm));
    assert_eq!(map(&state, code(KeyCode::Esc)), Some(Intent::Cancel));
    assert_eq!(map(&state, key('n')), Some(Intent::Cancel));
    let ctrl_enter = KeyEvent::new(KeyCode::Enter, KeyModifiers::CONTROL);
    assert_eq!(map(&state, ctrl_enter), None);
}
