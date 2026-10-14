//! Déroulement d'une commande sur un moteur ouvert : une requête, ou une suite pour les commandes composites.

use mira_assembly::api::{Engine, Request, Response};
use mira_assembly::mira_vocabulary::MiraError;

use super::account::AccountCommand;
use super::account::oauth_login::login;
use super::account::sync_wait::settle_first_sync;
use super::waiting::execute_and_wait;
use super::waiting::policy::policy_of;
use crate::arguments::Commands;
use crate::cli_io::CliIo;

/// Exécute la commande ; `account add-oauth` enchaîne autorisation et ajout, `--sync-wait` attend la synchro.
pub async fn run_steps(
    engine: &Engine,
    command: &Commands,
    request: Request,
    io: &mut CliIo<'_>,
) -> Result<Response, MiraError> {
    match command {
        Commands::Account(AccountCommand::AddOauth(args)) => login(engine, args, request, io).await,
        Commands::Account(AccountCommand::Add(args)) => {
            let added = execute_and_wait(engine, request, policy_of(command)).await?;
            settle_first_sync(engine, added, &args.sync).await
        }
        _ => execute_and_wait(engine, request, policy_of(command)).await,
    }
}
