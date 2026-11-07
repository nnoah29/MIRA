//! Arrivée annoncée par le moteur, avant la lecture de l'en-tête du message.

use mira_assembly::mira_vocabulary::{AccountId, Event, EventKind, MessageId};

/// Message arrivé et verdict « silencieux » du moteur.
#[derive(Debug, Clone)]
pub(super) struct Pending {
    pub account: AccountId,
    pub message: MessageId,
    pub silenced: bool,
}

impl Pending {
    /// L'arrivée annoncée par cet événement, si c'en est une.
    pub(super) fn from_event(event: &Event) -> Option<Self> {
        match &event.kind {
            EventKind::NewMessageReceived {
                account,
                message,
                silenced,
                ..
            } => Some(Self {
                account: account.clone(),
                message: message.clone(),
                silenced: *silenced,
            }),
            _ => None,
        }
    }
}
