//! Effets demandés par `update` et exécutés par la boucle d'événements.

use mira_assembly::api::Request;
use mira_assembly::mira_vocabulary::{
    AccountId, ActionId, Cursor, DraftId, IdempotencyKey, MessageId, ThreadId, ViewRef,
};

pub mod idempotency;
pub mod settings_tag;

pub use settings_tag::SettingsTag;

/// Étiquette qui relie une réponse du moteur à l'intention qui l'a demandée.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tag {
    Accounts,
    Mailboxes(AccountId),
    Messages {
        view: ViewRef,
        after: Option<Cursor>,
    },
    Body(MessageId),
    Attachments(MessageId),
    Thread(ThreadId),
    Action,
    UserAction(IdempotencyKey),
    Undo(ActionId),
    Search(String),
    StartSync,
    SyncStatus,
    Status,
    DraftCreated(IdempotencyKey),
    DraftSaved(DraftId),
    DraftDeleted(DraftId),
    Sent {
        draft: DraftId,
        key: IdempotencyKey,
    },
    Outbox {
        account: AccountId,
        round: u64,
    },
    QueueActions {
        account: AccountId,
        round: u64,
    },
    QueueChange(ActionId),
    Drafts(u64),
    DraftLoaded(u64),
    Settings(SettingsTag),
}

/// Effet de bord que la boucle doit réaliser après une transition.
#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum Command {
    Execute { tag: Tag, request: Request },
    Quit,
}
