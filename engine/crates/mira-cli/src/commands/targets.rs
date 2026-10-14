//! Arguments positionnels désignant une entité par son identifiant opaque.

use clap::Args;
use mira_assembly::mira_vocabulary::{AccountId, ActionId, DraftId, MailboxId, MessageId};

/// Compte visé par une commande.
#[derive(Debug, Args)]
pub struct AccountArg {
    /// Identifiant du compte.
    pub account: String,
}

/// Boîte visée par une commande.
#[derive(Debug, Args)]
pub struct MailboxArg {
    /// Identifiant de la boîte.
    pub mailbox: String,
}

/// Message visé par une commande.
#[derive(Debug, Args)]
pub struct MessageArg {
    /// Identifiant du message.
    pub message: String,
}

/// Brouillon visé par une commande.
#[derive(Debug, Args)]
pub struct DraftArg {
    /// Identifiant du brouillon.
    pub draft: String,
}

/// Action de la file visée par une commande.
#[derive(Debug, Args)]
pub struct ActionArg {
    /// Identifiant de l'action.
    pub action: String,
}

impl AccountArg {
    /// Rend l'identifiant typé du compte.
    pub fn id(&self) -> AccountId {
        AccountId::from_raw(self.account.as_str())
    }
}

impl MailboxArg {
    /// Rend l'identifiant typé de la boîte.
    pub fn id(&self) -> MailboxId {
        MailboxId::from_raw(self.mailbox.as_str())
    }
}

impl MessageArg {
    /// Rend l'identifiant typé du message.
    pub fn id(&self) -> MessageId {
        MessageId::from_raw(self.message.as_str())
    }
}

impl DraftArg {
    /// Rend l'identifiant typé du brouillon.
    pub fn id(&self) -> DraftId {
        DraftId::from_raw(self.draft.as_str())
    }
}

impl ActionArg {
    /// Rend l'identifiant typé de l'action.
    pub fn id(&self) -> ActionId {
        ActionId::from_raw(self.action.as_str())
    }
}
