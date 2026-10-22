//! Nature d'un événement : identifiants et petites valeurs, jamais de contenu ni de secret.

use serde::{Deserialize, Serialize};

use super::category::EventCategory;
use crate::domain::account::AccountState;
use crate::ids::kinds::{
    AccountId, ActionId, AttachmentId, MailboxId, MessageId, OperationId, RuleId, ThreadId,
};
use crate::time::Timestamp;

use EventKind as K;

/// Étape d'une synchronisation en cours.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncStep {
    Mailboxes,
    Headers,
    Bodies,
    Attachments,
    Changes,
    ActionsReplay,
}

/// Niveau d'un message de diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
}

/// Événement du moteur ; sérialisé avec un champ `type` en `snake_case`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventKind {
    EngineStarted,
    EngineStopping,
    EngineFatalError {
        code: String,
    },
    AccountAdded {
        account: AccountId,
    },
    AccountUpdated {
        account: AccountId,
    },
    AccountDeleted {
        account: AccountId,
    },
    AccountStateChanged {
        account: AccountId,
        state: AccountState,
    },
    SyncStarted {
        account: AccountId,
        operation: OperationId,
    },
    SyncProgress {
        account: AccountId,
        operation: OperationId,
        step: SyncStep,
        done: u64,
        total: u64,
    },
    SyncFinished {
        account: AccountId,
        operation: OperationId,
    },
    SyncFailed {
        account: AccountId,
        operation: OperationId,
        code: String,
    },
    WentOffline {
        account: AccountId,
    },
    BackOnline {
        account: AccountId,
    },
    MailboxCreated {
        account: AccountId,
        mailbox: MailboxId,
    },
    MailboxUpdated {
        account: AccountId,
        mailbox: MailboxId,
    },
    MailboxDeleted {
        account: AccountId,
        mailbox: MailboxId,
    },
    MailboxCountersUpdated {
        account: AccountId,
        mailbox: MailboxId,
        total: u64,
        unread: u64,
    },
    NewMessageReceived {
        account: AccountId,
        message: MessageId,
        mailbox: MailboxId,
        #[serde(default)]
        silenced: bool,
    },
    MessageUpdated {
        account: AccountId,
        message: MessageId,
    },
    MessageDeleted {
        account: AccountId,
        message: MessageId,
    },
    BodyAvailable {
        account: AccountId,
        message: MessageId,
    },
    AttachmentProgress {
        account: AccountId,
        attachment: AttachmentId,
        operation: OperationId,
        done: u64,
        total: u64,
    },
    AttachmentReady {
        account: AccountId,
        attachment: AttachmentId,
        operation: OperationId,
    },
    ThreadUpdated {
        account: AccountId,
        thread: ThreadId,
    },
    ActionQueued {
        account: AccountId,
        action: ActionId,
    },
    ActionApplied {
        account: AccountId,
        action: ActionId,
    },
    ActionFailed {
        account: AccountId,
        action: ActionId,
        code: String,
        reason: String,
    },
    ActionCancelled {
        account: AccountId,
        action: ActionId,
    },
    SendScheduled {
        account: AccountId,
        action: ActionId,
        at: Timestamp,
    },
    SendDeparted {
        account: AccountId,
        action: ActionId,
    },
    SendFailed {
        account: AccountId,
        action: ActionId,
        code: String,
    },
    SentCopyStored {
        account: AccountId,
        action: ActionId,
    },
    SendUncertain {
        account: AccountId,
        action: ActionId,
    },
    QuotaFull {
        account: AccountId,
    },
    CertificateNeedsApproval {
        account: AccountId,
        host: String,
    },
    RateLimited {
        account: AccountId,
    },
    MailboxResynchronized {
        account: AccountId,
        mailbox: MailboxId,
    },
    Diagnostic {
        level: LogLevel,
        target: String,
        text: String,
    },
    OperationProgress {
        operation: OperationId,
        done: u64,
        total: u64,
    },
    OperationFinished {
        operation: OperationId,
    },
    OperationFailed {
        operation: OperationId,
        code: String,
    },
    OperationCancelled {
        operation: OperationId,
    },
    RuleChanged {
        rule: RuleId,
    },
    RuleApplied {
        account: AccountId,
        rule: RuleId,
        message: MessageId,
    },
}

impl EventKind {
    /// Donne la catégorie de l'événement (un `match` exhaustif sur toutes les variantes).
    #[rustfmt::skip]
    pub fn category(&self) -> EventCategory {
        match self {
            K::EngineStarted | K::EngineStopping | K::EngineFatalError { .. } | K::OperationProgress { .. }
            | K::OperationFinished { .. } | K::OperationFailed { .. } | K::OperationCancelled { .. } => EventCategory::Engine,
            K::AccountAdded { .. } | K::AccountUpdated { .. } | K::AccountDeleted { .. } | K::AccountStateChanged { .. } => EventCategory::Account,
            K::SyncStarted { .. } | K::SyncProgress { .. } | K::SyncFinished { .. } | K::SyncFailed { .. }
            | K::WentOffline { .. } | K::BackOnline { .. } => EventCategory::Sync,
            K::MailboxCreated { .. } | K::MailboxUpdated { .. } | K::MailboxDeleted { .. } | K::MailboxCountersUpdated { .. } => EventCategory::Mailbox,
            K::NewMessageReceived { .. } | K::MessageUpdated { .. } | K::MessageDeleted { .. } | K::BodyAvailable { .. }
            | K::AttachmentProgress { .. } | K::AttachmentReady { .. } => EventCategory::Message,
            K::ThreadUpdated { .. } => EventCategory::Thread,
            K::ActionQueued { .. } | K::ActionApplied { .. } | K::ActionFailed { .. } | K::ActionCancelled { .. } => EventCategory::Action,
            K::SendScheduled { .. } | K::SendDeparted { .. } | K::SendFailed { .. } | K::SentCopyStored { .. }
            | K::SendUncertain { .. } => EventCategory::Send,
            K::QuotaFull { .. } | K::CertificateNeedsApproval { .. } | K::RateLimited { .. } | K::MailboxResynchronized { .. } => EventCategory::Alert,
            K::Diagnostic { .. } => EventCategory::Log,
            K::RuleChanged { .. } | K::RuleApplied { .. } => EventCategory::Rule,
        }
    }

    /// Donne le compte concerné, ou rien pour les événements du moteur, du journal et des opérations.
    #[rustfmt::skip]
    pub fn account(&self) -> Option<&AccountId> {
        match self {
            K::AccountAdded { account } | K::AccountUpdated { account } | K::AccountDeleted { account }
            | K::AccountStateChanged { account, .. } | K::SyncStarted { account, .. } | K::SyncProgress { account, .. }
            | K::SyncFinished { account, .. } | K::SyncFailed { account, .. } | K::WentOffline { account }
            | K::BackOnline { account } | K::MailboxCreated { account, .. } | K::MailboxUpdated { account, .. }
            | K::MailboxDeleted { account, .. } | K::MailboxCountersUpdated { account, .. }
            | K::NewMessageReceived { account, .. } | K::MessageUpdated { account, .. } | K::MessageDeleted { account, .. }
            | K::BodyAvailable { account, .. } | K::AttachmentProgress { account, .. } | K::AttachmentReady { account, .. }
            | K::ThreadUpdated { account, .. } | K::ActionQueued { account, .. } | K::ActionApplied { account, .. }
            | K::ActionFailed { account, .. } | K::ActionCancelled { account, .. } | K::SendScheduled { account, .. }
            | K::SendDeparted { account, .. } | K::SendFailed { account, .. } | K::SentCopyStored { account, .. }
            | K::SendUncertain { account, .. } | K::QuotaFull { account } | K::CertificateNeedsApproval { account, .. }
            | K::RateLimited { account } | K::MailboxResynchronized { account, .. }
            | K::RuleApplied { account, .. } => Some(account),
            K::EngineStarted | K::EngineStopping | K::EngineFatalError { .. } | K::Diagnostic { .. }
            | K::OperationProgress { .. } | K::OperationFinished { .. } | K::OperationFailed { .. }
            | K::OperationCancelled { .. } | K::RuleChanged { .. } => None,
        }
    }
}
