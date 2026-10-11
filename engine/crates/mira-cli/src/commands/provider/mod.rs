//! Commandes `mira provider …` : catalogue et détection.

pub mod detect;
pub mod labels;
pub mod list;

use mira_assembly::ProviderRegistry;

use crate::arguments::ProviderCommand;
use crate::cli_io::CliIo;
use crate::output::OutputMode;

/// Aiguille une commande du domaine `provider` vers son gestionnaire.
pub fn run_provider(
    command: ProviderCommand,
    registry: &ProviderRegistry,
    mode: OutputMode,
    io: &mut CliIo,
) -> i32 {
    match command {
        ProviderCommand::List => list::run_list(registry, mode, io),
        ProviderCommand::Detect { address } => detect::run_detect(registry, &address, mode, io),
    }
}
