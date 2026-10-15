//! Vue « file » : la sélection suit l'action (pas l'indice) quand la liste bouge ; une action disparue n'est jamais remplacée.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod action_fixtures;
mod message_fixtures;
mod navigation_fixtures;
mod queue_fixtures;
mod reader_fixtures;
mod sidebar_fixtures;

use action_fixtures::state_with_mailboxes;
use mira_assembly::api::{ComposeRequest, Request};
use mira_assembly::mira_vocabulary::{
    AccountId, ActionId, ActionState, Event, EventKind, EventSeq, Timestamp,
};
use mira_tui::state::AppState;
use mira_tui::update::{Intent, Msg, QueueIntent, update};
use navigation_fixtures::press;
use queue_fixtures::{actions_reply, change_command, open_queue_with, outbox_entry, outbox_reply};

/// Vue ouverte sur trois envois en attente A, B, C.
fn three_sends() -> AppState {
    let mut state = state_with_mailboxes();
    let entries = ["A", "B", "C"]
        .iter()
        .map(|id| outbox_entry(id, ActionState::Pending, &format!("Objet {id}"), None))
        .collect();
    open_queue_with(&mut state, entries, Vec::new());
    state
}

/// Un envoi part : l'événement relit la vue, qui ne contient plus que ces envois.
fn relist(state: &mut AppState, remaining: &[&str]) {
    let event = Msg::Event(Event {
        seq: EventSeq(3),
        at: Timestamp::from_millis(0),
        kind: EventKind::SendDeparted {
            account: AccountId::from_raw("a1"),
            action: ActionId::from_raw("A"),
        },
    });
    update(state, event);
    let entries = remaining
        .iter()
        .map(|id| outbox_entry(id, ActionState::Pending, &format!("Objet {id}"), None))
        .collect();
    update(state, outbox_reply("a1", 2, entries));
    update(state, actions_reply("a1", 2, Vec::new()));
}

/// Une entrée placée avant la sélection disparaît : la même action reste sélectionnée, `x` la vise elle.
#[test]
fn selection_follows_the_action_when_an_earlier_entry_leaves() {
    let mut state = three_sends();
    press(&mut state, Intent::MoveDown);
    relist(&mut state, &["B", "C"]);
    assert_eq!(state.queue.selected().unwrap().action.as_str(), "B");
    let cancel = Request::Compose(ComposeRequest::CancelSend {
        action: ActionId::from_raw("B"),
    });
    let commands = press(&mut state, Intent::Queue(QueueIntent::Cancel));
    assert_eq!(commands, vec![change_command("B", cancel)]);
    let shown = state.status.message.clone().unwrap();
    assert!(shown.contains("Envoi « Objet B » annulé"), "{shown}");
}

/// L'action sélectionnée a disparu : `x` et `r` n'annulent ni ne relancent rien d'autre, un message le dit.
#[test]
fn vanished_selection_never_targets_another_action() {
    let mut state = three_sends();
    relist(&mut state, &["B", "C"]);
    assert!(state.queue.selected().is_none());
    assert!(press(&mut state, Intent::Queue(QueueIntent::Cancel)).is_empty());
    let shown = state.status.message.clone().unwrap();
    assert!(shown.contains("Cette action n'existe plus"), "{shown}");
    assert!(press(&mut state, Intent::Queue(QueueIntent::Retry)).is_empty());
    press(&mut state, Intent::MoveDown);
    assert!(state.queue.selected().is_some());
}
