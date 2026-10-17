//! Rendu du fil de conversation dans le lecteur, du plus ancien au plus récent.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod message_fixtures;
mod navigation_fixtures;
mod reader_fixtures;
mod sidebar_fixtures;

use message_fixtures::summary;
use mira_assembly::api::Response;
use mira_assembly::mira_vocabulary::ThreadId;
use mira_tui::commands::Tag;
use mira_tui::update::{Intent, Msg, Reply, update};
use navigation_fixtures::press;
use reader_fixtures::{list_state, thread_view, unread_message};
use sidebar_fixtures::rendered_text;

/// Les messages du fil sont listés dans l'ordre reçu, le plus ancien d'abord.
#[test]
fn thread_view_lists_messages_oldest_first() {
    let mut message = unread_message();
    message.thread = Some(ThreadId::from_raw("t1"));
    let mut state = list_state(vec![message]);
    press(&mut state, Intent::Open);
    let older = summary("m0", "Alpha", "Ann");
    let newer = summary("m1", "Beta", "Bob");
    let reply = Reply {
        tag: Tag::Thread(ThreadId::from_raw("t1")),
        result: Ok(Response::Thread(thread_view("t1", vec![older, newer]))),
    };
    update(&mut state, Msg::Reply(reply));
    let text = rendered_text(&state);
    let first = text.find("Alpha").unwrap();
    let second = text.find("Beta").unwrap();
    assert!(first < second);
    assert!(text.contains("Fil (2 messages)"));
}

/// L'ouverture d'un message d'un fil demande ce fil.
#[test]
fn opening_threaded_message_requests_thread() {
    let mut message = unread_message();
    message.thread = Some(ThreadId::from_raw("t1"));
    let mut state = list_state(vec![message]);
    let commands = press(&mut state, Intent::Open);
    assert!(commands.iter().any(|c| matches!(
        c,
        mira_tui::commands::Command::Execute {
            tag: Tag::Thread(_),
            ..
        }
    )));
}
