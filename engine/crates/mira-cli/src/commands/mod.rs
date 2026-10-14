//! Commandes du CLI, un sous-dossier par domaine du contrat.

pub mod account;
pub mod action;
pub mod auth;
pub mod background_policy;
pub mod draft;
pub mod engine;
pub mod events;
pub mod execution;
pub mod filter_args;
pub mod identity;
pub mod instant_arg;
pub mod key_args;
pub mod list_selection;
pub mod mailbox;
pub mod maintenance;
pub mod message;
pub mod operation;
pub mod paging;
pub mod provider;
pub mod queue;
pub mod request_builder;
pub mod search;
pub mod steps;
pub mod sync;
pub mod targets;
pub mod thread;
pub mod view_args;
pub mod waiting;

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
