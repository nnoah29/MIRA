//! Commandes du CLI, un sous-dossier par domaine.

pub mod provider;

use mira_assembly::build_registry;

use crate::arguments::{Cli, Commands};
use crate::cli_io::CliIo;
use crate::output::OutputMode;

/// Aiguille la commande analysée vers le gestionnaire de son domaine.
pub fn dispatch(cli: Cli, io: &mut CliIo) -> i32 {
    let mode = OutputMode::from_json_flag(cli.json);
    match cli.command {
        Commands::Provider(command) => provider::run_provider(command, &build_registry(), mode, io),
    }
}
