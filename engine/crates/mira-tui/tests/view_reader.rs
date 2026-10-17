//! Rendu du lecteur : en-têtes, corps neutralisé, invite hors ligne, pièces jointes.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod message_fixtures;
mod navigation_fixtures;
mod reader_fixtures;
mod sidebar_fixtures;

use mira_assembly::api::Response;
use mira_assembly::mira_vocabulary::MessageId;
use mira_tui::commands::Tag;
use mira_tui::update::{Intent, Msg, Reply, update};
use navigation_fixtures::press;
use reader_fixtures::{attachment, body, body_reply, list_state, offline_error, unread_message};
use sidebar_fixtures::rendered_text;

/// Sans message ouvert, le lecteur le dit.
#[test]
fn reader_without_message_shows_placeholder() {
    let state = list_state(vec![unread_message()]);
    assert!(rendered_text(&state).contains("Sélectionnez un message"));
}

/// Les en-têtes et le corps du message ouvert sont affichés.
#[test]
fn reader_shows_headers_and_body() {
    let mut state = list_state(vec![unread_message()]);
    press(&mut state, Intent::Open);
    update(&mut state, body_reply("m1", Ok(body("Texte du corps"))));
    let text = rendered_text(&state);
    assert!(text.contains("Objet : Bonjour"));
    assert!(text.contains("Bob <bob@example.org>"));
    assert!(text.contains("Texte du corps"));
}

/// Le corps et les en-têtes passent par le texte neutralisé.
#[test]
fn body_text_is_rendered_through_safe_text() {
    let mut hostile = unread_message();
    hostile.subject = "Ob\u{1b}[2Jjet".to_string();
    let mut state = list_state(vec![hostile]);
    press(&mut state, Intent::Open);
    update(
        &mut state,
        body_reply("m1", Ok(body("a\u{1b}]0;x\u{7}b\u{1b}[31mc\nligne2"))),
    );
    let text = rendered_text(&state);
    assert!(text.contains("abc"));
    assert!(text.contains("ligne2"));
    assert!(text.contains("Objet : Objet"));
    assert!(!text.contains('\u{1b}'));
    assert!(!text.contains("[2J") && !text.contains("[31m"));
}

/// L'invite hors ligne est affichée à la place du corps.
#[test]
fn offline_hint_is_displayed() {
    let mut state = list_state(vec![unread_message()]);
    press(&mut state, Intent::Open);
    update(&mut state, body_reply("m1", Err(offline_error())));
    let text = rendered_text(&state);
    assert!(text.contains("Corps indisponible hors ligne"));
    assert!(text.contains("Entrée pour"));
    assert!(text.contains("télécharger"));
}

/// Les noms de pièces jointes sont listés et neutralisés.
#[test]
fn attachment_names_are_listed_and_safe() {
    let mut state = list_state(vec![unread_message()]);
    press(&mut state, Intent::Open);
    let files = vec![attachment("x", "m1", "re\u{1b}[2Jlevé.pdf")];
    let reply = Reply {
        tag: Tag::Attachments(MessageId::from_raw("m1")),
        result: Ok(Response::Attachments(files)),
    };
    update(&mut state, Msg::Reply(reply));
    let text = rendered_text(&state);
    assert!(text.contains("relevé.pdf"));
    assert!(!text.contains("[2J"));
}
