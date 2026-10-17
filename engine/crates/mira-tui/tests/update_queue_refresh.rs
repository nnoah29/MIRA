//! Vue « file » : réponses tardives ignorées, rafraîchissement par les événements d'action, sans rafale.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod action_fixtures;
mod message_fixtures;
mod navigation_fixtures;
mod queue_fixtures;
mod reader_fixtures;
mod sidebar_fixtures;

use action_fixtures::state_with_mailboxes;
use mira_assembly::mira_vocabulary::{
    AccountId, ActionId, ActionState, Event, EventKind, EventSeq, Timestamp,
};
use mira_tui::update::{Intent, Msg, QueueIntent, update};
use navigation_fixtures::press;
use queue_fixtures::{
    actions_reply, change_reply, list_commands, open_queue_with, outbox_entry, outbox_reply,
};

/// Événement d'action du compte `account`.
fn applied(account: &str) -> Msg {
    Msg::Event(Event {
        seq: EventSeq(1),
        at: Timestamp::from_millis(0),
        kind: EventKind::ActionApplied {
            account: AccountId::from_raw(account),
            action: ActionId::from_raw("x"),
        },
    })
}

/// Réponses d'un tour périmé, d'un autre compte ou arrivées vue fermée : ignorées.
#[test]
fn late_queue_replies_are_ignored() {
    let mut state = state_with_mailboxes();
    press(&mut state, Intent::Queue(QueueIntent::Open));
    press(&mut state, Intent::Back);
    let entry = || vec![outbox_entry("s1", ActionState::Pending, "Vieux", None)];
    update(&mut state, outbox_reply("a1", 1, entry()));
    assert!(state.queue.entries().is_empty());
    assert_eq!(
        press(&mut state, Intent::Queue(QueueIntent::Open)),
        list_commands("a1", 2)
    );
    update(&mut state, outbox_reply("a1", 1, entry()));
    update(&mut state, outbox_reply("a2", 2, entry()));
    update(&mut state, actions_reply("a2", 2, Vec::new()));
    assert!(state.queue.entries().is_empty());
    assert!(state.queue.is_loading());
    update(&mut state, outbox_reply("a1", 2, entry()));
    update(&mut state, actions_reply("a1", 2, Vec::new()));
    assert_eq!(state.queue.entries().len(), 1);
    assert!(!state.queue.is_loading());
}

/// Un événement d'action du compte relit la file ; pendant une lecture, un seul tour de plus à la fin.
#[test]
fn action_events_refresh_the_open_queue_without_bursts() {
    let mut state = state_with_mailboxes();
    open_queue_with(&mut state, Vec::new(), Vec::new());
    assert!(
        update(&mut state, applied("a2"))
            .iter()
            .all(|c| !list_commands("a1", 2).contains(c))
    );
    let first = update(&mut state, applied("a1"));
    assert!(
        list_commands("a1", 2).iter().all(|c| first.contains(c)),
        "{first:?}"
    );
    let during = update(&mut state, applied("a1"));
    assert!(list_commands("a1", 3).iter().all(|c| !during.contains(c)));
    update(&mut state, outbox_reply("a1", 2, Vec::new()));
    let last = update(&mut state, actions_reply("a1", 2, Vec::new()));
    assert_eq!(last, list_commands("a1", 3));
}

/// La réponse à une modification de la file relit la vue ouverte ; un échec est affiché.
#[test]
fn queue_change_reply_refreshes_or_reports() {
    let mut state = state_with_mailboxes();
    open_queue_with(&mut state, Vec::new(), Vec::new());
    assert_eq!(
        update(&mut state, change_reply("s1", Ok(()))),
        list_commands("a1", 2)
    );
    let error = reader_fixtures::offline_error();
    update(&mut state, change_reply("s1", Err(error)));
    assert!(state.status.message.is_some());
}

/// Un événement d'envoi incertain prévient dans la barre d'état sans rien renvoyer ni ouvrir de fenêtre.
#[test]
fn uncertain_event_only_warns() {
    let mut state = state_with_mailboxes();
    let event = Msg::Event(Event {
        seq: EventSeq(2),
        at: Timestamp::from_millis(0),
        kind: EventKind::SendUncertain {
            account: AccountId::from_raw("a1"),
            action: ActionId::from_raw("s1"),
        },
    });
    let commands = update(&mut state, event);
    assert!(
        format!("{commands:?}")
            .find("ResolveUncertainSend")
            .is_none()
    );
    assert_eq!(state.popup, mira_tui::state::Popup::Closed);
    let shown = state.status.message.clone().unwrap();
    assert!(shown.contains("incertain"), "{shown}");
}

/// Le compte sélectionné change (liste des comptes relue) : la vue se ferme et ses réponses tardives sont ignorées.
#[test]
fn account_change_closes_the_queue_and_ignores_its_replies() {
    let mut state = state_with_mailboxes();
    press(&mut state, Intent::Queue(QueueIntent::Open));
    let accounts = vec![sidebar_fixtures::account("a2")];
    let reply = Msg::Reply(mira_tui::update::Reply {
        tag: mira_tui::commands::Tag::Accounts,
        result: Ok(mira_assembly::api::Response::Accounts(accounts)),
    });
    update(&mut state, reply);
    assert!(!state.queue.is_open());
    let entry = vec![outbox_entry("s1", ActionState::Pending, "Vieux", None)];
    update(&mut state, outbox_reply("a1", 1, entry));
    assert!(state.queue.entries().is_empty());
}
