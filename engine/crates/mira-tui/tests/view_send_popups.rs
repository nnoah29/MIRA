//! Fenêtres de l'envoi : confirmation (destinataires et objet neutralisés) et décision sur un envoi incertain.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod action_fixtures;
mod compose_fixtures;
mod message_fixtures;
mod navigation_fixtures;
mod queue_fixtures;
mod reader_fixtures;
mod sidebar_fixtures;

use compose_fixtures::{compose, open_with, screen};
use mira_assembly::mira_vocabulary::{ActionState, AddressEntry};
use mira_tui::update::{ComposeIntent, Intent, QueueIntent};
use navigation_fixtures::press;

/// La confirmation nomme destinataires et objet ; noms et objet hostiles n'arrivent jamais bruts.
#[test]
fn confirm_send_popup_shows_recipients_and_subject_safely() {
    let mut draft = compose_fixtures::draft(
        "d1",
        &["bob@example.org"],
        "Re: \u{202e}Bonjour\u{2066}",
        "Corps",
    );
    draft.cc = vec![AddressEntry {
        name: Some("Ann\u{202e}\u{1b}".to_string()),
        address: "ann@example.org".to_string(),
    }];
    let mut state = open_with(draft);
    compose(&mut state, ComposeIntent::Send);
    let text = screen(&state, 100, 30).0;
    assert!(text.contains("Envoyer"), "{text}");
    assert!(
        text.contains("bob@example.org") && text.contains("ann@example.org"),
        "{text}"
    );
    assert!(text.contains("Re: Bonjour"), "{text}");
    for raw in ['\u{1b}', '\u{202e}', '\u{2066}'] {
        assert!(!text.contains(raw), "{raw:?} brut :\n{text}");
    }
}

/// La confirmation d'envoi tient dans les terminaux minuscules sans paniquer.
#[test]
fn confirm_send_popup_survives_tiny_terminals() {
    let mut state = compose_fixtures::open_reply();
    compose(&mut state, ComposeIntent::Send);
    for (width, height) in [(0, 0), (1, 1), (10, 3)] {
        screen(&state, width, height);
    }
}

/// La décision sur un envoi incertain dit l'enjeu et ses touches, objet neutralisé.
#[test]
fn uncertain_popup_explains_the_choice() {
    let mut state = action_fixtures::state_with_mailboxes();
    let doubtful =
        queue_fixtures::outbox_entry("s1", ActionState::Uncertain, "Contrat\u{1b}[5m", None);
    queue_fixtures::open_queue_with(&mut state, vec![doubtful], Vec::new());
    press(&mut state, Intent::Queue(QueueIntent::Resolve));
    let text = screen(&state, 100, 30).0;
    assert!(text.contains("peut-être été envoyé"), "{text}");
    assert!(
        text.contains("Contrat") && !text.contains('\u{1b}'),
        "{text}"
    );
    assert!(text.contains("renvoyer"), "{text}");
}
