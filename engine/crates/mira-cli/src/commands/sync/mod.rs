//! Commandes `mira sync …` : lancer, mettre en pause, état, remonter la plage, resynchroniser.

pub mod args;

use clap::Subcommand;
use mira_assembly::api::{Request, SyncRequest};
use mira_assembly::mira_vocabulary::{AccountId, MailboxId};
use mira_assembly::sync::BackgroundSync;

use super::waiting::WaitPolicy;

/// Commandes du domaine `sync`.
#[derive(Debug, Subcommand)]
pub enum SyncCommand {
    /// Synchronise tout, un compte ou une boîte, et attend la fin.
    Start(args::StartArgs),
    /// Met en pause la synchronisation d'un compte.
    Pause(args::PauseArgs),
    /// Affiche l'état de synchronisation de chaque compte.
    Status,
    /// Remonte la plage de synchronisation d'une boîte de quelques jours.
    ExtendWindow(args::ExtendWindowArgs),
    /// Relit entièrement une boîte depuis le serveur.
    Resync(args::ResyncArgs),
}

/// Traduit une commande `sync` en requête du contrat.
pub fn build_request(command: &SyncCommand) -> Request {
    let request = match command {
        SyncCommand::Start(args) => SyncRequest::StartSync {
            scope: args.scope(),
        },
        SyncCommand::Pause(args) => SyncRequest::PauseSync {
            account: AccountId::from_raw(args.account.as_str()),
        },
        SyncCommand::Status => SyncRequest::GetSyncStatus,
        SyncCommand::ExtendWindow(args) => SyncRequest::ExtendWindow {
            mailbox: MailboxId::from_raw(args.mailbox.as_str()),
            days: args.days,
        },
        SyncCommand::Resync(args) => SyncRequest::ResyncMailbox {
            mailbox: MailboxId::from_raw(args.mailbox.as_str()),
        },
    };
    Request::Sync(request)
}

/// Attente d'une commande `sync` : les opérations longues sont toujours attendues.
pub fn wait_policy(command: &SyncCommand) -> WaitPolicy {
    match command {
        SyncCommand::Start(args) => args.wait_limit.policy(),
        SyncCommand::ExtendWindow(args) => args.wait_limit.policy(),
        SyncCommand::Resync(args) => args.wait_limit.policy(),
        SyncCommand::Pause(_) | SyncCommand::Status => WaitPolicy::Immediate,
    }
}

/// Synchro de fond permise pendant la commande : seules les opérations longues l'utilisent.
pub fn background_mode(command: &SyncCommand, configured: BackgroundSync) -> BackgroundSync {
    match command {
        SyncCommand::Start(_) | SyncCommand::ExtendWindow(_) | SyncCommand::Resync(_) => configured,
        SyncCommand::Pause(_) | SyncCommand::Status => BackgroundSync::Off,
    }
}
