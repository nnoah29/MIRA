//! Types de la file d'actions locale : contexte, requête, intention résolue, étape élémentaire, enregistrement d'annulation.

use mira_vocabulary::{AccountId, ActionKind, IdempotencyKey, Mailbox, MailboxId, MessageId};
use serde::{Deserialize, Serialize};

use crate::events::EventSink;
use crate::ports::{Clock, IdGenerator};
use crate::storage::Database;

/// Délai d'annulation d'une action ordinaire, en millisecondes (5 s).
pub const UNDO_DELAY_MILLIS: i64 = 5_000;

/// Délai d'annulation d'un envoi sans date choisie, en millisecondes (10 s).
pub const SEND_UNDO_DELAY_MILLIS: i64 = 10_000;

/// Dépendances injectées des opérations locales : base, horloge, identifiants, événements.
#[derive(Debug, Clone, Copy)]
pub struct QueueContext<'a> {
    pub db: &'a Database,
    pub clock: &'a dyn Clock,
    pub ids: &'a dyn IdGenerator,
    pub events: &'a dyn EventSink,
}

/// Demande de mise en file d'une action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubmitRequest {
    pub account: AccountId,
    pub kind: ActionKind,
    pub targets: Vec<MessageId>,
    pub key: IdempotencyKey,
}

/// Effet local d'une action, résolu une fois pour toutes selon les particularités du service.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LocalIntent {
    SetSeen {
        seen: bool,
    },
    SetFlagged {
        flagged: bool,
    },
    MoveTo {
        mailbox: MailboxId,
    },
    DropFrom {
        mailbox: MailboxId,
    },
    AddTo {
        mailbox: MailboxId,
    },
    MarkDeleted,
    CreateMailbox {
        mailbox: Mailbox,
    },
    RenameMailbox {
        mailbox: MailboxId,
        name: String,
        display_name: String,
    },
    DeleteMailbox {
        mailbox: MailboxId,
    },
    EmptyMailbox {
        mailbox: MailboxId,
    },
    NoEffect,
}

/// Modification élémentaire et idempotente du miroir ; son inverse est une liste d'étapes du même type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LocalStep {
    SetSeen {
        message: MessageId,
        seen: bool,
    },
    SetFlagged {
        message: MessageId,
        flagged: bool,
    },
    PutAppearance {
        message: MessageId,
        mailbox: MailboxId,
        uid: Option<u32>,
        deleted: bool,
    },
    DropAppearance {
        message: MessageId,
        mailbox: MailboxId,
    },
    SetAppearanceDeleted {
        message: MessageId,
        mailbox: MailboxId,
        deleted: bool,
    },
    CreateMailbox {
        mailbox: Mailbox,
    },
    DropMailbox {
        mailbox: MailboxId,
    },
    RenameMailbox {
        mailbox: MailboxId,
        name: String,
        display_name: String,
    },
}

/// Ce qu'il faut garder d'une action pour rejouer ou défaire son effet local (colonne `undo_json`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UndoRecord {
    pub intent: LocalIntent,
    pub inverse: Vec<Vec<LocalStep>>,
}
