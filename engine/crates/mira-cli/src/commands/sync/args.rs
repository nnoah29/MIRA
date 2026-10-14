//! Options des commandes `mira sync …` : portée, boîte, jours, délai d'attente.

use clap::Args;
use mira_assembly::api::SyncScope;
use mira_assembly::mira_vocabulary::{AccountId, MailboxId};

use crate::commands::waiting::OperationWaitArgs;

/// Options de `mira sync start`.
#[derive(Debug, Args)]
pub struct StartArgs {
    /// Ne synchronise que ce compte (par défaut, tous les comptes actifs).
    #[arg(long, conflicts_with = "mailbox")]
    pub account: Option<String>,
    /// Ne synchronise que cette boîte.
    #[arg(long)]
    pub mailbox: Option<String>,
    /// Accepté pour la lisibilité : un CLI de courte durée attend toujours la fin de la synchro.
    #[arg(long)]
    pub wait: bool,
    #[command(flatten)]
    pub wait_limit: OperationWaitArgs,
}

/// Options de `mira sync pause`.
#[derive(Debug, Args)]
pub struct PauseArgs {
    /// Compte dont la synchronisation s'arrête.
    #[arg(long)]
    pub account: String,
}

/// Options de `mira sync extend-window`.
#[derive(Debug, Args)]
pub struct ExtendWindowArgs {
    /// Boîte dont la plage de synchronisation remonte.
    #[arg(long)]
    pub mailbox: String,
    /// Nombre de jours à remonter en plus.
    #[arg(long)]
    pub days: u32,
    #[command(flatten)]
    pub wait_limit: OperationWaitArgs,
}

/// Options de `mira sync resync`.
#[derive(Debug, Args)]
pub struct ResyncArgs {
    /// Boîte à relire entièrement.
    #[arg(long)]
    pub mailbox: String,
    #[command(flatten)]
    pub wait_limit: OperationWaitArgs,
}

impl StartArgs {
    /// Portée demandée : une boîte, un compte, ou tous les comptes.
    pub fn scope(&self) -> SyncScope {
        match (&self.mailbox, &self.account) {
            (Some(mailbox), _) => SyncScope::Mailbox {
                mailbox: MailboxId::from_raw(mailbox.as_str()),
            },
            (None, Some(account)) => SyncScope::Account {
                account: AccountId::from_raw(account.as_str()),
            },
            (None, None) => SyncScope::All,
        }
    }
}
