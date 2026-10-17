//! Réponses du moteur aux lectures et modifications de la file, et vue ouverte garnie.
#![allow(dead_code)]

use mira_assembly::api::{OutboxEntry, Response, WriteResponse};
use mira_assembly::mira_vocabulary::{AccountId, ActionId, MiraError, PendingAction};
use mira_tui::commands::Tag;
use mira_tui::state::AppState;
use mira_tui::update::{Intent, Msg, QueueIntent, Reply, update};

/// Réponse à la lecture de la boîte d'envoi de ce compte pour ce tour.
pub fn outbox_reply(account: &str, round: u64, entries: Vec<OutboxEntry>) -> Msg {
    Msg::Reply(Reply {
        tag: Tag::Outbox {
            account: AccountId::from_raw(account),
            round,
        },
        result: Ok(Response::Write(WriteResponse::Outbox(entries))),
    })
}

/// Réponse à la lecture des actions en file de ce compte pour ce tour.
pub fn actions_reply(account: &str, round: u64, actions: Vec<PendingAction>) -> Msg {
    Msg::Reply(Reply {
        tag: Tag::QueueActions {
            account: AccountId::from_raw(account),
            round,
        },
        result: Ok(Response::Write(WriteResponse::QueuedActions(actions))),
    })
}

/// Réponse du moteur à une modification de la file visant cette action.
pub fn change_reply(action: &str, result: Result<(), MiraError>) -> Msg {
    Msg::Reply(Reply {
        tag: Tag::QueueChange(ActionId::from_raw(action)),
        result: result.map(|()| Response::Done),
    })
}

/// Ouvre la vue sur le compte a1 et lui donne ces envois et ces actions (premier tour).
pub fn open_queue_with(
    state: &mut AppState,
    entries: Vec<OutboxEntry>,
    actions: Vec<PendingAction>,
) {
    update(state, Msg::Intent(Intent::Queue(QueueIntent::Open)));
    update(state, outbox_reply("a1", 1, entries));
    update(state, actions_reply("a1", 1, actions));
}
