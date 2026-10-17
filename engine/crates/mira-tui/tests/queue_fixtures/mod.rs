//! Fiches de la boîte d'envoi et de la file d'actions, commandes et réponses de la vue « file ».
#![allow(dead_code, unused_imports)]

mod replies;

use mira_assembly::api::{ComposeRequest, OutboxEntry, QueueRequest, Request};
use mira_assembly::mira_vocabulary::{
    AccountId, ActionId, ActionKind, ActionState, DraftId, IdempotencyKey, PendingAction, Timestamp,
};
use mira_tui::commands::{Command, Tag};

pub use replies::{actions_reply, change_reply, open_queue_with, outbox_reply};

use mira_tui::state::AppState;

/// États demandés à la file par la vue.
pub const QUEUE_STATES: [ActionState; 4] = [
    ActionState::Pending,
    ActionState::InProgress,
    ActionState::Failed,
    ActionState::Uncertain,
];

/// Envoi de la boîte d'envoi : action `id`, brouillon `d-<id>`, cet état, cet objet, cette erreur.
pub fn outbox_entry(
    id: &str,
    state: ActionState,
    subject: &str,
    error: Option<&str>,
) -> OutboxEntry {
    OutboxEntry {
        action: ActionId::from_raw(id),
        draft: DraftId::from_raw(format!("d-{id}")),
        state,
        scheduled_for: Timestamp::from_millis(0),
        subject: subject.to_string(),
        last_error: error.map(str::to_string),
    }
}

/// Action en file du compte a1, de cette nature et dans cet état.
pub fn pending_action(id: &str, kind: ActionKind, state: ActionState) -> PendingAction {
    PendingAction {
        id: ActionId::from_raw(id),
        account: AccountId::from_raw("a1"),
        kind,
        targets: Vec::new(),
        idempotency_key: IdempotencyKey(format!("k-{id}")),
        state,
        attempts: 0,
        last_error: None,
        created_at: Timestamp::from_millis(0),
        not_before: Timestamp::from_millis(0),
    }
}

/// Lectures attendues à l'ouverture (ou au rafraîchissement) de la vue pour ce compte et ce tour.
pub fn list_commands(account: &str, round: u64) -> Vec<Command> {
    let id = AccountId::from_raw(account);
    vec![
        Command::Execute {
            tag: Tag::Outbox {
                account: id.clone(),
                round,
            },
            request: Request::Compose(ComposeRequest::ListOutbox {
                account: id.clone(),
            }),
        },
        Command::Execute {
            tag: Tag::QueueActions {
                account: id.clone(),
                round,
            },
            request: Request::Queue(QueueRequest::ListActions {
                account: id,
                states: QUEUE_STATES.to_vec(),
            }),
        },
    ]
}

/// Commande attendue d'une modification de la file visant cette action.
pub fn change_command(action: &str, request: Request) -> Command {
    Command::Execute {
        tag: Tag::QueueChange(ActionId::from_raw(action)),
        request,
    }
}

/// Ouvre la vue sur : envoi échoué s1, envoi en attente s2, archivage en attente x3 (et le doublon de s1 dans la file).
pub fn open_mixed_queue(state: &mut AppState) {
    let failed = outbox_entry("s1", ActionState::Failed, "Échoué", Some("550 refusé"));
    let waiting = outbox_entry("s2", ActionState::Pending, "Attente", None);
    let send = ActionKind::Send {
        draft: failed.draft.clone(),
    };
    let duplicate = pending_action("s1", send, ActionState::Failed);
    let archive = pending_action("x3", ActionKind::Archive, ActionState::Pending);
    open_queue_with(state, vec![failed, waiting], vec![duplicate, archive]);
}
