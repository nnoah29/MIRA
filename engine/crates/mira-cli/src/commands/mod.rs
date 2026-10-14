//! Commandes du CLI, un sous-dossier par domaine du contrat.

pub mod account;
pub mod auth;
pub mod engine;
pub mod events;
pub mod execution;
pub mod identity;
pub mod operation;
pub mod provider;
pub mod request_builder;

use crate::arguments::{Cli, Commands};
use crate::cli_io::CliIo;
use crate::deps::CliDeps;
use crate::output::OutputMode;

/// Aiguille la commande analysée : catalogue local pour `provider`, moteur pour le reste.
pub async fn dispatch(cli: Cli, io: &mut CliIo<'_>, deps: &mut CliDeps) -> i32 {
    let mode = OutputMode::from_json_flag(cli.json);
    match cli.command {
        Commands::Provider(command) => provider::run_provider(command, &deps.registry, mode, io),
        other => execution::run_command(&other, &cli.profile, mode, io, deps).await,
    }
}
