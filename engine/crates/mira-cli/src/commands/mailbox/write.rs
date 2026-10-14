//! Écritures sur les boîtes : création, renommage, suppression, vidage (file d'actions).

use clap::Args;
use mira_assembly::api::{ActionRequest, Request};
use mira_assembly::mira_vocabulary::{AccountId, MailboxId};

use crate::argument_context::ArgumentContext;
use crate::commands::key_args::KeyArgs;
use crate::commands::waiting::WaitArgs;

/// Options de `mira mailbox create`.
#[derive(Debug, Args)]
pub struct CreateArgs {
    /// Identifiant du compte.
    pub account: String,
    /// Nom de la nouvelle boîte.
    pub name: String,
    /// Identifiant de la boîte parente.
    #[arg(long)]
    pub parent: Option<String>,
    #[command(flatten)]
    pub key: KeyArgs,
    #[command(flatten)]
    pub wait: WaitArgs,
}

/// Options de `mira mailbox rename`.
#[derive(Debug, Args)]
pub struct RenameArgs {
    /// Identifiant de la boîte.
    pub mailbox: String,
    /// Nouveau nom de la boîte.
    pub name: String,
    #[command(flatten)]
    pub key: KeyArgs,
    #[command(flatten)]
    pub wait: WaitArgs,
}

/// Options de `mira mailbox delete`.
#[derive(Debug, Args)]
pub struct DeleteArgs {
    /// Identifiant de la boîte.
    pub mailbox: String,
    #[command(flatten)]
    pub key: KeyArgs,
    #[command(flatten)]
    pub wait: WaitArgs,
}

/// Options de `mira mailbox empty`.
#[derive(Debug, Args)]
pub struct EmptyArgs {
    /// Identifiant de la boîte.
    pub mailbox: String,
    #[command(flatten)]
    pub key: KeyArgs,
    #[command(flatten)]
    pub wait: WaitArgs,
}

/// Requête de création de boîte.
pub fn create_request(args: &CreateArgs, context: &ArgumentContext) -> Request {
    Request::Action(ActionRequest::CreateMailbox {
        account: AccountId::from_raw(args.account.as_str()),
        name: args.name.clone(),
        parent: args.parent.as_deref().map(MailboxId::from_raw),
        idempotency_key: args.key.idempotency_key(context),
    })
}

/// Requête de renommage de boîte.
pub fn rename_request(args: &RenameArgs, context: &ArgumentContext) -> Request {
    Request::Action(ActionRequest::RenameMailbox {
        mailbox: MailboxId::from_raw(args.mailbox.as_str()),
        name: args.name.clone(),
        idempotency_key: args.key.idempotency_key(context),
    })
}

/// Requête de suppression de boîte.
pub fn delete_request(args: &DeleteArgs, context: &ArgumentContext) -> Request {
    Request::Action(ActionRequest::DeleteMailbox {
        mailbox: MailboxId::from_raw(args.mailbox.as_str()),
        idempotency_key: args.key.idempotency_key(context),
    })
}

/// Requête de vidage de boîte.
pub fn empty_request(args: &EmptyArgs, context: &ArgumentContext) -> Request {
    Request::Action(ActionRequest::EmptyMailbox {
        mailbox: MailboxId::from_raw(args.mailbox.as_str()),
        idempotency_key: args.key.idempotency_key(context),
    })
}
