//! Table de lecture des événements du moteur : ce que chaque nature d'événement fait recharger.

use mira_assembly::mira_vocabulary::EventKind;

/// Cibles à recharger après un événement.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Reaction {
    pub content: bool,
    pub mailboxes: bool,
    pub status: bool,
    pub accounts: bool,
}

const NOTHING: Reaction = Reaction {
    content: false,
    mailboxes: false,
    status: false,
    accounts: false,
};
const CONTENT: Reaction = Reaction {
    content: true,
    ..NOTHING
};
const MAILBOXES: Reaction = Reaction {
    mailboxes: true,
    ..NOTHING
};
const STATUS: Reaction = Reaction {
    status: true,
    ..NOTHING
};
const ACCOUNTS: Reaction = Reaction {
    accounts: true,
    status: true,
    ..NOTHING
};
const MAIL: Reaction = Reaction {
    content: true,
    mailboxes: true,
    ..NOTHING
};
const ALL: Reaction = Reaction {
    content: true,
    mailboxes: true,
    status: true,
    accounts: false,
};

/// Rend ce qu'il faut recharger ; chaque variante est nommée pour qu'une nouvelle apparaisse en revue.
#[rustfmt::skip]
pub fn reaction_of(kind: &EventKind) -> Reaction {
    use EventKind as K;
    match kind {
        K::SyncFinished { .. } | K::ActionApplied { .. } | K::ActionCancelled { .. } | K::ActionFailed { .. } | K::MailboxResynchronized { .. } => ALL,
        K::NewMessageReceived { .. } | K::MessageUpdated { .. } | K::MessageDeleted { .. } => MAIL,
        K::ThreadUpdated { .. } => CONTENT,
        K::MailboxCreated { .. } | K::MailboxUpdated { .. } | K::MailboxDeleted { .. } | K::MailboxCountersUpdated { .. } => MAILBOXES,
        K::AccountAdded { .. } | K::AccountUpdated { .. } | K::AccountDeleted { .. } => ACCOUNTS,
        K::SyncStarted { .. } | K::SyncFailed { .. } | K::WentOffline { .. } | K::BackOnline { .. } | K::AccountStateChanged { .. }
        | K::ActionQueued { .. } | K::SendScheduled { .. } | K::SendDeparted { .. } | K::SendFailed { .. } | K::SentCopyStored { .. }
        | K::SendUncertain { .. } | K::QuotaFull { .. } | K::RateLimited { .. } | K::CertificateNeedsApproval { .. } => STATUS,
        K::EngineStarted | K::EngineStopping | K::EngineFatalError { .. } | K::SyncProgress { .. } | K::BodyAvailable { .. }
        | K::AttachmentProgress { .. } | K::AttachmentReady { .. } | K::Diagnostic { .. } | K::OperationProgress { .. }
        | K::OperationFinished { .. } | K::OperationFailed { .. } | K::OperationCancelled { .. } => NOTHING,
    }
}
