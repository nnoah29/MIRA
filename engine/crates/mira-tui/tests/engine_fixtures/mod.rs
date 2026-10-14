//! Fabriques d'événements, de réponses d'état et de commandes attendues pour les tests d'événements.
#![allow(dead_code, unused_imports)]

mod replies;

pub use replies::{status_reply, sync_status_reply};

use crate::sidebar_fixtures::account;
use mira_assembly::api::{
    AccountStatus, AccountSyncStatus, EngineRequest, EngineStatus, MailboxRequest, Request,
    Response, SyncRequest, SyncResponse,
};
use mira_assembly::mira_vocabulary::{
    AccountId, AccountState, Event, EventKind, EventSeq, OperationId, ProfileId, Timestamp,
};
use mira_tui::commands::{Command, Tag};
use mira_tui::update::{Msg, Reply};

/// Message d'événement portant cette nature.
pub fn event(kind: EventKind) -> Msg {
    Msg::Event(Event {
        seq: EventSeq(1),
        at: Timestamp::from_millis(0),
        kind,
    })
}

/// Fin de synchronisation du compte donné.
pub fn sync_finished(account: &str) -> Msg {
    event(EventKind::SyncFinished {
        account: AccountId::from_raw(account),
        operation: OperationId::from_raw("op1"),
    })
}

/// Lecture de l'état des comptes et de leur file d'actions.
pub fn status_command() -> Command {
    Command::Execute {
        tag: Tag::Status,
        request: Request::Engine(EngineRequest::GetStatus),
    }
}

/// Lecture de l'état de synchronisation.
pub fn sync_status_command() -> Command {
    Command::Execute {
        tag: Tag::SyncStatus,
        request: Request::Sync(SyncRequest::GetSyncStatus),
    }
}

/// Relecture des boîtes de ce compte.
pub fn mailboxes_command(id: &str) -> Command {
    let account = AccountId::from_raw(id);
    Command::Execute {
        tag: Tag::Mailboxes(account.clone()),
        request: Request::Mailbox(MailboxRequest::ListMailboxes { account }),
    }
}
