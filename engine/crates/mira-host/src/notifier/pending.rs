//! Arrivée annoncée par le moteur, avant la lecture de l'en-tête du message.

use mira_assembly::mira_vocabulary::{AccountId, Event, EventKind, MailboxId, MessageId};

/// Message arrivé et verdict « silencieux » du moteur.
#[derive(Debug, Clone)]
pub(super) struct Pending {
    pub account: AccountId,
    pub message: MessageId,
    pub mailbox: MailboxId,
    pub silenced: bool,
}

impl Pending {
    /// L'arrivée annoncée par cet événement, si c'en est une.
    pub(super) fn from_event(event: &Event) -> Option<Self> {
        match &event.kind {
            EventKind::NewMessageReceived {
                account,
                message,
                mailbox,
                silenced,
            } => Some(Self {
                account: account.clone(),
                message: message.clone(),
                mailbox: mailbox.clone(),
                silenced: *silenced,
            }),
            _ => None,
        }
    }
}
