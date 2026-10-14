//! Lecteur : réponses tardives ignorées, retry du téléchargement, défilement borné.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod message_fixtures;
mod navigation_fixtures;
mod reader_fixtures;
mod sidebar_fixtures;

use message_fixtures::summary;
use mira_assembly::api::Response;
use mira_assembly::mira_vocabulary::{MessageId, ThreadId};
use mira_tui::commands::Tag;
use mira_tui::state::reader::ReaderContent;
use mira_tui::update::{Intent, Msg, Reply, update};
use navigation_fixtures::press;
use reader_fixtures::{
    attachment, body, body_reply, fetch_body_command, list_state, offline_error, thread_view,
    unread_message,
};

/// Le corps d'un message qui n'est plus ouvert est ignoré, succès comme erreur.
#[test]
fn late_body_of_previous_message_is_ignored() {
    let mut state = list_state(vec![unread_message(), summary("m2", "Autre", "Ann")]);
    press(&mut state, Intent::Open);
    press(&mut state, Intent::Back);
    press(&mut state, Intent::MoveDown);
    press(&mut state, Intent::Open);
    update(&mut state, body_reply("m1", Ok(body("ancien"))));
    assert_eq!(state.reader.content, ReaderContent::Loading);
    update(&mut state, body_reply("m1", Err(offline_error())));
    assert_eq!(state.reader.content, ReaderContent::Loading);
}

/// Entrée sur l'invite hors ligne demande le téléchargement du corps.
#[test]
fn enter_on_offline_hint_fetches_body() {
    let mut state = list_state(vec![unread_message()]);
    press(&mut state, Intent::Open);
    update(&mut state, body_reply("m1", Err(offline_error())));
    let commands = press(&mut state, Intent::Open);
    assert_eq!(commands, vec![fetch_body_command("m1")]);
    assert_eq!(state.reader.content, ReaderContent::Loading);
}

/// Les pièces jointes d'un autre message sont ignorées.
#[test]
fn late_attachments_are_ignored() {
    let mut state = list_state(vec![unread_message()]);
    press(&mut state, Intent::Open);
    let late = Reply {
        tag: Tag::Attachments(MessageId::from_raw("zz")),
        result: Ok(Response::Attachments(vec![attachment("x", "zz", "a.pdf")])),
    };
    update(&mut state, Msg::Reply(late));
    assert!(state.reader.attachments.is_empty());
}

/// Un fil qui n'est pas celui du message ouvert est ignoré.
#[test]
fn late_thread_is_ignored() {
    let mut message = unread_message();
    message.thread = Some(ThreadId::from_raw("t1"));
    let mut state = list_state(vec![message]);
    press(&mut state, Intent::Open);
    let late = Reply {
        tag: Tag::Thread(ThreadId::from_raw("t9")),
        result: Ok(Response::Thread(thread_view(
            "t9",
            vec![summary("m5", "X", "Y")],
        ))),
    };
    update(&mut state, Msg::Reply(late));
    assert!(state.reader.thread.is_empty());
}

/// j/k font défiler le corps dans le lecteur, sans dépasser ses bornes.
#[test]
fn j_and_k_scroll_body_within_bounds() {
    let mut state = list_state(vec![unread_message()]);
    press(&mut state, Intent::Open);
    update(&mut state, body_reply("m1", Ok(body("a\nb\nc"))));
    for _ in 0..10 {
        press(&mut state, Intent::MoveDown);
    }
    assert_eq!(state.reader.scroll, 2);
    press(&mut state, Intent::MoveUp);
    assert_eq!(state.reader.scroll, 1);
    press(&mut state, Intent::Top);
    assert_eq!(state.reader.scroll, 0);
}
