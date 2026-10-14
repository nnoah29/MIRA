//! `mira mailbox set-synced` : choisit les boîtes synchronisées d'un compte.

use clap::Args;
use mira_assembly::api::{MailboxRequest, Request};
use mira_assembly::mira_vocabulary::{AccountId, MailboxId};

/// Options de `mira mailbox set-synced`.
#[derive(Debug, Args)]
pub struct SetSyncedArgs {
    /// Identifiant du compte.
    pub account: String,
    /// Boîtes à synchroniser ; sans aucune, plus aucune boîte n'est synchronisée.
    pub mailboxes: Vec<String>,
}

/// Requête de choix des boîtes synchronisées.
pub fn request(args: &SetSyncedArgs) -> Request {
    Request::Mailbox(MailboxRequest::SetSyncedMailboxes {
        account: AccountId::from_raw(args.account.as_str()),
        mailboxes: args.mailboxes.iter().map(MailboxId::from_raw).collect(),
    })
}
