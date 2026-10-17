//! `mira account update` : modifie le nom, les serveurs ou la fenêtre de synchronisation.

use clap::Args;
use mira_assembly::api::{AccountChanges, AccountRequest};
use mira_assembly::mira_vocabulary::{AccountId, MailboxId, MiraError};

use super::server_args::ServerArgs;

/// Options de `mira account update`.
#[derive(Debug, Args)]
pub struct UpdateArgs {
    /// Identifiant du compte.
    pub account: String,
    /// Nouveau nom affiché.
    #[arg(long)]
    pub display_name: Option<String>,
    /// Nouvelle fenêtre de synchronisation, en jours.
    #[arg(long)]
    pub window_days: Option<u32>,
    /// Boîtes à synchroniser (répétable ; remplace la liste actuelle).
    #[arg(long = "mailbox")]
    pub mailboxes: Vec<String>,
    #[command(flatten)]
    pub servers: ServerArgs,
}

/// Construit la requête de modification de compte ; un champ absent n'est pas modifié.
pub fn request(args: &UpdateArgs) -> Result<AccountRequest, MiraError> {
    let synced = (!args.mailboxes.is_empty())
        .then(|| args.mailboxes.iter().map(MailboxId::from_raw).collect());
    Ok(AccountRequest::UpdateAccount {
        account: AccountId::from_raw(args.account.as_str()),
        changes: AccountChanges {
            display_name: args.display_name.clone(),
            servers: args.servers.settings()?,
            window_days: args.window_days,
            synced_mailboxes: synced,
        },
    })
}
